//! SQLite storage layer for Exom

mod attachments;
mod channels;
mod discovery;
mod dms;
mod emoji;
mod halls;
mod invites;
mod messages;
mod migrations;
mod moderation;
mod notifications;
mod parse;
mod permission_overrides;
mod profiles;
mod reactions;
mod relationships;
mod traits;
mod users;
mod voice;
mod webhooks;

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::error::Result;
use crate::models::{
    Channel, ChannelInfo, ChannelType, Hall, HallRole, Invite, MemberInfo, Membership, Message,
    MessageDisplay, Session, User,
};
use rusqlite::Connection;
use std::path::Path;
use tracing::instrument;

pub use attachments::AttachmentStore;
pub use channels::ChannelStore;
pub use discovery::DiscoveryStore;
pub use dms::DmStore;
pub use emoji::EmojiStore;
pub use halls::HallStore;
pub use invites::InviteStore;
pub use messages::MessageStore;
pub use moderation::ModerationStore;
pub use notifications::NotificationStore;
pub use permission_overrides::PermissionOverrideStore;
pub use profiles::ProfileStore;
pub use reactions::ReactionStore;
pub use relationships::RelationshipStore;
pub use traits::{
    ChannelRepository, HallRepository, InviteRepository, MessageRepository, Storage,
    UserRepository,
};
pub use users::UserStore;
pub use voice::VoiceStore;
pub use webhooks::WebhookStore;

/// Main database handle
pub struct Database {
    conn: Connection,
}

impl Database {
    #[instrument(skip(path), fields(path = %path.as_ref().display()))]
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch("PRAGMA foreign_keys = ON")?;
        let db = Self { conn };
        db.init()?;
        Ok(db)
    }

    #[instrument]
    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch("PRAGMA foreign_keys = ON")?;
        let db = Self { conn };
        db.init()?;
        Ok(db)
    }

    fn init(&self) -> Result<()> {
        migrations::run_migrations(&self.conn)?;
        Ok(())
    }

    pub fn schema_version(&self) -> u32 {
        self.conn
            .query_row("SELECT MAX(version) FROM schema_migrations", [], |row| {
                row.get(0)
            })
            .unwrap_or(0)
    }

    // --- Store accessors ---

    pub fn users(&self) -> UserStore<'_> {
        UserStore::new(&self.conn)
    }

    pub fn halls(&self) -> HallStore<'_> {
        HallStore::new(&self.conn)
    }

    pub fn channels(&self) -> ChannelStore<'_> {
        ChannelStore::new(&self.conn)
    }

    pub fn messages(&self) -> MessageStore<'_> {
        MessageStore::new(&self.conn)
    }

    pub fn invites(&self) -> InviteStore<'_> {
        InviteStore::new(&self.conn)
    }

    pub fn profiles(&self) -> ProfileStore<'_> {
        ProfileStore::new(&self.conn)
    }

    pub fn dms(&self) -> DmStore<'_> {
        DmStore::new(&self.conn)
    }

    pub fn relationships(&self) -> RelationshipStore<'_> {
        RelationshipStore::new(&self.conn)
    }

    pub fn reactions(&self) -> ReactionStore<'_> {
        ReactionStore::new(&self.conn)
    }

    pub fn attachments(&self) -> AttachmentStore<'_> {
        AttachmentStore::new(&self.conn)
    }

    pub fn moderation(&self) -> ModerationStore<'_> {
        ModerationStore::new(&self.conn)
    }

    pub fn notifications(&self) -> NotificationStore<'_> {
        NotificationStore::new(&self.conn)
    }

    pub fn emoji(&self) -> EmojiStore<'_> {
        EmojiStore::new(&self.conn)
    }

    pub fn webhooks(&self) -> WebhookStore<'_> {
        WebhookStore::new(&self.conn)
    }

    pub fn voice(&self) -> VoiceStore<'_> {
        VoiceStore::new(&self.conn)
    }

    pub fn discovery(&self) -> DiscoveryStore<'_> {
        DiscoveryStore::new(&self.conn)
    }

    pub fn permission_overrides(&self) -> PermissionOverrideStore<'_> {
        PermissionOverrideStore::new(&self.conn)
    }
}

// --- Repository trait implementations for Database ---

impl UserRepository for Database {
    fn create_user(&self, user: &User) -> Result<()> {
        self.users().create(user)
    }
    fn find_user_by_id(&self, id: Uuid) -> Result<Option<User>> {
        self.users().find_by_id(id)
    }
    fn find_user_by_username(&self, username: &str) -> Result<Option<User>> {
        self.users().find_by_username(username)
    }
    fn update_last_login(&self, user_id: Uuid) -> Result<()> {
        self.users().update_last_login(user_id)
    }
    fn create_session(&self, session: &Session) -> Result<()> {
        self.users().create_session(session)
    }
    fn find_valid_session(&self, session_id: Uuid) -> Result<Option<Session>> {
        self.users().find_valid_session(session_id)
    }
    fn delete_session(&self, session_id: Uuid) -> Result<()> {
        self.users().delete_session(session_id)
    }
    fn delete_user_sessions(&self, user_id: Uuid) -> Result<()> {
        self.users().delete_user_sessions(user_id)
    }
    fn cleanup_expired_sessions(&self) -> Result<u64> {
        self.users().cleanup_expired_sessions()
    }
}

impl HallRepository for Database {
    fn create_hall(&self, hall: &Hall) -> Result<()> {
        self.halls().create(hall)
    }
    fn find_hall_by_id(&self, id: Uuid) -> Result<Option<Hall>> {
        self.halls().find_by_id(id)
    }
    fn update_hall(&self, hall: &Hall) -> Result<()> {
        self.halls().update(hall)
    }
    fn delete_hall(&self, hall_id: Uuid) -> Result<()> {
        self.halls().delete(hall_id)
    }
    fn list_halls_for_user(&self, user_id: Uuid) -> Result<Vec<Hall>> {
        self.halls().list_for_user(user_id)
    }
    fn add_member(&self, membership: &Membership) -> Result<()> {
        self.halls().add_member(membership)
    }
    fn get_membership(&self, user_id: Uuid, hall_id: Uuid) -> Result<Option<Membership>> {
        self.halls().get_membership(user_id, hall_id)
    }
    fn update_role(&self, user_id: Uuid, hall_id: Uuid, new_role: HallRole) -> Result<()> {
        self.halls().update_role(user_id, hall_id, new_role)
    }
    fn update_online_status(&self, user_id: Uuid, hall_id: Uuid, is_online: bool) -> Result<()> {
        self.halls().update_online_status(user_id, hall_id, is_online)
    }
    fn remove_member(&self, user_id: Uuid, hall_id: Uuid) -> Result<()> {
        self.halls().remove_member(user_id, hall_id)
    }
    fn list_members(&self, hall_id: Uuid) -> Result<Vec<MemberInfo>> {
        self.halls().list_members(hall_id)
    }
    fn get_user_role(&self, user_id: Uuid, hall_id: Uuid) -> Result<Option<HallRole>> {
        self.halls().get_user_role(user_id, hall_id)
    }
}

impl ChannelRepository for Database {
    fn create_channel(&self, channel: &Channel) -> Result<()> {
        self.channels().create(channel)
    }
    fn find_channel_by_id(&self, id: Uuid) -> Result<Option<Channel>> {
        self.channels().find_by_id(id)
    }
    fn list_channels_for_hall(&self, hall_id: Uuid) -> Result<Vec<Channel>> {
        self.channels().list_for_hall(hall_id)
    }
    fn list_channels_by_type(&self, hall_id: Uuid, channel_type: ChannelType) -> Result<Vec<Channel>> {
        self.channels().list_by_type(hall_id, channel_type)
    }
    fn list_channel_children(&self, parent_id: Uuid) -> Result<Vec<Channel>> {
        self.channels().list_children(parent_id)
    }
    fn update_channel(&self, channel: &Channel) -> Result<()> {
        self.channels().update(channel)
    }
    fn delete_channel(&self, channel_id: Uuid) -> Result<()> {
        self.channels().delete(channel_id)
    }
    fn update_channel_positions(&self, positions: &[(Uuid, i32)]) -> Result<()> {
        self.channels().update_positions(positions)
    }
    fn move_channel_to_category(&self, channel_id: Uuid, new_parent_id: Option<Uuid>, new_position: i32) -> Result<()> {
        self.channels().move_to_category(channel_id, new_parent_id, new_position)
    }
    fn touch_channel_last_message(&self, channel_id: Uuid) -> Result<()> {
        self.channels().touch_last_message(channel_id)
    }
    fn count_channels_for_hall(&self, hall_id: Uuid) -> Result<u64> {
        self.channels().count_for_hall(hall_id)
    }
    fn list_channel_info_for_hall(&self, hall_id: Uuid) -> Result<Vec<ChannelInfo>> {
        self.channels().list_info_for_hall(hall_id)
    }
}

impl MessageRepository for Database {
    fn create_message(&self, message: &Message) -> Result<()> {
        self.messages().create(message)
    }
    fn find_message_by_id(&self, id: Uuid) -> Result<Option<Message>> {
        self.messages().find_by_id(id)
    }
    fn list_messages_for_channel(
        &self,
        channel_id: Uuid,
        limit: u32,
        before: Option<DateTime<Utc>>,
    ) -> Result<Vec<MessageDisplay>> {
        self.messages().list_for_channel(channel_id, limit, before)
    }
    fn update_message_content(&self, message_id: Uuid, new_content: &str) -> Result<()> {
        self.messages().update_content(message_id, new_content)
    }
    fn delete_message(&self, message_id: Uuid) -> Result<()> {
        self.messages().delete(message_id)
    }
    fn count_messages_for_channel(&self, channel_id: Uuid) -> Result<u64> {
        self.messages().count_for_channel(channel_id)
    }
    fn pin_message(&self, message_id: Uuid) -> Result<()> {
        self.messages().pin(message_id)
    }
    fn unpin_message(&self, message_id: Uuid) -> Result<()> {
        self.messages().unpin(message_id)
    }
    fn list_pinned_messages(&self, channel_id: Uuid) -> Result<Vec<MessageDisplay>> {
        self.messages().list_pinned(channel_id)
    }
    fn list_thread_messages(
        &self,
        thread_id: Uuid,
        limit: u32,
        before: Option<DateTime<Utc>>,
    ) -> Result<Vec<MessageDisplay>> {
        self.messages().list_for_thread(thread_id, limit, before)
    }
    fn count_thread_replies(&self, thread_id: Uuid) -> Result<u64> {
        self.messages().count_thread_replies(thread_id)
    }
}

impl InviteRepository for Database {
    fn create_invite(&self, invite: &Invite) -> Result<()> {
        self.invites().create(invite)
    }
    fn find_invite_by_token(&self, token: &str) -> Result<Option<Invite>> {
        self.invites().find_by_token(token)
    }
    fn list_invites_for_hall(&self, hall_id: Uuid) -> Result<Vec<Invite>> {
        self.invites().list_for_hall(hall_id)
    }
    fn increment_use_count(&self, invite_id: Uuid) -> Result<()> {
        self.invites().increment_use_count(invite_id)
    }
    fn revoke_invite(&self, invite_id: Uuid) -> Result<()> {
        self.invites().revoke(invite_id)
    }
    fn delete_invite(&self, invite_id: Uuid) -> Result<()> {
        self.invites().delete(invite_id)
    }
}
