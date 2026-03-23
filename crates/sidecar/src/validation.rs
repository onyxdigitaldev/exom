/// Input validation for all API endpoints.
///
/// Enforces length limits, character restrictions, and format
/// requirements on all user-submitted data. These constraints
/// are applied before any data touches the database.

use axum::{http::StatusCode, Json};

use crate::routes::auth::ErrorBody;

/// Maximum lengths for user-submitted strings.
pub mod limits {
    pub const USERNAME_MIN: usize = 3;
    pub const USERNAME_MAX: usize = 32;
    pub const PASSWORD_MIN: usize = 6;
    pub const PASSWORD_MAX: usize = 128;
    pub const HALL_NAME_MAX: usize = 100;
    pub const CHANNEL_NAME_MAX: usize = 100;
    pub const MESSAGE_MAX: usize = 4000;
    pub const BIO_MAX: usize = 190;
    pub const DISPLAY_NAME_MAX: usize = 32;
    pub const TOPIC_MAX: usize = 1024;
    pub const DESCRIPTION_MAX: usize = 2000;
    pub const REASON_MAX: usize = 512;
    pub const INVITE_TOKEN_LEN: usize = 16;
}

type ValidationResult = Result<(), (StatusCode, Json<ErrorBody>)>;

fn err(msg: &str) -> (StatusCode, Json<ErrorBody>) {
    (StatusCode::BAD_REQUEST, Json(ErrorBody { error: msg.to_string() }))
}

/// Validate a username meets requirements.
pub fn validate_username(username: &str) -> ValidationResult {
    let trimmed = username.trim();
    if trimmed.len() < limits::USERNAME_MIN {
        return Err(err(&format!("Username must be at least {} characters", limits::USERNAME_MIN)));
    }
    if trimmed.len() > limits::USERNAME_MAX {
        return Err(err(&format!("Username cannot exceed {} characters", limits::USERNAME_MAX)));
    }
    // Only allow alphanumeric, underscores, hyphens, periods
    if !trimmed.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-' || c == '.') {
        return Err(err("Username can only contain letters, numbers, underscores, hyphens, and periods"));
    }
    Ok(())
}

/// Validate a password meets requirements.
pub fn validate_password(password: &str) -> ValidationResult {
    if password.len() < limits::PASSWORD_MIN {
        return Err(err(&format!("Password must be at least {} characters", limits::PASSWORD_MIN)));
    }
    if password.len() > limits::PASSWORD_MAX {
        return Err(err(&format!("Password cannot exceed {} characters", limits::PASSWORD_MAX)));
    }
    Ok(())
}

/// Validate message content.
pub fn validate_message(content: &str) -> ValidationResult {
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return Err(err("Message cannot be empty"));
    }
    if trimmed.len() > limits::MESSAGE_MAX {
        return Err(err(&format!("Message cannot exceed {} characters", limits::MESSAGE_MAX)));
    }
    // Strip null bytes
    if content.contains('\0') {
        return Err(err("Message contains invalid characters"));
    }
    Ok(())
}

/// Validate a hall name.
pub fn validate_hall_name(name: &str) -> ValidationResult {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(err("Hall name cannot be empty"));
    }
    if trimmed.len() > limits::HALL_NAME_MAX {
        return Err(err(&format!("Hall name cannot exceed {} characters", limits::HALL_NAME_MAX)));
    }
    Ok(())
}

/// Validate a channel name.
pub fn validate_channel_name(name: &str) -> ValidationResult {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(err("Channel name cannot be empty"));
    }
    if trimmed.len() > limits::CHANNEL_NAME_MAX {
        return Err(err(&format!("Channel name cannot exceed {} characters", limits::CHANNEL_NAME_MAX)));
    }
    // Channel names: lowercase, alphanumeric, hyphens, underscores
    if !trimmed.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
        return Err(err("Channel name can only contain letters, numbers, hyphens, and underscores"));
    }
    Ok(())
}

/// Validate a UUID string format.
pub fn validate_uuid(id: &str) -> ValidationResult {
    if uuid::Uuid::parse_str(id).is_err() {
        return Err(err("Invalid ID format"));
    }
    Ok(())
}

/// Validate an optional text field with a max length.
pub fn validate_optional_text(text: &Option<String>, field: &str, max: usize) -> ValidationResult {
    if let Some(t) = text {
        if t.len() > max {
            return Err(err(&format!("{} cannot exceed {} characters", field, max)));
        }
        if t.contains('\0') {
            return Err(err(&format!("{} contains invalid characters", field)));
        }
    }
    Ok(())
}
