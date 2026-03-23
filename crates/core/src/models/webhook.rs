//! Webhook and bot models

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A webhook that can post messages to a channel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Webhook {
    pub id: Uuid,
    pub channel_id: Uuid,
    pub hall_id: Uuid,
    pub name: String,
    pub avatar_hash: Option<String>,
    /// Unique token for sending messages via this webhook
    pub token: String,
    pub creator_id: Uuid,
    pub created_at: DateTime<Utc>,
}

impl Webhook {
    pub fn new(channel_id: Uuid, hall_id: Uuid, name: String, token: String, creator_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            channel_id,
            hall_id,
            name,
            avatar_hash: None,
            token,
            creator_id,
            created_at: Utc::now(),
        }
    }
}
