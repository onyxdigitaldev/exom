//! Event system — typing indicators, presence updates, and the event bus
//!
//! These are in-memory/ephemeral events, not persisted to the database.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::UserStatus;

/// A typing indicator event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypingEvent {
    pub user_id: Uuid,
    pub channel_id: Uuid,
    pub started_at: DateTime<Utc>,
}

impl TypingEvent {
    pub fn new(user_id: Uuid, channel_id: Uuid) -> Self {
        Self {
            user_id,
            channel_id,
            started_at: Utc::now(),
        }
    }

    /// Typing indicators expire after 10 seconds
    pub fn is_expired(&self) -> bool {
        Utc::now().signed_duration_since(self.started_at).num_seconds() > 10
    }
}

/// A user presence update event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresenceUpdate {
    pub user_id: Uuid,
    pub status: UserStatus,
    pub custom_status_text: Option<String>,
    pub custom_status_emoji: Option<String>,
    pub updated_at: DateTime<Utc>,
}

impl PresenceUpdate {
    pub fn new(user_id: Uuid, status: UserStatus) -> Self {
        Self {
            user_id,
            status,
            custom_status_text: None,
            custom_status_emoji: None,
            updated_at: Utc::now(),
        }
    }
}

/// All event types that can flow through the event bus
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExomEvent {
    // Typing
    TypingStarted(TypingEvent),

    // Presence
    PresenceUpdated(PresenceUpdate),

    // Messages
    MessageCreated {
        channel_id: Uuid,
        message_id: Uuid,
        sender_id: Uuid,
    },
    MessageUpdated {
        channel_id: Uuid,
        message_id: Uuid,
    },
    MessageDeleted {
        channel_id: Uuid,
        message_id: Uuid,
    },

    // Reactions
    ReactionAdded {
        channel_id: Uuid,
        message_id: Uuid,
        user_id: Uuid,
        emoji: String,
    },
    ReactionRemoved {
        channel_id: Uuid,
        message_id: Uuid,
        user_id: Uuid,
        emoji: String,
    },

    // Channels
    ChannelCreated {
        hall_id: Uuid,
        channel_id: Uuid,
    },
    ChannelUpdated {
        hall_id: Uuid,
        channel_id: Uuid,
    },
    ChannelDeleted {
        hall_id: Uuid,
        channel_id: Uuid,
    },

    // Members
    MemberJoined {
        hall_id: Uuid,
        user_id: Uuid,
    },
    MemberLeft {
        hall_id: Uuid,
        user_id: Uuid,
    },
    MemberUpdated {
        hall_id: Uuid,
        user_id: Uuid,
    },

    // Voice
    VoiceStateUpdated {
        hall_id: Uuid,
        channel_id: Uuid,
        user_id: Uuid,
    },

    // Host
    HostChanged {
        hall_id: Uuid,
        new_host_id: Option<Uuid>,
    },
}

/// Trait for an event bus that distributes events to subscribers
pub trait EventBus: Send + Sync {
    /// Emit an event to all subscribers
    fn emit(&self, event: ExomEvent);
}

/// A simple in-memory event bus using tokio broadcast channels
/// (placeholder — will be implemented when the app layer integrates)
pub struct LocalEventBus {
    sender: tokio::sync::broadcast::Sender<ExomEvent>,
}

impl LocalEventBus {
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = tokio::sync::broadcast::channel(capacity);
        Self { sender }
    }

    /// Subscribe to events
    pub fn subscribe(&self) -> tokio::sync::broadcast::Receiver<ExomEvent> {
        self.sender.subscribe()
    }
}

impl EventBus for LocalEventBus {
    fn emit(&self, event: ExomEvent) {
        // Ignore send errors (no subscribers)
        let _ = self.sender.send(event);
    }
}
