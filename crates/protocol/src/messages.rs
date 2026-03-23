//! Protocol message types
//!
//! All messages flow through the relay as ClientMessage (client -> relay)
//! or ServerMessage (relay -> client). The relay inspects the envelope
//! (hall_id, channel_id) to route, but does NOT inspect payloads.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ──────────────────────────────────────────────
// Client -> Relay
// ──────────────────────────────────────────────

/// Messages sent from a client to the relay
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientMessage {
    /// Initial authentication handshake
    Authenticate(AuthPayload),

    /// Heartbeat / keepalive (client sends every 30s)
    Heartbeat,

    /// Subscribe to a Hall's events (join)
    HallJoin { hall_id: Uuid },

    /// Unsubscribe from a Hall's events (leave)
    HallLeave { hall_id: Uuid },

    /// Send a chat message to a channel
    ChannelMessage {
        hall_id: Uuid,
        channel_id: Uuid,
        message: MessagePayload,
    },

    /// Edit an existing message
    MessageEdit {
        hall_id: Uuid,
        channel_id: Uuid,
        message_id: Uuid,
        new_content: String,
    },

    /// Delete a message
    MessageDelete {
        hall_id: Uuid,
        channel_id: Uuid,
        message_id: Uuid,
    },

    /// Add a reaction to a message
    ReactionAdd {
        hall_id: Uuid,
        channel_id: Uuid,
        message_id: Uuid,
        emoji: String,
        is_custom: bool,
    },

    /// Remove a reaction from a message
    ReactionRemove {
        hall_id: Uuid,
        channel_id: Uuid,
        message_id: Uuid,
        emoji: String,
    },

    /// Typing indicator (ephemeral, not queued for offline)
    TypingStart {
        hall_id: Uuid,
        channel_id: Uuid,
    },

    /// Presence update
    PresenceUpdate {
        status: u8,
        custom_text: Option<String>,
    },

    /// Mark a channel as read
    MarkRead {
        hall_id: Uuid,
        channel_id: Uuid,
        message_id: Uuid,
    },

    /// Pin a message
    MessagePin {
        hall_id: Uuid,
        channel_id: Uuid,
        message_id: Uuid,
    },

    /// Unpin a message
    MessageUnpin {
        hall_id: Uuid,
        channel_id: Uuid,
        message_id: Uuid,
    },

    // --- Voice signaling ---

    /// Join a voice channel
    VoiceJoin {
        hall_id: Uuid,
        channel_id: Uuid,
    },

    /// Leave voice channel
    VoiceLeave,

    /// Update voice state (mute/deaf/video/stream toggles)
    VoiceStateUpdate {
        self_mute: bool,
        self_deaf: bool,
        video: bool,
        streaming: bool,
    },

    /// WebRTC signaling: SDP offer
    VoiceOffer {
        target_user_id: Uuid,
        sdp: String,
    },

    /// WebRTC signaling: SDP answer
    VoiceAnswer {
        target_user_id: Uuid,
        sdp: String,
    },

    /// WebRTC signaling: ICE candidate
    VoiceIceCandidate {
        target_user_id: Uuid,
        candidate: String,
    },

    // --- DMs ---

    /// Send a direct message
    DirectMessage {
        channel_id: Uuid,
        message: MessagePayload,
    },

    // --- Sync ---

    /// Request missed messages since a timestamp
    SyncRequest {
        hall_id: Uuid,
        since: DateTime<Utc>,
    },

    /// Request the offline message queue
    QueueDrain,
}

// ──────────────────────────────────────────────
// Relay -> Client
// ──────────────────────────────────────────────

/// Messages sent from the relay to a client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerMessage {
    /// Authentication result
    AuthResult {
        success: bool,
        user_id: Option<Uuid>,
        error: Option<String>,
    },

    /// Heartbeat acknowledgement
    HeartbeatAck {
        server_time: DateTime<Utc>,
    },

    /// Confirmed subscription to a Hall
    HallJoined {
        hall_id: Uuid,
        online_members: Vec<Uuid>,
    },

    /// Confirmed unsubscription from a Hall
    HallLeft { hall_id: Uuid },

    /// A new message in a channel (from another user)
    ChannelMessage {
        hall_id: Uuid,
        channel_id: Uuid,
        sender_id: Uuid,
        message: MessagePayload,
    },

    /// A message was edited
    MessageEdited {
        hall_id: Uuid,
        channel_id: Uuid,
        message_id: Uuid,
        new_content: String,
        edited_at: DateTime<Utc>,
    },

    /// A message was deleted
    MessageDeleted {
        hall_id: Uuid,
        channel_id: Uuid,
        message_id: Uuid,
    },

    /// A reaction was added
    ReactionAdded {
        hall_id: Uuid,
        channel_id: Uuid,
        message_id: Uuid,
        user_id: Uuid,
        emoji: String,
        is_custom: bool,
    },

    /// A reaction was removed
    ReactionRemoved {
        hall_id: Uuid,
        channel_id: Uuid,
        message_id: Uuid,
        user_id: Uuid,
        emoji: String,
    },

    /// Someone is typing (ephemeral)
    TypingStarted {
        hall_id: Uuid,
        channel_id: Uuid,
        user_id: Uuid,
    },

    /// User presence changed
    PresenceUpdated {
        user_id: Uuid,
        status: u8,
        custom_text: Option<String>,
    },

    /// A message was pinned
    MessagePinned {
        hall_id: Uuid,
        channel_id: Uuid,
        message_id: Uuid,
        pinned_by: Uuid,
    },

    /// A message was unpinned
    MessageUnpinned {
        hall_id: Uuid,
        channel_id: Uuid,
        message_id: Uuid,
    },

    /// Member came online in a Hall
    MemberOnline {
        hall_id: Uuid,
        user_id: Uuid,
    },

    /// Member went offline in a Hall
    MemberOffline {
        hall_id: Uuid,
        user_id: Uuid,
    },

    /// Member joined a Hall
    MemberJoined {
        hall_id: Uuid,
        user_id: Uuid,
        username: String,
        role: u8,
    },

    /// Member left a Hall
    MemberLeft {
        hall_id: Uuid,
        user_id: Uuid,
    },

    // --- Voice ---

    /// User joined a voice channel
    VoiceUserJoined {
        hall_id: Uuid,
        channel_id: Uuid,
        user_id: Uuid,
    },

    /// User left a voice channel
    VoiceUserLeft {
        hall_id: Uuid,
        channel_id: Uuid,
        user_id: Uuid,
    },

    /// Voice state updated
    VoiceStateUpdated {
        hall_id: Uuid,
        channel_id: Uuid,
        user_id: Uuid,
        self_mute: bool,
        self_deaf: bool,
        video: bool,
        streaming: bool,
    },

    /// WebRTC signaling: incoming SDP offer
    VoiceOffer {
        from_user_id: Uuid,
        sdp: String,
    },

    /// WebRTC signaling: incoming SDP answer
    VoiceAnswer {
        from_user_id: Uuid,
        sdp: String,
    },

    /// WebRTC signaling: incoming ICE candidate
    VoiceIceCandidate {
        from_user_id: Uuid,
        candidate: String,
    },

    // --- DMs ---

    /// Incoming direct message
    DirectMessage {
        channel_id: Uuid,
        sender_id: Uuid,
        message: MessagePayload,
    },

    // --- Sync ---

    /// Queued messages for offline delivery (batch)
    QueuedMessages {
        messages: Vec<QueuedMessage>,
    },

    /// Sync response with missed events
    SyncResponse {
        hall_id: Uuid,
        events: Vec<ServerMessage>,
    },

    // --- Errors ---

    /// Server-side error
    Error {
        code: ErrorCode,
        message: String,
        /// Original request context (if applicable)
        context: Option<String>,
    },
}

// ──────────────────────────────────────────────
// Shared payloads
// ──────────────────────────────────────────────

/// Authentication payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthPayload {
    /// User ID claiming to authenticate
    pub user_id: Uuid,
    /// HMAC-signed token: user_id + timestamp, signed with relay secret
    pub token: String,
    /// Timestamp the token was generated (for expiry checking)
    pub timestamp: DateTime<Utc>,
    /// Protocol version for compatibility checking
    pub protocol_version: u32,
}

/// Current protocol version
pub const PROTOCOL_VERSION: u32 = 1;

/// A message payload (content + metadata)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessagePayload {
    pub id: Uuid,
    pub content: String,
    pub reply_to: Option<Uuid>,
    pub thread_id: Option<Uuid>,
    pub timestamp: DateTime<Utc>,
    /// Attachment metadata (files are uploaded separately)
    pub attachments: Vec<AttachmentMeta>,
}

impl MessagePayload {
    pub fn new(content: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            content,
            reply_to: None,
            thread_id: None,
            timestamp: Utc::now(),
            attachments: Vec::new(),
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

/// Attachment metadata (included in message, actual file transferred separately)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttachmentMeta {
    pub id: Uuid,
    pub filename: String,
    pub size_bytes: u64,
    pub content_type: Option<String>,
    pub hash: String,
}

/// A queued message for offline delivery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueuedMessage {
    pub queued_at: DateTime<Utc>,
    pub event: ServerMessage,
}

/// Error codes from the relay
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorCode {
    /// Authentication failed
    AuthFailed,
    /// Token expired
    TokenExpired,
    /// Protocol version mismatch
    VersionMismatch,
    /// Not subscribed to the requested Hall
    NotInHall,
    /// Rate limited
    RateLimited,
    /// Message too large
    MessageTooLarge,
    /// Internal relay error
    InternalError,
    /// Unknown message type
    UnknownMessage,
}

impl ErrorCode {
    pub fn as_u16(&self) -> u16 {
        match self {
            ErrorCode::AuthFailed => 4001,
            ErrorCode::TokenExpired => 4002,
            ErrorCode::VersionMismatch => 4003,
            ErrorCode::NotInHall => 4004,
            ErrorCode::RateLimited => 4005,
            ErrorCode::MessageTooLarge => 4006,
            ErrorCode::InternalError => 5000,
            ErrorCode::UnknownMessage => 4007,
        }
    }
}
