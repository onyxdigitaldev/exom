/// Notification routes: mark channel as read, list unread channels.
///
/// Read state tracks the last message each user has seen in each channel,
/// along with a count of unread mentions.

use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::state::AppState;
use super::auth::ErrorBody;

fn err(status: StatusCode, msg: &str) -> (StatusCode, Json<ErrorBody>) {
    (status, Json(ErrorBody { error: msg.to_string() }))
}

fn require_user(state: &AppState) -> Result<Uuid, (StatusCode, Json<ErrorBody>)> {
    state.current_user().ok_or_else(|| err(StatusCode::UNAUTHORIZED, "Not authenticated"))
}

#[derive(Deserialize)]
pub struct MarkReadRequest {
    pub message_id: String,
}

#[derive(Serialize)]
pub struct UnreadChannelResponse {
    pub channel_id: String,
    pub last_read_message_id: Option<String>,
    pub mention_count: u32,
}

/// POST /api/channels/:channel_id/read
pub async fn mark_read(
    State(state): State<Arc<AppState>>,
    Path(channel_id): Path<String>,
    Json(req): Json<MarkReadRequest>,
) -> Result<StatusCode, (StatusCode, Json<ErrorBody>)> {
    let user_id = require_user(&state)?;
    let channel_id = Uuid::parse_str(&channel_id)
        .map_err(|_| err(StatusCode::BAD_REQUEST, "Invalid channel ID"))?;
    let message_id = Uuid::parse_str(&req.message_id)
        .map_err(|_| err(StatusCode::BAD_REQUEST, "Invalid message ID"))?;

    let db = state.db.lock().unwrap();
    db.notifications().mark_read(user_id, channel_id, message_id)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}

/// GET /api/notifications/unread
pub async fn list_unread(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<UnreadChannelResponse>>, (StatusCode, Json<ErrorBody>)> {
    let user_id = require_user(&state)?;

    let db = state.db.lock().unwrap();
    let states = db.notifications().list_unread_channels(user_id)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    let responses: Vec<UnreadChannelResponse> = states
        .into_iter()
        .map(|s| UnreadChannelResponse {
            channel_id: s.channel_id.to_string(),
            last_read_message_id: s.last_read_message_id.map(|id| id.to_string()),
            mention_count: s.mention_count,
        })
        .collect();

    Ok(Json(responses))
}
