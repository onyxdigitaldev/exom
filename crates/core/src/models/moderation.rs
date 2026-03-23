//! Moderation models — bans, audit log

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A ban on a user in a Hall
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ban {
    pub id: Uuid,
    pub hall_id: Uuid,
    pub user_id: Uuid,
    pub banned_by: Uuid,
    pub reason: Option<String>,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

impl Ban {
    pub fn new(hall_id: Uuid, user_id: Uuid, banned_by: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            hall_id,
            user_id,
            banned_by,
            reason: None,
            created_at: Utc::now(),
            expires_at: None,
        }
    }

    pub fn with_reason(mut self, reason: String) -> Self {
        self.reason = Some(reason);
        self
    }

    pub fn with_expiry(mut self, hours: u64) -> Self {
        self.expires_at = Some(Utc::now() + chrono::Duration::hours(hours as i64));
        self
    }

    /// Whether this ban is still active
    pub fn is_active(&self) -> bool {
        match self.expires_at {
            Some(expires) => Utc::now() < expires,
            None => true, // permanent ban
        }
    }
}

/// Audit log action types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AuditAction {
    // Hall actions
    HallCreate,
    HallUpdate,
    HallDelete,
    // Channel actions
    ChannelCreate,
    ChannelUpdate,
    ChannelDelete,
    // Member actions
    MemberJoin,
    MemberLeave,
    MemberKick,
    MemberBan,
    MemberUnban,
    MemberRoleUpdate,
    // Message actions
    MessageDelete,
    MessagePin,
    MessageUnpin,
    // Invite actions
    InviteCreate,
    InviteDelete,
    InviteRevoke,
    // Role/permission actions
    PermissionOverrideCreate,
    PermissionOverrideUpdate,
    PermissionOverrideDelete,
    // Webhook actions
    WebhookCreate,
    WebhookUpdate,
    WebhookDelete,
    // Emoji actions
    EmojiCreate,
    EmojiDelete,
    // Host actions
    HostTransfer,
    HostElection,
}

impl AuditAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            AuditAction::HallCreate => "hall_create",
            AuditAction::HallUpdate => "hall_update",
            AuditAction::HallDelete => "hall_delete",
            AuditAction::ChannelCreate => "channel_create",
            AuditAction::ChannelUpdate => "channel_update",
            AuditAction::ChannelDelete => "channel_delete",
            AuditAction::MemberJoin => "member_join",
            AuditAction::MemberLeave => "member_leave",
            AuditAction::MemberKick => "member_kick",
            AuditAction::MemberBan => "member_ban",
            AuditAction::MemberUnban => "member_unban",
            AuditAction::MemberRoleUpdate => "member_role_update",
            AuditAction::MessageDelete => "message_delete",
            AuditAction::MessagePin => "message_pin",
            AuditAction::MessageUnpin => "message_unpin",
            AuditAction::InviteCreate => "invite_create",
            AuditAction::InviteDelete => "invite_delete",
            AuditAction::InviteRevoke => "invite_revoke",
            AuditAction::PermissionOverrideCreate => "perm_override_create",
            AuditAction::PermissionOverrideUpdate => "perm_override_update",
            AuditAction::PermissionOverrideDelete => "perm_override_delete",
            AuditAction::WebhookCreate => "webhook_create",
            AuditAction::WebhookUpdate => "webhook_update",
            AuditAction::WebhookDelete => "webhook_delete",
            AuditAction::EmojiCreate => "emoji_create",
            AuditAction::EmojiDelete => "emoji_delete",
            AuditAction::HostTransfer => "host_transfer",
            AuditAction::HostElection => "host_election",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        Some(match s {
            "hall_create" => AuditAction::HallCreate,
            "hall_update" => AuditAction::HallUpdate,
            "hall_delete" => AuditAction::HallDelete,
            "channel_create" => AuditAction::ChannelCreate,
            "channel_update" => AuditAction::ChannelUpdate,
            "channel_delete" => AuditAction::ChannelDelete,
            "member_join" => AuditAction::MemberJoin,
            "member_leave" => AuditAction::MemberLeave,
            "member_kick" => AuditAction::MemberKick,
            "member_ban" => AuditAction::MemberBan,
            "member_unban" => AuditAction::MemberUnban,
            "member_role_update" => AuditAction::MemberRoleUpdate,
            "message_delete" => AuditAction::MessageDelete,
            "message_pin" => AuditAction::MessagePin,
            "message_unpin" => AuditAction::MessageUnpin,
            "invite_create" => AuditAction::InviteCreate,
            "invite_delete" => AuditAction::InviteDelete,
            "invite_revoke" => AuditAction::InviteRevoke,
            "perm_override_create" => AuditAction::PermissionOverrideCreate,
            "perm_override_update" => AuditAction::PermissionOverrideUpdate,
            "perm_override_delete" => AuditAction::PermissionOverrideDelete,
            "webhook_create" => AuditAction::WebhookCreate,
            "webhook_update" => AuditAction::WebhookUpdate,
            "webhook_delete" => AuditAction::WebhookDelete,
            "emoji_create" => AuditAction::EmojiCreate,
            "emoji_delete" => AuditAction::EmojiDelete,
            "host_transfer" => AuditAction::HostTransfer,
            "host_election" => AuditAction::HostElection,
            _ => return None,
        })
    }
}

/// Target type for audit log entries
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AuditTargetType {
    Hall,
    Channel,
    User,
    Message,
    Invite,
    Webhook,
    Emoji,
    PermissionOverride,
}

impl AuditTargetType {
    pub fn as_str(&self) -> &'static str {
        match self {
            AuditTargetType::Hall => "hall",
            AuditTargetType::Channel => "channel",
            AuditTargetType::User => "user",
            AuditTargetType::Message => "message",
            AuditTargetType::Invite => "invite",
            AuditTargetType::Webhook => "webhook",
            AuditTargetType::Emoji => "emoji",
            AuditTargetType::PermissionOverride => "perm_override",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        Some(match s {
            "hall" => AuditTargetType::Hall,
            "channel" => AuditTargetType::Channel,
            "user" => AuditTargetType::User,
            "message" => AuditTargetType::Message,
            "invite" => AuditTargetType::Invite,
            "webhook" => AuditTargetType::Webhook,
            "emoji" => AuditTargetType::Emoji,
            "perm_override" => AuditTargetType::PermissionOverride,
            _ => return None,
        })
    }
}

/// An entry in the audit log
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    pub id: Uuid,
    pub hall_id: Uuid,
    pub actor_id: Uuid,
    pub action_type: AuditAction,
    pub target_type: Option<AuditTargetType>,
    pub target_id: Option<Uuid>,
    /// JSON blob of what changed (old/new values)
    pub changes: Option<String>,
    pub reason: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl AuditLogEntry {
    pub fn new(hall_id: Uuid, actor_id: Uuid, action_type: AuditAction) -> Self {
        Self {
            id: Uuid::new_v4(),
            hall_id,
            actor_id,
            action_type,
            target_type: None,
            target_id: None,
            changes: None,
            reason: None,
            created_at: Utc::now(),
        }
    }

    pub fn with_target(mut self, target_type: AuditTargetType, target_id: Uuid) -> Self {
        self.target_type = Some(target_type);
        self.target_id = Some(target_id);
        self
    }

    pub fn with_changes(mut self, changes: String) -> Self {
        self.changes = Some(changes);
        self
    }

    pub fn with_reason(mut self, reason: String) -> Self {
        self.reason = Some(reason);
        self
    }
}
