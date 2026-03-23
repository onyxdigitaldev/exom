//! Hall discovery and vanity invite models

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Discovery listing for a Hall
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HallDiscovery {
    pub hall_id: Uuid,
    pub description: Option<String>,
    pub category: Option<String>,
    /// Comma-separated tags
    pub tags: Option<String>,
    pub member_count_approx: u32,
    pub is_listed: bool,
    pub updated_at: DateTime<Utc>,
}

impl HallDiscovery {
    pub fn new(hall_id: Uuid) -> Self {
        Self {
            hall_id,
            description: None,
            category: None,
            tags: None,
            member_count_approx: 0,
            is_listed: false,
            updated_at: Utc::now(),
        }
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    pub fn with_category(mut self, category: String) -> Self {
        self.category = Some(category);
        self
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = Some(tags.join(","));
        self
    }

    pub fn listed(mut self) -> Self {
        self.is_listed = true;
        self
    }

    /// Get tags as a vector
    pub fn tag_list(&self) -> Vec<&str> {
        self.tags
            .as_deref()
            .map(|t| t.split(',').filter(|s| !s.is_empty()).collect())
            .unwrap_or_default()
    }
}

/// A vanity invite slug for a Hall
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VanityInvite {
    pub hall_id: Uuid,
    pub slug: String,
}

impl VanityInvite {
    pub fn new(hall_id: Uuid, slug: String) -> Self {
        Self { hall_id, slug }
    }
}

/// Hall info for discovery search results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryResult {
    pub hall_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub icon_hash: Option<String>,
    pub splash_hash: Option<String>,
    pub category: Option<String>,
    pub member_count_approx: u32,
}
