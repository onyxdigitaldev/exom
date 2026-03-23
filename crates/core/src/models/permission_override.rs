//! Per-channel permission override model

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Target type for a permission override
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum OverrideTargetType {
    /// Override applies to a role
    Role = 0,
    /// Override applies to a specific member
    Member = 1,
}

/// A per-channel permission override
///
/// Permissions are stored as bitfields. Each bit corresponds to a HallAction.
/// `allow_bits` grants permissions, `deny_bits` revokes them.
/// The effective permission is: (base_from_role | allow_bits) & !deny_bits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionOverride {
    pub id: Uuid,
    pub channel_id: Uuid,
    pub target_type: OverrideTargetType,
    /// The role level (as u8) or user ID this override applies to
    pub target_id: String,
    /// Bitfield of permissions explicitly allowed
    pub allow_bits: u64,
    /// Bitfield of permissions explicitly denied
    pub deny_bits: u64,
}

impl PermissionOverride {
    pub fn new(channel_id: Uuid, target_type: OverrideTargetType, target_id: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            channel_id,
            target_type,
            target_id,
            allow_bits: 0,
            deny_bits: 0,
        }
    }

    /// Set a permission bit as allowed
    pub fn allow(mut self, bit: u64) -> Self {
        self.allow_bits |= bit;
        self.deny_bits &= !bit;
        self
    }

    /// Set a permission bit as denied
    pub fn deny(mut self, bit: u64) -> Self {
        self.deny_bits |= bit;
        self.allow_bits &= !bit;
        self
    }

    /// Reset a permission bit (inherit from role)
    pub fn reset(mut self, bit: u64) -> Self {
        self.allow_bits &= !bit;
        self.deny_bits &= !bit;
        self
    }

    /// Check if a specific permission is explicitly allowed
    pub fn is_allowed(&self, bit: u64) -> bool {
        self.allow_bits & bit != 0
    }

    /// Check if a specific permission is explicitly denied
    pub fn is_denied(&self, bit: u64) -> bool {
        self.deny_bits & bit != 0
    }
}

/// Permission bits for channel-level overrides
/// These map to HallAction values for bitfield storage
pub mod permission_bits {
    pub const VIEW_CHANNEL: u64 = 1 << 0;
    pub const SEND_MESSAGES: u64 = 1 << 1;
    pub const SEND_VOICE: u64 = 1 << 2;
    pub const MANAGE_MESSAGES: u64 = 1 << 3;
    pub const MANAGE_CHANNEL: u64 = 1 << 4;
    pub const ADD_REACTIONS: u64 = 1 << 5;
    pub const ATTACH_FILES: u64 = 1 << 6;
    pub const EMBED_LINKS: u64 = 1 << 7;
    pub const USE_CUSTOM_EMOJI: u64 = 1 << 8;
    pub const MENTION_EVERYONE: u64 = 1 << 9;
    pub const CREATE_THREADS: u64 = 1 << 10;
    pub const CONNECT_VOICE: u64 = 1 << 11;
    pub const SPEAK: u64 = 1 << 12;
    pub const STREAM: u64 = 1 << 13;
    pub const MUTE_MEMBERS: u64 = 1 << 14;
    pub const DEAFEN_MEMBERS: u64 = 1 << 15;
    pub const MOVE_MEMBERS: u64 = 1 << 16;
    pub const PRIORITY_SPEAKER: u64 = 1 << 17;
}
