/// Moderation routes: ban, unban, list bans, audit log.
///
/// Bans remove the target user from the hall and prevent re-entry.
/// The audit log records all moderation actions for accountability.

use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use exom_core::{AuditLogEntry, Ban, HallRepository};

use crate::state::AppState;
use super::auth::ErrorBody;

fn err(status: StatusCode, msg: &str) -> (StatusCode, Json<ErrorBody>) {
    (status, Json(ErrorBody { error: msg.to_string() }))
}

fn require_user(state: &AppState) -> Result<Uuid, (StatusCode, Json<ErrorBody>)> {
    state.current_user().ok_or_else(|| err(StatusCode::UNAUTHORIZED, "Not authenticated"))
}

#[derive(Deserialize)]
pub struct BanRequest {
    pub user_id: String,
    pub reason: Option<String>,
    pub duration_hours: Option<u64>,
}

#[derive(Serialize)]
pub struct BanResponse {
    pub id: String,
    pub hall_id: String,
    pub user_id: String,
    pub banned_by: String,
    pub reason: Option<String>,
    pub created_at: String,
    pub expires_at: Option<String>,
}

impl From<Ban> for BanResponse {
    fn from(b: Ban) -> Self {
        Self {
            id: b.id.to_string(),
            hall_id: b.hall_id.to_string(),
            user_id: b.user_id.to_string(),
            banned_by: b.banned_by.to_string(),
            reason: b.reason,
            created_at: b.created_at.to_rfc3339(),
            expires_at: b.expires_at.map(|t| t.to_rfc3339()),
        }
    }
}

#[derive(Serialize)]
pub struct AuditLogResponse {
    pub id: String,
    pub hall_id: String,
    pub actor_id: String,
    pub action_type: String,
    pub target_type: Option<String>,
    pub target_id: Option<String>,
    pub changes: Option<String>,
    pub reason: Option<String>,
    pub created_at: String,
}

impl From<AuditLogEntry> for AuditLogResponse {
    fn from(e: AuditLogEntry) -> Self {
        Self {
            id: e.id.to_string(),
            hall_id: e.hall_id.to_string(),
            actor_id: e.actor_id.to_string(),
            action_type: e.action_type.as_str().to_string(),
            target_type: e.target_type.as_ref().map(|t| t.as_str().to_string()),
            target_id: e.target_id.map(|t| t.to_string()),
            changes: e.changes,
            reason: e.reason,
            created_at: e.created_at.to_rfc3339(),
        }
    }
}

#[derive(Deserialize)]
pub struct AuditLogQuery {
    pub limit: Option<u32>,
}

/// POST /api/halls/:hall_id/bans
pub async fn ban_user(
    State(state): State<Arc<AppState>>,
    Path(hall_id): Path<String>,
    Json(req): Json<BanRequest>,
) -> Result<(StatusCode, Json<BanResponse>), (StatusCode, Json<ErrorBody>)> {
    let actor_id = require_user(&state)?;
    let hall_id = Uuid::parse_str(&hall_id).map_err(|_| err(StatusCode::BAD_REQUEST, "Invalid hall ID"))?;
    let target_id = Uuid::parse_str(&req.user_id).map_err(|_| err(StatusCode::BAD_REQUEST, "Invalid user ID"))?;

    if actor_id == target_id {
        return Err(err(StatusCode::BAD_REQUEST, "You cannot ban yourself"));
    }

    let db = state.db.lock().unwrap();

    // Verify the actor is a member of the hall
    let _actor_role = db.get_user_role(actor_id, hall_id)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?
        .ok_or_else(|| err(StatusCode::FORBIDDEN, "You are not a member of this Hall"))?;

    // Check if already banned
    if db.moderation().is_banned(hall_id, target_id)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?
    {
        return Err(err(StatusCode::CONFLICT, "User is already banned"));
    }

    let mut ban = Ban::new(hall_id, target_id, actor_id);

    if let Some(reason) = req.reason {
        ban = ban.with_reason(reason);
    }

    if let Some(hours) = req.duration_hours {
        ban = ban.with_expiry(hours);
    }

    db.moderation().create_ban(&ban)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    // Remove the banned user from the hall
    let _ = db.remove_member(target_id, hall_id);

    Ok((StatusCode::CREATED, Json(BanResponse::from(ban))))
}

/// DELETE /api/halls/:hall_id/bans/:user_id
pub async fn unban_user(
    State(state): State<Arc<AppState>>,
    Path((hall_id, target_user_id)): Path<(String, String)>,
) -> Result<StatusCode, (StatusCode, Json<ErrorBody>)> {
    let _actor_id = require_user(&state)?;
    let hall_id = Uuid::parse_str(&hall_id).map_err(|_| err(StatusCode::BAD_REQUEST, "Invalid hall ID"))?;
    let target_id = Uuid::parse_str(&target_user_id).map_err(|_| err(StatusCode::BAD_REQUEST, "Invalid user ID"))?;

    let db = state.db.lock().unwrap();
    db.moderation().delete_ban(hall_id, target_id)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}

/// GET /api/halls/:hall_id/bans
pub async fn list_bans(
    State(state): State<Arc<AppState>>,
    Path(hall_id): Path<String>,
) -> Result<Json<Vec<BanResponse>>, (StatusCode, Json<ErrorBody>)> {
    let _user_id = require_user(&state)?;
    let hall_id = Uuid::parse_str(&hall_id).map_err(|_| err(StatusCode::BAD_REQUEST, "Invalid hall ID"))?;

    let db = state.db.lock().unwrap();
    let bans = db.moderation().list_bans(hall_id)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    Ok(Json(bans.into_iter().map(BanResponse::from).collect()))
}

/// GET /api/halls/:hall_id/audit-log
pub async fn list_audit_log(
    State(state): State<Arc<AppState>>,
    Path(hall_id): Path<String>,
    Query(query): Query<AuditLogQuery>,
) -> Result<Json<Vec<AuditLogResponse>>, (StatusCode, Json<ErrorBody>)> {
    let _user_id = require_user(&state)?;
    let hall_id = Uuid::parse_str(&hall_id).map_err(|_| err(StatusCode::BAD_REQUEST, "Invalid hall ID"))?;

    let limit = query.limit.unwrap_or(50).min(100);

    let db = state.db.lock().unwrap();
    let entries = db.moderation().list_audit_log(hall_id, limit, None)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    Ok(Json(entries.into_iter().map(AuditLogResponse::from).collect()))
}
