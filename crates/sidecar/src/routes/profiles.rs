/// Profile routes: get profile, update own profile.
///
/// Profiles store display names, bios, avatars, and status information.
/// Users can only update their own profile via PUT /api/profiles/me.

use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use exom_core::{UserProfile, UserStatus};

use crate::state::AppState;
use super::auth::ErrorBody;

fn err(status: StatusCode, msg: &str) -> (StatusCode, Json<ErrorBody>) {
    (status, Json(ErrorBody { error: msg.to_string() }))
}

fn require_user(state: &AppState) -> Result<Uuid, (StatusCode, Json<ErrorBody>)> {
    state.current_user().ok_or_else(|| err(StatusCode::UNAUTHORIZED, "Not authenticated"))
}

fn parse_status(s: &str) -> Option<UserStatus> {
    match s.to_lowercase().as_str() {
        "online" => Some(UserStatus::Online),
        "offline" => Some(UserStatus::Offline),
        "idle" => Some(UserStatus::Idle),
        "dnd" | "do_not_disturb" => Some(UserStatus::DoNotDisturb),
        "invisible" => Some(UserStatus::Invisible),
        _ => None,
    }
}

#[derive(Serialize)]
pub struct ProfileResponse {
    pub user_id: String,
    pub display_name: Option<String>,
    pub avatar_hash: Option<String>,
    pub bio: Option<String>,
    pub status: String,
    pub custom_status_text: Option<String>,
    pub custom_status_emoji: Option<String>,
    pub updated_at: String,
}

impl From<UserProfile> for ProfileResponse {
    fn from(p: UserProfile) -> Self {
        Self {
            user_id: p.user_id.to_string(),
            display_name: p.display_name,
            avatar_hash: p.avatar_hash,
            bio: p.bio,
            status: p.status.display_name().to_string(),
            custom_status_text: p.custom_status_text,
            custom_status_emoji: p.custom_status_emoji,
            updated_at: p.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Deserialize)]
pub struct UpdateProfileRequest {
    pub display_name: Option<String>,
    pub bio: Option<String>,
    pub status: Option<String>,
}

/// GET /api/profiles/:user_id
pub async fn get_profile(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<String>,
) -> Result<Json<ProfileResponse>, (StatusCode, Json<ErrorBody>)> {
    let _caller = require_user(&state)?;
    let user_id = Uuid::parse_str(&user_id).map_err(|_| err(StatusCode::BAD_REQUEST, "Invalid user ID"))?;

    let db = state.db.lock().unwrap();
    let profile = db.profiles().find_by_user(user_id)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?
        .ok_or_else(|| err(StatusCode::NOT_FOUND, "Profile not found"))?;

    Ok(Json(ProfileResponse::from(profile)))
}

/// PUT /api/profiles/me
pub async fn update_profile(
    State(state): State<Arc<AppState>>,
    Json(req): Json<UpdateProfileRequest>,
) -> Result<Json<ProfileResponse>, (StatusCode, Json<ErrorBody>)> {
    let user_id = require_user(&state)?;

    let db = state.db.lock().unwrap();

    // Load existing profile or create a new one
    let mut profile = db.profiles().find_by_user(user_id)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?
        .unwrap_or_else(|| UserProfile::new(user_id));

    if let Some(name) = req.display_name {
        profile.display_name = Some(name);
    }

    if let Some(bio) = req.bio {
        profile.bio = Some(bio);
    }

    if let Some(status_str) = req.status {
        let status = parse_status(&status_str)
            .ok_or_else(|| err(StatusCode::BAD_REQUEST, "Invalid status value"))?;
        profile.status = status;
    }

    profile.updated_at = chrono::Utc::now();

    db.profiles().upsert(&profile)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    Ok(Json(ProfileResponse::from(profile)))
}
