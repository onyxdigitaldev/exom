/// Authentication routes: login, register, logout, current user.
///
/// Passwords are hashed with Argon2. Sessions are 1-week expiry.
/// The sidecar stores auth state in AppState::current_user_id.

use std::sync::Arc;

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};

use exom_core::{Session, User, UserRepository};

use crate::state::AppState;

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub user_id: String,
    pub username: String,
    pub session_id: String,
}

#[derive(Serialize)]
pub struct UserResponse {
    pub user_id: String,
    pub username: String,
}

#[derive(Serialize)]
pub struct ErrorBody {
    pub error: String,
}

fn err(status: StatusCode, msg: &str) -> (StatusCode, Json<ErrorBody>) {
    (status, Json(ErrorBody { error: msg.to_string() }))
}

/// POST /api/auth/login
pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, Json<ErrorBody>)> {
    let db = state.db.lock().unwrap();

    let user = db
        .find_user_by_username(&req.username)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?
        .ok_or_else(|| err(StatusCode::UNAUTHORIZED, "Invalid credentials"))?;

    // Verify password
    let parsed_hash = PasswordHash::new(&user.password_hash)
        .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, "Password hash error"))?;

    Argon2::default()
        .verify_password(req.password.as_bytes(), &parsed_hash)
        .map_err(|_| err(StatusCode::UNAUTHORIZED, "Invalid credentials"))?;

    // Update last login
    let _ = db.update_last_login(user.id);

    // Create session (1 week)
    let session = Session::new(user.id, 168);
    db.create_session(&session)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    drop(db);
    state.set_current_user(Some(user.id));

    Ok(Json(AuthResponse {
        user_id: user.id.to_string(),
        username: user.username,
        session_id: session.id.to_string(),
    }))
}

/// POST /api/auth/register
pub async fn register(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, Json<ErrorBody>)> {
    if req.username.len() < 3 {
        return Err(err(StatusCode::BAD_REQUEST, "Username must be at least 3 characters"));
    }
    if req.password.len() < 6 {
        return Err(err(StatusCode::BAD_REQUEST, "Password must be at least 6 characters"));
    }

    let db = state.db.lock().unwrap();

    // Check username availability
    if db.find_user_by_username(&req.username)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?
        .is_some()
    {
        return Err(err(StatusCode::CONFLICT, "Username already taken"));
    }

    // Hash password
    let salt = SaltString::generate(&mut OsRng);
    let password_hash = Argon2::default()
        .hash_password(req.password.as_bytes(), &salt)
        .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, "Password hashing failed"))?
        .to_string();

    // Create user
    let user = User::new(req.username.clone(), password_hash);
    db.create_user(&user)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    // Auto-login: create session
    let session = Session::new(user.id, 168);
    db.create_session(&session)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    drop(db);
    state.set_current_user(Some(user.id));

    Ok(Json(AuthResponse {
        user_id: user.id.to_string(),
        username: req.username,
        session_id: session.id.to_string(),
    }))
}

/// POST /api/auth/logout
pub async fn logout(
    State(state): State<Arc<AppState>>,
) -> StatusCode {
    state.set_current_user(None);
    StatusCode::NO_CONTENT
}

/// GET /api/auth/me
pub async fn current_user(
    State(state): State<Arc<AppState>>,
) -> Result<Json<UserResponse>, (StatusCode, Json<ErrorBody>)> {
    let user_id = state
        .current_user()
        .ok_or_else(|| err(StatusCode::UNAUTHORIZED, "Not authenticated"))?;

    let db = state.db.lock().unwrap();
    let user = db
        .find_user_by_id(user_id)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?
        .ok_or_else(|| err(StatusCode::NOT_FOUND, "User not found"))?;

    Ok(Json(UserResponse {
        user_id: user.id.to_string(),
        username: user.username,
    }))
}
