//! Notification models — read state, mentions, mute settings

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Mute level for a channel
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum MuteLevel {
    /// All notifications enabled
    All = 0,
    /// Only @mentions and DMs
    MentionsOnly = 1,
    /// No notifications
    Nothing = 2,
}

/// Type of mention
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum MentionType {
    /// @user mention
    User = 0,
    /// @role mention
    Role = 1,
    /// @everyone mention
    Everyone = 2,
    /// @here mention (only online users)
    Here = 3,
}

/// Read state for a user in a channel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadState {
    pub id: Uuid,
    pub user_id: Uuid,
    pub channel_id: Uuid,
    /// Last message the user has read
    pub last_read_message_id: Option<Uuid>,
    /// Number of unread mentions
    pub mention_count: u32,
}

impl ReadState {
    pub fn new(user_id: Uuid, channel_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            channel_id,
            last_read_message_id: None,
            mention_count: 0,
        }
    }
}

/// Per-channel notification settings for a user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationSettings {
    pub id: Uuid,
    pub user_id: Uuid,
    pub channel_id: Uuid,
    pub mute_level: MuteLevel,
    /// Temporary mute until this time
    pub mute_until: Option<DateTime<Utc>>,
}

impl NotificationSettings {
    pub fn new(user_id: Uuid, channel_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            channel_id,
            mute_level: MuteLevel::All,
            mute_until: None,
        }
    }

    /// Whether notifications are currently muted
    pub fn is_muted(&self) -> bool {
        if self.mute_level == MuteLevel::Nothing {
            return true;
        }
        if let Some(until) = self.mute_until {
            return Utc::now() < until;
        }
        false
    }
}

/// A mention of a user in a message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mention {
    pub id: Uuid,
    pub message_id: Uuid,
    pub user_id: Uuid,
    pub mention_type: MentionType,
}

impl Mention {
    pub fn new(message_id: Uuid, user_id: Uuid, mention_type: MentionType) -> Self {
        Self {
            id: Uuid::new_v4(),
            message_id,
            user_id,
            mention_type,
        }
    }
}
