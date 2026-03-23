//! Message model for Hall chat

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::HallRole;

/// A chat message in a channel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: Uuid,
    pub channel_id: Uuid,
    pub hall_id: Uuid,
    pub sender_id: Uuid,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub edited_at: Option<DateTime<Utc>>,
    pub is_deleted: bool,
    /// If this message is a reply, the ID of the message being replied to
    pub reply_to: Option<Uuid>,
    /// If this message is part of a thread, the thread's parent message ID
    pub thread_id: Option<Uuid>,
    /// Whether this message is pinned
    pub is_pinned: bool,
}

impl Message {
    pub fn new(channel_id: Uuid, hall_id: Uuid, sender_id: Uuid, content: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            channel_id,
            hall_id,
            sender_id,
            content,
            created_at: Utc::now(),
            edited_at: None,
            is_deleted: false,
            reply_to: None,
            thread_id: None,
            is_pinned: false,
        }
    }

    pub fn with_reply(mut self, reply_to: Uuid) -> Self {
        self.reply_to = Some(reply_to);
        self
    }

    pub fn with_thread(mut self, thread_id: Uuid) -> Self {
        self.thread_id = Some(thread_id);
        self
    }
}

/// Message with sender information for display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageDisplay {
    pub id: Uuid,
    pub sender_id: Uuid,
    pub sender_username: String,
    pub sender_role: HallRole,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub is_edited: bool,
    pub reply_to: Option<Uuid>,
    pub thread_id: Option<Uuid>,
    pub is_pinned: bool,
    /// Number of reactions on this message (summary)
    pub reaction_count: u32,
    /// Number of replies in thread (if this is a thread starter)
    pub thread_reply_count: u32,
}

impl MessageDisplay {
    pub fn format_timestamp(&self) -> String {
        self.timestamp.format("%H:%M").to_string()
    }

    pub fn format_date(&self) -> String {
        self.timestamp.format("%Y-%m-%d").to_string()
    }
}
