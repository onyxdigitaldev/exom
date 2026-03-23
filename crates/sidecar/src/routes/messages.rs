/// Message routes: send, list, edit, delete, pin, unpin, list pinned.

use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use exom_core::{Message, MessageDisplay, MessageRepository, UserRepository};

use crate::state::AppState;
use super::auth::ErrorBody;

fn err(status: StatusCode, msg: &str) -> (StatusCode, Json<ErrorBody>) {
    (status, Json(ErrorBody { error: msg.to_string() }))
}

fn require_user(state: &AppState) -> Result<Uuid, (StatusCode, Json<ErrorBody>)> {
    state.current_user().ok_or_else(|| err(StatusCode::UNAUTHORIZED, "Not authenticated"))
}

#[derive(Deserialize)]
pub struct SendMessageRequest {
    pub content: String,
    pub reply_to: Option<String>,
}

#[derive(Deserialize)]
pub struct ListMessagesQuery {
    pub limit: Option<u32>,
    pub before: Option<String>,
}

#[derive(Serialize)]
pub struct MessageResponse {
    pub id: String,
    pub sender_id: String,
    pub sender_username: String,
    pub sender_role: String,
    pub content: String,
    pub timestamp: String,
    pub is_edited: bool,
    pub reply_to: Option<String>,
    pub thread_id: Option<String>,
    pub is_pinned: bool,
    pub reaction_count: u32,
    pub thread_reply_count: u32,
}

impl From<MessageDisplay> for MessageResponse {
    fn from(m: MessageDisplay) -> Self {
        Self {
            id: m.id.to_string(),
            sender_id: m.sender_id.to_string(),
            sender_username: m.sender_username,
            sender_role: m.sender_role.short_name().to_string(),
            content: m.content,
            timestamp: m.timestamp.to_rfc3339(),
            is_edited: m.is_edited,
            reply_to: m.reply_to.map(|r| r.to_string()),
            thread_id: m.thread_id.map(|t| t.to_string()),
            is_pinned: m.is_pinned,
            reaction_count: m.reaction_count,
            thread_reply_count: m.thread_reply_count,
        }
    }
}

/// POST /api/halls/:hall_id/channels/:channel_id/messages
pub async fn send_message(
    State(state): State<Arc<AppState>>,
    Path((hall_id, channel_id)): Path<(String, String)>,
    Json(req): Json<SendMessageRequest>,
) -> Result<(StatusCode, Json<MessageResponse>), (StatusCode, Json<ErrorBody>)> {
    let user_id = require_user(&state)?;
    let hall_id = Uuid::parse_str(&hall_id).map_err(|_| err(StatusCode::BAD_REQUEST, "Invalid hall ID"))?;
    let channel_id = Uuid::parse_str(&channel_id).map_err(|_| err(StatusCode::BAD_REQUEST, "Invalid channel ID"))?;

    if req.content.trim().is_empty() {
        return Err(err(StatusCode::BAD_REQUEST, "Message content cannot be empty"));
    }

    let mut message = Message::new(channel_id, hall_id, user_id, req.content.clone());

    if let Some(reply) = &req.reply_to {
        let reply_id = Uuid::parse_str(reply).map_err(|_| err(StatusCode::BAD_REQUEST, "Invalid reply ID"))?;
        message = message.with_reply(reply_id);
    }

    let db = state.db.lock().unwrap();
    db.create_message(&message)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    // Update channel last_message_at
    let _ = db.channels().touch_last_message(channel_id);

    // Fetch the display version
    let user = db.find_user_by_id(user_id)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?
        .ok_or_else(|| err(StatusCode::INTERNAL_SERVER_ERROR, "User not found"))?;

    let role = db.halls().get_user_role(user_id, hall_id)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?
        .unwrap_or(exom_core::HallRole::HallFellow);

    Ok((StatusCode::CREATED, Json(MessageResponse {
        id: message.id.to_string(),
        sender_id: user_id.to_string(),
        sender_username: user.username,
        sender_role: role.short_name().to_string(),
        content: req.content,
        timestamp: message.created_at.to_rfc3339(),
        is_edited: false,
        reply_to: message.reply_to.map(|r| r.to_string()),
        thread_id: None,
        is_pinned: false,
        reaction_count: 0,
        thread_reply_count: 0,
    })))
}

/// GET /api/halls/:hall_id/channels/:channel_id/messages
pub async fn list_messages(
    State(state): State<Arc<AppState>>,
    Path((_hall_id, channel_id)): Path<(String, String)>,
    Query(query): Query<ListMessagesQuery>,
) -> Result<Json<Vec<MessageResponse>>, (StatusCode, Json<ErrorBody>)> {
    let _user_id = require_user(&state)?;
    let channel_id = Uuid::parse_str(&channel_id).map_err(|_| err(StatusCode::BAD_REQUEST, "Invalid channel ID"))?;

    let limit = query.limit.unwrap_or(50).min(100);
    let before: Option<DateTime<Utc>> = query
        .before
        .as_deref()
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&Utc));

    let db = state.db.lock().unwrap();
    let messages = db
        .list_messages_for_channel(channel_id, limit, before)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    Ok(Json(messages.into_iter().map(MessageResponse::from).collect()))
}

/// PUT /api/messages/:id
pub async fn edit_message(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<SendMessageRequest>,
) -> Result<StatusCode, (StatusCode, Json<ErrorBody>)> {
    let _user_id = require_user(&state)?;
    let message_id = Uuid::parse_str(&id).map_err(|_| err(StatusCode::BAD_REQUEST, "Invalid message ID"))?;

    let db = state.db.lock().unwrap();
    db.update_message_content(message_id, &req.content)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}

/// DELETE /api/messages/:id
pub async fn delete_message(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<ErrorBody>)> {
    let _user_id = require_user(&state)?;
    let message_id = Uuid::parse_str(&id).map_err(|_| err(StatusCode::BAD_REQUEST, "Invalid message ID"))?;

    let db = state.db.lock().unwrap();
    db.delete_message(message_id)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}

/// POST /api/messages/:id/pin
pub async fn pin_message(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<ErrorBody>)> {
    let _user_id = require_user(&state)?;
    let message_id = Uuid::parse_str(&id).map_err(|_| err(StatusCode::BAD_REQUEST, "Invalid message ID"))?;

    let db = state.db.lock().unwrap();
    db.pin_message(message_id)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}

/// DELETE /api/messages/:id/pin
pub async fn unpin_message(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<ErrorBody>)> {
    let _user_id = require_user(&state)?;
    let message_id = Uuid::parse_str(&id).map_err(|_| err(StatusCode::BAD_REQUEST, "Invalid message ID"))?;

    let db = state.db.lock().unwrap();
    db.unpin_message(message_id)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}
