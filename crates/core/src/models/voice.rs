//! Voice state model — tracks who is in voice channels

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A user's state in a voice channel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceState {
    pub user_id: Uuid,
    pub channel_id: Uuid,
    pub hall_id: Uuid,
    /// User has muted themselves
    pub self_mute: bool,
    /// User has deafened themselves
    pub self_deaf: bool,
    /// Server has muted this user
    pub server_mute: bool,
    /// Server has deafened this user
    pub server_deaf: bool,
    /// User is screen sharing
    pub streaming: bool,
    /// User has camera on
    pub video: bool,
    pub connected_at: DateTime<Utc>,
}

impl VoiceState {
    pub fn new(user_id: Uuid, channel_id: Uuid, hall_id: Uuid) -> Self {
        Self {
            user_id,
            channel_id,
            hall_id,
            self_mute: false,
            self_deaf: false,
            server_mute: false,
            server_deaf: false,
            streaming: false,
            video: false,
            connected_at: Utc::now(),
        }
    }

    /// Whether the user can speak (not muted by self or server)
    pub fn can_speak(&self) -> bool {
        !self.self_mute && !self.server_mute
    }

    /// Whether the user can hear (not deafened by self or server)
    pub fn can_hear(&self) -> bool {
        !self.self_deaf && !self.server_deaf
    }
}

/// Voice channel info for display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceChannelInfo {
    pub channel_id: Uuid,
    pub channel_name: String,
    pub connected_users: Vec<VoiceUserInfo>,
}

/// Voice user info for display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceUserInfo {
    pub user_id: Uuid,
    pub username: String,
    pub self_mute: bool,
    pub self_deaf: bool,
    pub server_mute: bool,
    pub server_deaf: bool,
    pub streaming: bool,
    pub video: bool,
}
