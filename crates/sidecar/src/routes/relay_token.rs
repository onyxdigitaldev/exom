/// Relay token route: generate an HMAC-signed auth token for WebSocket relay.
///
/// The token is signed with the relay secret configured in AppState. The relay
/// server validates this token to authenticate the WebSocket connection.

use std::sync::Arc;

use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use serde::Serialize;
use uuid::Uuid;

use exom_core::generate_auth_token;

use crate::state::AppState;
use super::auth::ErrorBody;

fn err(status: StatusCode, msg: &str) -> (StatusCode, Json<ErrorBody>) {
    (status, Json(ErrorBody { error: msg.to_string() }))
}

fn require_user(state: &AppState) -> Result<Uuid, (StatusCode, Json<ErrorBody>)> {
    state.current_user().ok_or_else(|| err(StatusCode::UNAUTHORIZED, "Not authenticated"))
}

#[derive(Serialize)]
pub struct RelayTokenResponse {
    pub user_id: String,
    pub token: String,
    pub timestamp: String,
    pub protocol_version: u32,
}

/// POST /api/relay/token
pub async fn generate_token(
    State(state): State<Arc<AppState>>,
) -> Result<Json<RelayTokenResponse>, (StatusCode, Json<ErrorBody>)> {
    let user_id = require_user(&state)?;

    let payload = generate_auth_token(user_id, &state.relay_secret);

    Ok(Json(RelayTokenResponse {
        user_id: payload.user_id.to_string(),
        token: payload.token,
        timestamp: payload.timestamp.to_rfc3339(),
        protocol_version: payload.protocol_version,
    }))
}
