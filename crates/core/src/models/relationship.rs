//! Relationship model — friends, blocks

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Type of relationship between two users
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum RelationshipType {
    /// Mutual friends
    Friend = 1,
    /// User has blocked the target
    Blocked = 2,
    /// Outgoing friend request (from user to target)
    PendingOutgoing = 3,
    /// Incoming friend request (from target to user)
    PendingIncoming = 4,
}

impl RelationshipType {
    pub fn display_name(&self) -> &'static str {
        match self {
            RelationshipType::Friend => "Friend",
            RelationshipType::Blocked => "Blocked",
            RelationshipType::PendingOutgoing => "Request Sent",
            RelationshipType::PendingIncoming => "Request Received",
        }
    }
}

impl std::fmt::Display for RelationshipType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// A relationship between two users
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    pub id: Uuid,
    pub user_id: Uuid,
    pub target_id: Uuid,
    pub relationship_type: RelationshipType,
    pub created_at: DateTime<Utc>,
}

impl Relationship {
    pub fn new(user_id: Uuid, target_id: Uuid, relationship_type: RelationshipType) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            target_id,
            relationship_type,
            created_at: Utc::now(),
        }
    }
}

/// Friend info for display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipInfo {
    pub user_id: Uuid,
    pub username: String,
    pub display_name: Option<String>,
    pub avatar_hash: Option<String>,
    pub is_online: bool,
    pub relationship_type: RelationshipType,
    pub since: DateTime<Utc>,
}
