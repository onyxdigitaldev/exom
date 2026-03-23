//! Custom emoji model

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A custom emoji belonging to a Hall
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomEmoji {
    pub id: Uuid,
    pub hall_id: Uuid,
    pub name: String,
    /// Hash of the emoji image file
    pub image_hash: String,
    pub creator_id: Uuid,
    pub animated: bool,
    pub created_at: DateTime<Utc>,
}

impl CustomEmoji {
    pub fn new(hall_id: Uuid, name: String, image_hash: String, creator_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            hall_id,
            name,
            image_hash,
            creator_id,
            animated: false,
            created_at: Utc::now(),
        }
    }

    pub fn with_animated(mut self, animated: bool) -> Self {
        self.animated = animated;
        self
    }

    /// Format for use in message text: :name:
    pub fn format_text(&self) -> String {
        format!(":{}:", self.name)
    }
}
