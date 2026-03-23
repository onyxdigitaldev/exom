//! Direct message models — DM channels, group DMs, direct messages

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Type of DM channel
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum DmChannelType {
    /// 1-on-1 DM
    Direct = 0,
    /// Group DM (up to 10 participants)
    Group = 1,
}

/// A direct message channel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DmChannel {
    pub id: Uuid,
    pub channel_type: DmChannelType,
    /// Group DM name (None for 1-on-1)
    pub name: Option<String>,
    /// Group DM icon hash
    pub icon_hash: Option<String>,
    /// Group DM owner (who created it)
    pub owner_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub last_message_at: Option<DateTime<Utc>>,
}

impl DmChannel {
    /// Create a 1-on-1 DM channel
    pub fn new_direct() -> Self {
        Self {
            id: Uuid::new_v4(),
            channel_type: DmChannelType::Direct,
            name: None,
            icon_hash: None,
            owner_id: None,
            created_at: Utc::now(),
            last_message_at: None,
        }
    }

    /// Create a group DM channel
    pub fn new_group(owner_id: Uuid, name: Option<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            channel_type: DmChannelType::Group,
            name,
            icon_hash: None,
            owner_id: Some(owner_id),
            created_at: Utc::now(),
            last_message_at: None,
        }
    }
}

/// A participant in a DM channel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DmParticipant {
    pub id: Uuid,
    pub channel_id: Uuid,
    pub user_id: Uuid,
    pub joined_at: DateTime<Utc>,
}

impl DmParticipant {
    pub fn new(channel_id: Uuid, user_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            channel_id,
            user_id,
            joined_at: Utc::now(),
        }
    }
}

/// A direct message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectMessage {
    pub id: Uuid,
    pub channel_id: Uuid,
    pub sender_id: Uuid,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub edited_at: Option<DateTime<Utc>>,
    pub is_deleted: bool,
    pub reply_to: Option<Uuid>,
}

impl DirectMessage {
    pub fn new(channel_id: Uuid, sender_id: Uuid, content: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            channel_id,
            sender_id,
            content,
            created_at: Utc::now(),
            edited_at: None,
            is_deleted: false,
            reply_to: None,
        }
    }

    pub fn with_reply(mut self, reply_to: Uuid) -> Self {
        self.reply_to = Some(reply_to);
        self
    }
}

/// DM display info for the channel list
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DmChannelDisplay {
    pub id: Uuid,
    pub channel_type: DmChannelType,
    pub name: Option<String>,
    pub participants: Vec<DmParticipantInfo>,
    pub last_message_preview: Option<String>,
    pub last_message_at: Option<DateTime<Utc>>,
}

/// Participant info for display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DmParticipantInfo {
    pub user_id: Uuid,
    pub username: String,
    pub display_name: Option<String>,
    pub avatar_hash: Option<String>,
    pub is_online: bool,
}
