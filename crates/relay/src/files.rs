//! File upload/download HTTP endpoints
//!
//! Upload: PUT /upload/:filename with raw body bytes
//! Download: GET /files/:hash/:filename

use std::sync::Arc;

use axum::{
    body::{Body, Bytes},
    extract::{Path, State},
    http::{header, HeaderMap, StatusCode},
    response::Response,
    Json,
};
use serde::Serialize;
use sha2::{Digest, Sha256};
use tokio::fs;
use tracing::info;

use crate::state::RelayState;

/// Maximum file size: 25 MB
const MAX_FILE_SIZE: usize = 25 * 1024 * 1024;

#[derive(Serialize)]
pub struct UploadResponse {
    pub hash: String,
    pub filename: String,
    pub size_bytes: u64,
    pub content_type: Option<String>,
    pub url: String,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

/// PUT /upload/:filename — raw body file upload
pub async fn upload_file(
    State(state): State<Arc<RelayState>>,
    Path(filename): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<UploadResponse>, (StatusCode, Json<ErrorResponse>)> {
    if body.len() > MAX_FILE_SIZE {
        return Err((
            StatusCode::PAYLOAD_TOO_LARGE,
            Json(ErrorResponse {
                error: format!(
                    "File too large: {} bytes (max {} bytes)",
                    body.len(),
                    MAX_FILE_SIZE
                ),
            }),
        ));
    }

    if body.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Empty file body".to_string(),
            }),
        ));
    }

    let content_type = headers
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    // Hash for deduplication
    let mut hasher = Sha256::new();
    hasher.update(&body);
    let hash = format!("{:x}", hasher.finalize());

    // Store under hash-based path (first 2 chars as subdirectory)
    let file_dir = state.file_storage_path.join(&hash[..2]);
    fs::create_dir_all(&file_dir).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Storage error: {}", e),
            }),
        )
    })?;

    let file_path = file_dir.join(&hash);
    if !file_path.exists() {
        fs::write(&file_path, &body).await.map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Write error: {}", e),
                }),
            )
        })?;
    }

    let size_bytes = body.len() as u64;
    let url = format!("/files/{}/{}", hash, filename);

    info!(
        hash = %hash,
        filename = %filename,
        size = size_bytes,
        "file uploaded"
    );

    Ok(Json(UploadResponse {
        hash,
        filename,
        size_bytes,
        content_type,
        url,
    }))
}

/// GET /files/:hash/:filename — file download
pub async fn download_file(
    State(state): State<Arc<RelayState>>,
    Path((hash, filename)): Path<(String, String)>,
) -> Result<Response, StatusCode> {
    // Validate hash format (hex, 64 chars)
    if hash.len() != 64 || !hash.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(StatusCode::BAD_REQUEST);
    }

    let file_path = state.file_storage_path.join(&hash[..2]).join(&hash);

    if !file_path.exists() {
        return Err(StatusCode::NOT_FOUND);
    }

    let data = fs::read(&file_path)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let content_type = mime_from_filename(&filename);

    let response = Response::builder()
        .header(header::CONTENT_TYPE, content_type)
        .header(
            header::CONTENT_DISPOSITION,
            format!("inline; filename=\"{}\"", filename),
        )
        .header(header::CACHE_CONTROL, "public, max-age=31536000, immutable")
        .body(Body::from(data))
        .unwrap();

    Ok(response)
}

fn mime_from_filename(filename: &str) -> &'static str {
    let ext = filename.rsplit('.').next().unwrap_or("").to_lowercase();
    match ext.as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "svg" => "image/svg+xml",
        "mp4" => "video/mp4",
        "webm" => "video/webm",
        "mp3" => "audio/mpeg",
        "ogg" => "audio/ogg",
        "wav" => "audio/wav",
        "pdf" => "application/pdf",
        "zip" => "application/zip",
        "txt" => "text/plain",
        "json" => "application/json",
        _ => "application/octet-stream",
    }
}
