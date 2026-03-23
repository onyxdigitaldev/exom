//! Exom Core Library
//!
//! Core models, permissions, hosting logic, storage, and events for the Exom platform.

pub mod chest;
pub mod error;
pub mod events;
pub mod hosting;
pub mod models;
pub mod network;
pub mod permissions;
pub mod storage;

pub use chest::HallChest;
pub use error::{Error, Result};
pub use events::*;
pub use hosting::*;
pub use models::*;
pub use permissions::*;
pub use network::{RelayClient, RelayClientConfig, ConnectionState, generate_auth_token};
pub use storage::{
    AttachmentStore, ChannelRepository, ChannelStore, Database, DiscoveryStore, DmStore,
    EmojiStore, HallRepository, HallStore, InviteRepository, InviteStore, MessageRepository,
    MessageStore, ModerationStore, NotificationStore, PermissionOverrideStore, ProfileStore,
    ReactionStore, RelationshipStore, Storage, UserRepository, UserStore, VoiceStore, WebhookStore,
};
