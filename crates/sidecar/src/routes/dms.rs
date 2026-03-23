/// DM routes: create channel, list channels, list messages, send message.
///
/// DM channels are 1-on-1 conversations between two users. If a DM channel
/// already exists between the two users, the existing channel is returned
/// instead of creating a duplicate.

use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use exom_core::{DirectMessage, DmChannel, DmParticipant};

use crate::state::AppState;
use super::auth::ErrorBody;

fn err(status: StatusCode, msg: &str) -> (StatusCode, Json<ErrorBody>) {
    (status, Json(ErrorBody { error: msg.to_string() }))
}

fn require_user(state: &AppState) -> Result<Uuid, (StatusCode, Json<ErrorBody>)> {
    state.current_user().ok_or_else(|| err(StatusCode::UNAUTHORIZED, "Not authenticated"))
}

#[derive(Deserialize)]
pub struct CreateDmRequest {
    pub user_id: String,
}

#[derive(Serialize)]
pub struct DmChannelResponse {
    pub id: String,
    pub channel_type: String,
    pub name: Option<String>,
    pub created_at: String,
    pub last_message_at: Option<String>,
}

impl From<DmChannel> for DmChannelResponse {
    fn from(ch: DmChannel) -> Self {
        let channel_type = match ch.channel_type {
            exom_core::DmChannelType::Direct => "direct",
            exom_core::DmChannelType::Group => "group",
        };
        Self {
            id: ch.id.to_string(),
            channel_type: channel_type.to_string(),
            name: ch.name,
            created_at: ch.created_at.to_rfc3339(),
            last_message_at: ch.last_message_at.map(|t| t.to_rfc3339()),
        }
    }
}

#[derive(Serialize)]
pub struct DmMessageResponse {
    pub id: String,
    pub channel_id: String,
    pub sender_id: String,
    pub content: String,
    pub created_at: String,
    pub edited_at: Option<String>,
    pub reply_to: Option<String>,
}

impl From<DirectMessage> for DmMessageResponse {
    fn from(m: DirectMessage) -> Self {
        Self {
            id: m.id.to_string(),
            channel_id: m.channel_id.to_string(),
            sender_id: m.sender_id.to_string(),
            content: m.content,
            created_at: m.created_at.to_rfc3339(),
            edited_at: m.edited_at.map(|t| t.to_rfc3339()),
            reply_to: m.reply_to.map(|r| r.to_string()),
        }
    }
}

#[derive(Deserialize)]
pub struct SendDmRequest {
    pub content: String,
    pub reply_to: Option<String>,
}

#[derive(Deserialize)]
pub struct ListDmMessagesQuery {
    pub limit: Option<u32>,
    pub before: Option<String>,
}

/// POST /api/dms
pub async fn create_dm(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateDmRequest>,
) -> Result<(StatusCode, Json<DmChannelResponse>), (StatusCode, Json<ErrorBody>)> {
    let user_id = require_user(&state)?;
    let target_id = Uuid::parse_str(&req.user_id)
        .map_err(|_| err(StatusCode::BAD_REQUEST, "Invalid target user ID"))?;

    if user_id == target_id {
        return Err(err(StatusCode::BAD_REQUEST, "Cannot create a DM with yourself"));
    }

    let db = state.db.lock().unwrap();

    // Check if a DM channel already exists between these two users
    if let Some(existing) = db.dms().find_direct_channel(user_id, target_id)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?
    {
        return Ok((StatusCode::OK, Json(DmChannelResponse::from(existing))));
    }

    // Create the channel and add both participants
    let channel = DmChannel::new_direct();

    db.dms().create_channel(&channel)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    let participant_self = DmParticipant::new(channel.id, user_id);
    db.dms().add_participant(&participant_self)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    let participant_target = DmParticipant::new(channel.id, target_id);
    db.dms().add_participant(&participant_target)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    Ok((StatusCode::CREATED, Json(DmChannelResponse::from(channel))))
}

/// GET /api/dms
pub async fn list_dms(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<DmChannelResponse>>, (StatusCode, Json<ErrorBody>)> {
    let user_id = require_user(&state)?;

    let db = state.db.lock().unwrap();
    let channels = db.dms().list_channels_for_user(user_id)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    Ok(Json(channels.into_iter().map(DmChannelResponse::from).collect()))
}

/// GET /api/dms/:channel_id/messages
pub async fn list_dm_messages(
    State(state): State<Arc<AppState>>,
    Path(channel_id): Path<String>,
    Query(query): Query<ListDmMessagesQuery>,
) -> Result<Json<Vec<DmMessageResponse>>, (StatusCode, Json<ErrorBody>)> {
    let _user_id = require_user(&state)?;
    let channel_id = Uuid::parse_str(&channel_id)
        .map_err(|_| err(StatusCode::BAD_REQUEST, "Invalid channel ID"))?;

    let limit = query.limit.unwrap_or(50).min(100);
    let before: Option<DateTime<Utc>> = query
        .before
        .as_deref()
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&Utc));

    let db = state.db.lock().unwrap();
    let messages = db.dms().list_messages(channel_id, limit, before)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    Ok(Json(messages.into_iter().map(DmMessageResponse::from).collect()))
}

/// POST /api/dms/:channel_id/messages
pub async fn send_dm(
    State(state): State<Arc<AppState>>,
    Path(channel_id): Path<String>,
    Json(req): Json<SendDmRequest>,
) -> Result<(StatusCode, Json<DmMessageResponse>), (StatusCode, Json<ErrorBody>)> {
    let user_id = require_user(&state)?;
    let channel_id = Uuid::parse_str(&channel_id)
        .map_err(|_| err(StatusCode::BAD_REQUEST, "Invalid channel ID"))?;

    if req.content.trim().is_empty() {
        return Err(err(StatusCode::BAD_REQUEST, "Message content cannot be empty"));
    }

    let mut message = DirectMessage::new(channel_id, user_id, req.content);

    if let Some(reply) = &req.reply_to {
        let reply_id = Uuid::parse_str(reply)
            .map_err(|_| err(StatusCode::BAD_REQUEST, "Invalid reply ID"))?;
        message = message.with_reply(reply_id);
    }

    let db = state.db.lock().unwrap();
    db.dms().create_message(&message)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    Ok((StatusCode::CREATED, Json(DmMessageResponse::from(message))))
}
