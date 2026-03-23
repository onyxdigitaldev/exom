//! Channel model — text, voice, and category channels within a Hall

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Type of channel
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum ChannelType {
    /// Text chat channel
    Text = 0,
    /// Voice channel (future: WebRTC)
    Voice = 1,
    /// Category — groups other channels, not directly usable
    Category = 2,
    /// Announcement channel — only certain roles can post
    Announcement = 3,
    /// Stage channel — speaker/audience model
    Stage = 4,
}

impl ChannelType {
    pub fn display_name(&self) -> &'static str {
        match self {
            ChannelType::Text => "Text",
            ChannelType::Voice => "Voice",
            ChannelType::Category => "Category",
            ChannelType::Announcement => "Announcement",
            ChannelType::Stage => "Stage",
        }
    }

    /// Whether messages can be sent in this channel type
    pub fn supports_messages(&self) -> bool {
        matches!(self, ChannelType::Text | ChannelType::Announcement)
    }

    /// Whether this channel type supports voice connections
    pub fn supports_voice(&self) -> bool {
        matches!(self, ChannelType::Voice | ChannelType::Stage)
    }
}

impl std::fmt::Display for ChannelType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// A channel within a Hall
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Channel {
    pub id: Uuid,
    pub hall_id: Uuid,
    pub name: String,
    pub topic: Option<String>,
    pub channel_type: ChannelType,
    /// Parent category channel ID (None for top-level channels and categories themselves)
    pub parent_id: Option<Uuid>,
    /// Position within its category (or top-level if no parent)
    pub position: i32,
    /// Slowmode delay in seconds (0 = disabled)
    pub slowmode_seconds: u32,
    /// Whether the channel is NSFW
    pub nsfw: bool,
    pub created_at: DateTime<Utc>,
    /// Last message timestamp for sorting/unread tracking
    pub last_message_at: Option<DateTime<Utc>>,
}

impl Channel {
    pub fn new(hall_id: Uuid, name: String, channel_type: ChannelType) -> Self {
        Self {
            id: Uuid::new_v4(),
            hall_id,
            name,
            topic: None,
            channel_type,
            parent_id: None,
            position: 0,
            slowmode_seconds: 0,
            nsfw: false,
            created_at: Utc::now(),
            last_message_at: None,
        }
    }

    pub fn with_topic(mut self, topic: String) -> Self {
        self.topic = Some(topic);
        self
    }

    pub fn with_parent(mut self, parent_id: Uuid) -> Self {
        self.parent_id = Some(parent_id);
        self
    }

    pub fn with_position(mut self, position: i32) -> Self {
        self.position = position;
        self
    }

    pub fn with_slowmode(mut self, seconds: u32) -> Self {
        self.slowmode_seconds = seconds;
        self
    }
}

/// Channel info for display (includes unread state in future)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelInfo {
    pub id: Uuid,
    pub name: String,
    pub channel_type: ChannelType,
    pub topic: Option<String>,
    pub parent_id: Option<Uuid>,
    pub position: i32,
}
