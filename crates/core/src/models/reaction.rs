//! Reaction model — emoji reactions on messages

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A reaction on a message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reaction {
    pub id: Uuid,
    pub message_id: Uuid,
    pub user_id: Uuid,
    /// Unicode emoji string or custom emoji name
    pub emoji: String,
    /// Whether this is a custom emoji (vs unicode)
    pub is_custom: bool,
    /// If custom, the custom emoji ID
    pub custom_emoji_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

impl Reaction {
    /// Create a unicode emoji reaction
    pub fn new_unicode(message_id: Uuid, user_id: Uuid, emoji: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            message_id,
            user_id,
            emoji,
            is_custom: false,
            custom_emoji_id: None,
            created_at: Utc::now(),
        }
    }

    /// Create a custom emoji reaction
    pub fn new_custom(message_id: Uuid, user_id: Uuid, emoji_name: String, emoji_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            message_id,
            user_id,
            emoji: emoji_name,
            is_custom: true,
            custom_emoji_id: Some(emoji_id),
            created_at: Utc::now(),
        }
    }
}

/// Summary of reactions on a message for display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReactionSummary {
    pub emoji: String,
    pub is_custom: bool,
    pub custom_emoji_id: Option<Uuid>,
    pub count: u32,
    /// Whether the current user has reacted with this emoji
    pub me: bool,
}
