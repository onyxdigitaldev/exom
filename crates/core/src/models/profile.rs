//! User profile model — display name, avatar, bio, status

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// User presence status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum UserStatus {
    Offline = 0,
    Online = 1,
    Idle = 2,
    DoNotDisturb = 3,
    Invisible = 4,
}

impl UserStatus {
    pub fn display_name(&self) -> &'static str {
        match self {
            UserStatus::Offline => "Offline",
            UserStatus::Online => "Online",
            UserStatus::Idle => "Idle",
            UserStatus::DoNotDisturb => "Do Not Disturb",
            UserStatus::Invisible => "Invisible",
        }
    }

    /// Whether this status should show as "online" to others
    pub fn is_visible(&self) -> bool {
        matches!(self, UserStatus::Online | UserStatus::Idle | UserStatus::DoNotDisturb)
    }
}

impl std::fmt::Display for UserStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// Extended user profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub user_id: Uuid,
    pub display_name: Option<String>,
    pub avatar_hash: Option<String>,
    pub bio: Option<String>,
    pub status: UserStatus,
    pub custom_status_text: Option<String>,
    pub custom_status_emoji: Option<String>,
    pub updated_at: DateTime<Utc>,
}

impl UserProfile {
    pub fn new(user_id: Uuid) -> Self {
        Self {
            user_id,
            display_name: None,
            avatar_hash: None,
            bio: None,
            status: UserStatus::Online,
            custom_status_text: None,
            custom_status_emoji: None,
            updated_at: Utc::now(),
        }
    }

    pub fn with_display_name(mut self, name: String) -> Self {
        self.display_name = Some(name);
        self
    }

    pub fn with_bio(mut self, bio: String) -> Self {
        self.bio = Some(bio);
        self
    }

    pub fn with_status(mut self, status: UserStatus) -> Self {
        self.status = status;
        self
    }

    /// Get the name to display (display_name or fallback to username from User)
    pub fn effective_name<'a>(&'a self, username: &'a str) -> &'a str {
        self.display_name.as_deref().unwrap_or(username)
    }
}
