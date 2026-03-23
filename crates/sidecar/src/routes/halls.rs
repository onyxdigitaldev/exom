/// Hall management routes: create, list, get, update, delete, leave.
///
/// When a Hall is created, a default "general" text channel is also created,
/// the creator is added as HallBuilder, and the Hall Chest is initialized.

use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use exom_core::{
    Channel, ChannelType, Hall, HallRepository, HallRole, Membership,
};

use crate::state::AppState;
use super::auth::ErrorBody;

fn err(status: StatusCode, msg: &str) -> (StatusCode, Json<ErrorBody>) {
    (status, Json(ErrorBody { error: msg.to_string() }))
}

fn require_user(state: &AppState) -> Result<Uuid, (StatusCode, Json<ErrorBody>)> {
    state.current_user().ok_or_else(|| err(StatusCode::UNAUTHORIZED, "Not authenticated"))
}

#[derive(Deserialize)]
pub struct CreateHallRequest {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Serialize)]
pub struct HallResponse {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub owner_id: String,
    pub icon_hash: Option<String>,
    pub member_count: u64,
}

/// POST /api/halls
pub async fn create_hall(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateHallRequest>,
) -> Result<(StatusCode, Json<HallResponse>), (StatusCode, Json<ErrorBody>)> {
    let user_id = require_user(&state)?;

    if req.name.trim().is_empty() {
        return Err(err(StatusCode::BAD_REQUEST, "Hall name cannot be empty"));
    }

    let db = state.db.lock().unwrap();

    // Create hall
    let mut hall = Hall::new(req.name.clone(), user_id);
    hall.current_host_id = Some(user_id);
    hall.election_epoch = 1;

    if let Some(desc) = req.description {
        hall = hall.with_description(desc);
    }

    db.create_hall(&hall)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    // Add creator as HallBuilder
    let membership = Membership::new(user_id, hall.id, HallRole::HallBuilder);
    db.add_member(&membership)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    // Create default "general" text channel
    let general = Channel::new(hall.id, "general".to_string(), ChannelType::Text);
    db.channels().create(&general)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    // Initialize Hall Chest
    let chest = state.chest.lock().unwrap();
    let _ = chest.init_hall_chest(hall.id, &hall.name, HallRole::HallBuilder);

    Ok((StatusCode::CREATED, Json(HallResponse {
        id: hall.id.to_string(),
        name: hall.name,
        description: hall.description,
        owner_id: hall.owner_id.to_string(),
        icon_hash: hall.icon_hash,
        member_count: 1,
    })))
}

/// GET /api/halls
pub async fn list_halls(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<HallResponse>>, (StatusCode, Json<ErrorBody>)> {
    let user_id = require_user(&state)?;
    let db = state.db.lock().unwrap();

    let halls = db
        .list_halls_for_user(user_id)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    let responses: Vec<HallResponse> = halls
        .into_iter()
        .map(|h| {
            let count = db.list_members(h.id).map(|m| m.len() as u64).unwrap_or(0);
            HallResponse {
                id: h.id.to_string(),
                name: h.name,
                description: h.description,
                owner_id: h.owner_id.to_string(),
                icon_hash: h.icon_hash,
                member_count: count,
            }
        })
        .collect();

    Ok(Json(responses))
}

/// GET /api/halls/:id
pub async fn get_hall(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<HallResponse>, (StatusCode, Json<ErrorBody>)> {
    let _user_id = require_user(&state)?;
    let hall_id = Uuid::parse_str(&id).map_err(|_| err(StatusCode::BAD_REQUEST, "Invalid hall ID"))?;
    let db = state.db.lock().unwrap();

    let hall = db
        .find_hall_by_id(hall_id)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?
        .ok_or_else(|| err(StatusCode::NOT_FOUND, "Hall not found"))?;

    let count = db.list_members(hall.id).map(|m| m.len() as u64).unwrap_or(0);

    Ok(Json(HallResponse {
        id: hall.id.to_string(),
        name: hall.name,
        description: hall.description,
        owner_id: hall.owner_id.to_string(),
        icon_hash: hall.icon_hash,
        member_count: count,
    }))
}

/// DELETE /api/halls/:id/leave
pub async fn leave_hall(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<ErrorBody>)> {
    let user_id = require_user(&state)?;
    let hall_id = Uuid::parse_str(&id).map_err(|_| err(StatusCode::BAD_REQUEST, "Invalid hall ID"))?;
    let db = state.db.lock().unwrap();

    // Cannot leave if owner
    let hall = db
        .find_hall_by_id(hall_id)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?
        .ok_or_else(|| err(StatusCode::NOT_FOUND, "Hall not found"))?;

    if hall.owner_id == user_id {
        return Err(err(StatusCode::FORBIDDEN, "Owner cannot leave their own Hall"));
    }

    db.remove_member(user_id, hall_id)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}
