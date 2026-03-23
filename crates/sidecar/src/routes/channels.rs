/// Channel routes: create, list, update, delete, reorder.

use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use exom_core::{Channel, ChannelType};

use crate::state::AppState;
use super::auth::ErrorBody;

fn err(status: StatusCode, msg: &str) -> (StatusCode, Json<ErrorBody>) {
    (status, Json(ErrorBody { error: msg.to_string() }))
}

fn require_user(state: &AppState) -> Result<Uuid, (StatusCode, Json<ErrorBody>)> {
    state.current_user().ok_or_else(|| err(StatusCode::UNAUTHORIZED, "Not authenticated"))
}

fn parse_channel_type(s: &str) -> ChannelType {
    match s {
        "voice" => ChannelType::Voice,
        "category" => ChannelType::Category,
        "announcement" => ChannelType::Announcement,
        "stage" => ChannelType::Stage,
        _ => ChannelType::Text,
    }
}

#[derive(Deserialize)]
pub struct CreateChannelRequest {
    pub name: String,
    pub channel_type: String,
    pub topic: Option<String>,
    pub parent_id: Option<String>,
    pub nsfw: Option<bool>,
}

#[derive(Serialize)]
pub struct ChannelResponse {
    pub id: String,
    pub hall_id: String,
    pub name: String,
    pub channel_type: String,
    pub topic: Option<String>,
    pub parent_id: Option<String>,
    pub position: i32,
    pub nsfw: bool,
}

impl From<Channel> for ChannelResponse {
    fn from(ch: Channel) -> Self {
        Self {
            id: ch.id.to_string(),
            hall_id: ch.hall_id.to_string(),
            name: ch.name,
            channel_type: ch.channel_type.display_name().to_lowercase(),
            topic: ch.topic,
            parent_id: ch.parent_id.map(|p| p.to_string()),
            position: ch.position,
            nsfw: ch.nsfw,
        }
    }
}

/// POST /api/halls/:hall_id/channels
pub async fn create_channel(
    State(state): State<Arc<AppState>>,
    Path(hall_id): Path<String>,
    Json(req): Json<CreateChannelRequest>,
) -> Result<(StatusCode, Json<ChannelResponse>), (StatusCode, Json<ErrorBody>)> {
    let _user_id = require_user(&state)?;
    let hall_id = Uuid::parse_str(&hall_id).map_err(|_| err(StatusCode::BAD_REQUEST, "Invalid hall ID"))?;

    let channel_type = parse_channel_type(&req.channel_type);
    let mut channel = Channel::new(hall_id, req.name, channel_type);

    if let Some(topic) = req.topic {
        channel = channel.with_topic(topic);
    }
    if let Some(parent) = req.parent_id {
        let pid = Uuid::parse_str(&parent).map_err(|_| err(StatusCode::BAD_REQUEST, "Invalid parent ID"))?;
        channel = channel.with_parent(pid);
    }
    if req.nsfw.unwrap_or(false) {
        channel.nsfw = true;
    }

    let db = state.db.lock().unwrap();
    db.channels().create(&channel)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    Ok((StatusCode::CREATED, Json(ChannelResponse::from(channel))))
}

/// GET /api/halls/:hall_id/channels
pub async fn list_channels(
    State(state): State<Arc<AppState>>,
    Path(hall_id): Path<String>,
) -> Result<Json<Vec<ChannelResponse>>, (StatusCode, Json<ErrorBody>)> {
    let _user_id = require_user(&state)?;
    let hall_id = Uuid::parse_str(&hall_id).map_err(|_| err(StatusCode::BAD_REQUEST, "Invalid hall ID"))?;

    let db = state.db.lock().unwrap();
    let channels = db.channels().list_for_hall(hall_id)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    Ok(Json(channels.into_iter().map(ChannelResponse::from).collect()))
}

/// DELETE /api/channels/:id
pub async fn delete_channel(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<ErrorBody>)> {
    let _user_id = require_user(&state)?;
    let channel_id = Uuid::parse_str(&id).map_err(|_| err(StatusCode::BAD_REQUEST, "Invalid channel ID"))?;

    let db = state.db.lock().unwrap();
    db.channels().delete(channel_id)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}
