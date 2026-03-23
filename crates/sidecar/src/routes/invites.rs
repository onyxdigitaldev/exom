/// Invite routes: create, list, accept, revoke.
///
/// Invite tokens are 16-character alphanumeric strings with a 7-day default
/// expiry. Accepting an invite adds the user as a HallFellow.

use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use rand::Rng;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use exom_core::{HallRepository, HallRole, Invite, InviteRepository, Membership};

use crate::state::AppState;
use super::auth::ErrorBody;

fn err(status: StatusCode, msg: &str) -> (StatusCode, Json<ErrorBody>) {
    (status, Json(ErrorBody { error: msg.to_string() }))
}

fn require_user(state: &AppState) -> Result<Uuid, (StatusCode, Json<ErrorBody>)> {
    state.current_user().ok_or_else(|| err(StatusCode::UNAUTHORIZED, "Not authenticated"))
}

/// Generate a 16-character alphanumeric invite token.
fn generate_token() -> String {
    let mut rng = rand::thread_rng();
    (0..16).map(|_| {
        let idx = rng.gen_range(0..36);
        if idx < 10 { (b'0' + idx) as char } else { (b'a' + idx - 10) as char }
    }).collect()
}

#[derive(Deserialize)]
pub struct CreateInviteRequest {
    pub max_uses: Option<u32>,
    pub expiry_hours: Option<i64>,
}

#[derive(Serialize)]
pub struct InviteResponse {
    pub id: String,
    pub hall_id: String,
    pub token: String,
    pub created_by: String,
    pub role: String,
    pub created_at: String,
    pub expires_at: Option<String>,
    pub max_uses: Option<u32>,
    pub use_count: u32,
    pub is_revoked: bool,
}

impl From<Invite> for InviteResponse {
    fn from(inv: Invite) -> Self {
        Self {
            id: inv.id.to_string(),
            hall_id: inv.hall_id.to_string(),
            token: inv.token,
            created_by: inv.created_by.to_string(),
            role: format!("{:?}", inv.role),
            created_at: inv.created_at.to_rfc3339(),
            expires_at: inv.expires_at.map(|t| t.to_rfc3339()),
            max_uses: inv.max_uses,
            use_count: inv.use_count,
            is_revoked: inv.is_revoked,
        }
    }
}

#[derive(Serialize)]
pub struct AcceptInviteResponse {
    pub hall_id: String,
    pub user_id: String,
}

/// POST /api/halls/:hall_id/invites
pub async fn create_invite(
    State(state): State<Arc<AppState>>,
    Path(hall_id): Path<String>,
    Json(req): Json<CreateInviteRequest>,
) -> Result<(StatusCode, Json<InviteResponse>), (StatusCode, Json<ErrorBody>)> {
    let user_id = require_user(&state)?;
    let hall_id = Uuid::parse_str(&hall_id).map_err(|_| err(StatusCode::BAD_REQUEST, "Invalid hall ID"))?;

    let db = state.db.lock().unwrap();

    // Verify the user is a member of this hall
    let _role = db.get_user_role(user_id, hall_id)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?
        .ok_or_else(|| err(StatusCode::FORBIDDEN, "You are not a member of this Hall"))?;

    let token = generate_token();
    let expiry_hours = req.expiry_hours.unwrap_or(168); // 7 days default
    let mut invite = Invite::new(hall_id, user_id, HallRole::HallFellow, token);
    invite = invite.with_expiry(expiry_hours);

    if let Some(max) = req.max_uses {
        invite = invite.with_max_uses(max);
    }

    db.create_invite(&invite)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    Ok((StatusCode::CREATED, Json(InviteResponse::from(invite))))
}

/// GET /api/halls/:hall_id/invites
pub async fn list_invites(
    State(state): State<Arc<AppState>>,
    Path(hall_id): Path<String>,
) -> Result<Json<Vec<InviteResponse>>, (StatusCode, Json<ErrorBody>)> {
    let user_id = require_user(&state)?;
    let hall_id = Uuid::parse_str(&hall_id).map_err(|_| err(StatusCode::BAD_REQUEST, "Invalid hall ID"))?;

    let db = state.db.lock().unwrap();

    // Verify membership
    let _role = db.get_user_role(user_id, hall_id)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?
        .ok_or_else(|| err(StatusCode::FORBIDDEN, "You are not a member of this Hall"))?;

    let invites = db.list_invites_for_hall(hall_id)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    Ok(Json(invites.into_iter().map(InviteResponse::from).collect()))
}

/// POST /api/invites/:token/accept
pub async fn accept_invite(
    State(state): State<Arc<AppState>>,
    Path(token): Path<String>,
) -> Result<Json<AcceptInviteResponse>, (StatusCode, Json<ErrorBody>)> {
    let user_id = require_user(&state)?;

    let db = state.db.lock().unwrap();

    let invite = db.find_invite_by_token(&token)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?
        .ok_or_else(|| err(StatusCode::NOT_FOUND, "Invite not found"))?;

    if !invite.is_valid() {
        return Err(err(StatusCode::GONE, "Invite has expired or been revoked"));
    }

    // Check if the user is already a member
    if db.get_user_role(user_id, invite.hall_id)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?
        .is_some()
    {
        return Err(err(StatusCode::CONFLICT, "You are already a member of this Hall"));
    }

    // Add user as a member with the invite's role
    let membership = Membership::new(user_id, invite.hall_id, invite.role);
    db.add_member(&membership)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    // Increment the invite use count
    db.increment_use_count(invite.id)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    Ok(Json(AcceptInviteResponse {
        hall_id: invite.hall_id.to_string(),
        user_id: user_id.to_string(),
    }))
}

/// DELETE /api/invites/:id
pub async fn revoke_invite(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<ErrorBody>)> {
    let _user_id = require_user(&state)?;
    let invite_id = Uuid::parse_str(&id).map_err(|_| err(StatusCode::BAD_REQUEST, "Invalid invite ID"))?;

    let db = state.db.lock().unwrap();
    db.revoke_invite(invite_id)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}
