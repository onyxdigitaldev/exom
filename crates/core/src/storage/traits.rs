//! Storage repository traits
//!
//! These traits define the storage interface, allowing for different
//! implementations (SQLite, mock, future network backend).

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::error::Result;
use crate::models::{
    Channel, ChannelInfo, ChannelType, Hall, HallRole, Invite, MemberInfo, Membership, Message,
    MessageDisplay, Session, User,
};

/// User repository operations
pub trait UserRepository {
    fn create_user(&self, user: &User) -> Result<()>;
    fn find_user_by_id(&self, id: Uuid) -> Result<Option<User>>;
    fn find_user_by_username(&self, username: &str) -> Result<Option<User>>;
    fn update_last_login(&self, user_id: Uuid) -> Result<()>;
    fn create_session(&self, session: &Session) -> Result<()>;
    fn find_valid_session(&self, session_id: Uuid) -> Result<Option<Session>>;
    fn delete_session(&self, session_id: Uuid) -> Result<()>;
    fn delete_user_sessions(&self, user_id: Uuid) -> Result<()>;
    fn cleanup_expired_sessions(&self) -> Result<u64>;
}

/// Hall repository operations
pub trait HallRepository {
    fn create_hall(&self, hall: &Hall) -> Result<()>;
    fn find_hall_by_id(&self, id: Uuid) -> Result<Option<Hall>>;
    fn update_hall(&self, hall: &Hall) -> Result<()>;
    fn delete_hall(&self, hall_id: Uuid) -> Result<()>;
    fn list_halls_for_user(&self, user_id: Uuid) -> Result<Vec<Hall>>;
    fn add_member(&self, membership: &Membership) -> Result<()>;
    fn get_membership(&self, user_id: Uuid, hall_id: Uuid) -> Result<Option<Membership>>;
    fn update_role(&self, user_id: Uuid, hall_id: Uuid, new_role: HallRole) -> Result<()>;
    fn update_online_status(&self, user_id: Uuid, hall_id: Uuid, is_online: bool) -> Result<()>;
    fn remove_member(&self, user_id: Uuid, hall_id: Uuid) -> Result<()>;
    fn list_members(&self, hall_id: Uuid) -> Result<Vec<MemberInfo>>;
    fn get_user_role(&self, user_id: Uuid, hall_id: Uuid) -> Result<Option<HallRole>>;
}

/// Channel repository operations
pub trait ChannelRepository {
    fn create_channel(&self, channel: &Channel) -> Result<()>;
    fn find_channel_by_id(&self, id: Uuid) -> Result<Option<Channel>>;
    fn list_channels_for_hall(&self, hall_id: Uuid) -> Result<Vec<Channel>>;
    fn list_channels_by_type(&self, hall_id: Uuid, channel_type: ChannelType) -> Result<Vec<Channel>>;
    fn list_channel_children(&self, parent_id: Uuid) -> Result<Vec<Channel>>;
    fn update_channel(&self, channel: &Channel) -> Result<()>;
    fn delete_channel(&self, channel_id: Uuid) -> Result<()>;
    fn update_channel_positions(&self, positions: &[(Uuid, i32)]) -> Result<()>;
    fn move_channel_to_category(&self, channel_id: Uuid, new_parent_id: Option<Uuid>, new_position: i32) -> Result<()>;
    fn touch_channel_last_message(&self, channel_id: Uuid) -> Result<()>;
    fn count_channels_for_hall(&self, hall_id: Uuid) -> Result<u64>;
    fn list_channel_info_for_hall(&self, hall_id: Uuid) -> Result<Vec<ChannelInfo>>;
}

/// Message repository operations
pub trait MessageRepository {
    fn create_message(&self, message: &Message) -> Result<()>;
    fn find_message_by_id(&self, id: Uuid) -> Result<Option<Message>>;
    fn list_messages_for_channel(
        &self,
        channel_id: Uuid,
        limit: u32,
        before: Option<DateTime<Utc>>,
    ) -> Result<Vec<MessageDisplay>>;
    fn update_message_content(&self, message_id: Uuid, new_content: &str) -> Result<()>;
    fn delete_message(&self, message_id: Uuid) -> Result<()>;
    fn count_messages_for_channel(&self, channel_id: Uuid) -> Result<u64>;
    fn pin_message(&self, message_id: Uuid) -> Result<()>;
    fn unpin_message(&self, message_id: Uuid) -> Result<()>;
    fn list_pinned_messages(&self, channel_id: Uuid) -> Result<Vec<MessageDisplay>>;
    fn list_thread_messages(
        &self,
        thread_id: Uuid,
        limit: u32,
        before: Option<DateTime<Utc>>,
    ) -> Result<Vec<MessageDisplay>>;
    fn count_thread_replies(&self, thread_id: Uuid) -> Result<u64>;
}

/// Invite repository operations
pub trait InviteRepository {
    fn create_invite(&self, invite: &Invite) -> Result<()>;
    fn find_invite_by_token(&self, token: &str) -> Result<Option<Invite>>;
    fn list_invites_for_hall(&self, hall_id: Uuid) -> Result<Vec<Invite>>;
    fn increment_use_count(&self, invite_id: Uuid) -> Result<()>;
    fn revoke_invite(&self, invite_id: Uuid) -> Result<()>;
    fn delete_invite(&self, invite_id: Uuid) -> Result<()>;
}

/// Combined storage interface
pub trait Storage:
    UserRepository + HallRepository + ChannelRepository + MessageRepository + InviteRepository
{
}

impl<T> Storage for T where
    T: UserRepository + HallRepository + ChannelRepository + MessageRepository + InviteRepository
{
}
