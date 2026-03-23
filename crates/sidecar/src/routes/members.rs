/// Member routes: list, promote, demote, kick.

use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use exom_core::{HallRepository, HallRole, MemberInfo, PermissionMatrix};

use crate::state::AppState;
use super::auth::ErrorBody;

fn err(status: StatusCode, msg: &str) -> (StatusCode, Json<ErrorBody>) {
    (status, Json(ErrorBody { error: msg.to_string() }))
}

fn require_user(state: &AppState) -> Result<Uuid, (StatusCode, Json<ErrorBody>)> {
    state.current_user().ok_or_else(|| err(StatusCode::UNAUTHORIZED, "Not authenticated"))
}

#[derive(Serialize)]
pub struct MemberResponse {
    pub user_id: String,
    pub username: String,
    pub role: String,
    pub is_online: bool,
    pub is_host: bool,
}

impl From<MemberInfo> for MemberResponse {
    fn from(m: MemberInfo) -> Self {
        Self {
            user_id: m.user_id.to_string(),
            username: m.username,
            role: m.role.short_name().to_string(),
            is_online: m.is_online,
            is_host: m.is_host,
        }
    }
}

#[derive(Deserialize)]
pub struct RoleChangeRequest {
    pub role: String,
}

fn parse_role(s: &str) -> Option<HallRole> {
    match s.to_lowercase().as_str() {
        "builder" => Some(HallRole::HallBuilder),
        "prefect" => Some(HallRole::HallPrefect),
        "moderator" => Some(HallRole::HallModerator),
        "agent" => Some(HallRole::HallAgent),
        "fellow" => Some(HallRole::HallFellow),
        _ => None,
    }
}

/// GET /api/halls/:hall_id/members
pub async fn list_members(
    State(state): State<Arc<AppState>>,
    Path(hall_id): Path<String>,
) -> Result<Json<Vec<MemberResponse>>, (StatusCode, Json<ErrorBody>)> {
    let _user_id = require_user(&state)?;
    let hall_id = Uuid::parse_str(&hall_id).map_err(|_| err(StatusCode::BAD_REQUEST, "Invalid hall ID"))?;

    let db = state.db.lock().unwrap();
    let members = db.list_members(hall_id)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    Ok(Json(members.into_iter().map(MemberResponse::from).collect()))
}

/// PUT /api/halls/:hall_id/members/:user_id/role
pub async fn update_role(
    State(state): State<Arc<AppState>>,
    Path((hall_id, target_user_id)): Path<(String, String)>,
    Json(req): Json<RoleChangeRequest>,
) -> Result<StatusCode, (StatusCode, Json<ErrorBody>)> {
    let actor_id = require_user(&state)?;
    let hall_id = Uuid::parse_str(&hall_id).map_err(|_| err(StatusCode::BAD_REQUEST, "Invalid hall ID"))?;
    let target_id = Uuid::parse_str(&target_user_id).map_err(|_| err(StatusCode::BAD_REQUEST, "Invalid user ID"))?;
    let new_role = parse_role(&req.role).ok_or_else(|| err(StatusCode::BAD_REQUEST, "Invalid role"))?;

    let db = state.db.lock().unwrap();

    let actor_role = db.get_user_role(actor_id, hall_id)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?
        .ok_or_else(|| err(StatusCode::FORBIDDEN, "You are not a member of this Hall"))?;

    let target_role = db.get_user_role(target_id, hall_id)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?
        .ok_or_else(|| err(StatusCode::NOT_FOUND, "Target user is not a member"))?;

    if !PermissionMatrix::can_change_role(actor_role, target_role, new_role) {
        return Err(err(StatusCode::FORBIDDEN, "Insufficient permissions to change this role"));
    }

    db.update_role(target_id, hall_id, new_role)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}

/// DELETE /api/halls/:hall_id/members/:user_id
pub async fn kick_member(
    State(state): State<Arc<AppState>>,
    Path((hall_id, target_user_id)): Path<(String, String)>,
) -> Result<StatusCode, (StatusCode, Json<ErrorBody>)> {
    let actor_id = require_user(&state)?;
    let hall_id = Uuid::parse_str(&hall_id).map_err(|_| err(StatusCode::BAD_REQUEST, "Invalid hall ID"))?;
    let target_id = Uuid::parse_str(&target_user_id).map_err(|_| err(StatusCode::BAD_REQUEST, "Invalid user ID"))?;

    let db = state.db.lock().unwrap();

    let actor_role = db.get_user_role(actor_id, hall_id)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?
        .ok_or_else(|| err(StatusCode::FORBIDDEN, "You are not a member of this Hall"))?;

    let target_role = db.get_user_role(target_id, hall_id)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?
        .ok_or_else(|| err(StatusCode::NOT_FOUND, "Target user is not a member"))?;

    if !PermissionMatrix::can_kick(actor_role, target_role) {
        return Err(err(StatusCode::FORBIDDEN, "Insufficient permissions to kick this member"));
    }

    db.remove_member(target_id, hall_id)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}
