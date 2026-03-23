//! Hall model - the core workspace unit

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::ParlorId;

/// A Hall is a shared workspace with members, roles, channels, and chat
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hall {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub owner_id: Uuid,
    pub created_at: DateTime<Utc>,
    /// Currently active parlor module (future plugin system)
    pub active_parlor: Option<ParlorId>,
    /// Current host user ID (for hosting state)
    pub current_host_id: Option<Uuid>,
    /// Election epoch to prevent split-host scenarios
    pub election_epoch: u64,
    /// Hall icon image hash
    pub icon_hash: Option<String>,
    /// Hall banner image hash
    pub banner_hash: Option<String>,
    /// Hall invite splash image hash
    pub splash_hash: Option<String>,
}

impl Hall {
    pub fn new(name: String, owner_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            description: None,
            owner_id,
            created_at: Utc::now(),
            active_parlor: None,
            current_host_id: None,
            election_epoch: 0,
            icon_hash: None,
            banner_hash: None,
            splash_hash: None,
        }
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }
}
