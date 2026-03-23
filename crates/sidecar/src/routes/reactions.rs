/// Reaction routes: add, remove, list summaries.
///
/// Reactions are unicode emoji strings attached to messages. Each user can
/// react with a given emoji once per message.

use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use exom_core::{Reaction, ReactionSummary};

use crate::state::AppState;
use super::auth::ErrorBody;

fn err(status: StatusCode, msg: &str) -> (StatusCode, Json<ErrorBody>) {
    (status, Json(ErrorBody { error: msg.to_string() }))
}

fn require_user(state: &AppState) -> Result<Uuid, (StatusCode, Json<ErrorBody>)> {
    state.current_user().ok_or_else(|| err(StatusCode::UNAUTHORIZED, "Not authenticated"))
}

#[derive(Deserialize)]
pub struct AddReactionRequest {
    pub emoji: String,
}

#[derive(Serialize)]
pub struct ReactionSummaryResponse {
    pub emoji: String,
    pub is_custom: bool,
    pub custom_emoji_id: Option<String>,
    pub count: u32,
    pub me: bool,
}

impl From<ReactionSummary> for ReactionSummaryResponse {
    fn from(s: ReactionSummary) -> Self {
        Self {
            emoji: s.emoji,
            is_custom: s.is_custom,
            custom_emoji_id: s.custom_emoji_id.map(|id| id.to_string()),
            count: s.count,
            me: s.me,
        }
    }
}

/// POST /api/messages/:message_id/reactions
pub async fn add_reaction(
    State(state): State<Arc<AppState>>,
    Path(message_id): Path<String>,
    Json(req): Json<AddReactionRequest>,
) -> Result<StatusCode, (StatusCode, Json<ErrorBody>)> {
    let user_id = require_user(&state)?;
    let message_id = Uuid::parse_str(&message_id)
        .map_err(|_| err(StatusCode::BAD_REQUEST, "Invalid message ID"))?;

    if req.emoji.trim().is_empty() {
        return Err(err(StatusCode::BAD_REQUEST, "Emoji cannot be empty"));
    }

    let reaction = Reaction::new_unicode(message_id, user_id, req.emoji);

    let db = state.db.lock().unwrap();
    db.reactions().create(&reaction)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    Ok(StatusCode::CREATED)
}

/// DELETE /api/messages/:message_id/reactions/:emoji
pub async fn remove_reaction(
    State(state): State<Arc<AppState>>,
    Path((message_id, emoji)): Path<(String, String)>,
) -> Result<StatusCode, (StatusCode, Json<ErrorBody>)> {
    let user_id = require_user(&state)?;
    let message_id = Uuid::parse_str(&message_id)
        .map_err(|_| err(StatusCode::BAD_REQUEST, "Invalid message ID"))?;

    let db = state.db.lock().unwrap();
    db.reactions().delete(message_id, user_id, &emoji)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}

/// GET /api/messages/:message_id/reactions
pub async fn list_reactions(
    State(state): State<Arc<AppState>>,
    Path(message_id): Path<String>,
) -> Result<Json<Vec<ReactionSummaryResponse>>, (StatusCode, Json<ErrorBody>)> {
    let user_id = require_user(&state)?;
    let message_id = Uuid::parse_str(&message_id)
        .map_err(|_| err(StatusCode::BAD_REQUEST, "Invalid message ID"))?;

    let db = state.db.lock().unwrap();
    let summaries = db.reactions().list_summaries(message_id, user_id)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    Ok(Json(summaries.into_iter().map(ReactionSummaryResponse::from).collect()))
}
