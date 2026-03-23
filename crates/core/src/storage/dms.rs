//! DM storage operations

use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use tracing::instrument;
use uuid::Uuid;

use super::parse::{parse_datetime, parse_datetime_opt, parse_uuid, parse_uuid_opt, OptionalExt};
use crate::error::Result;
use crate::models::{DirectMessage, DmChannel, DmChannelType, DmParticipant};

fn dm_channel_type_from_u8(v: u8) -> DmChannelType {
    match v {
        0 => DmChannelType::Direct,
        1 => DmChannelType::Group,
        _ => DmChannelType::Direct,
    }
}

pub struct DmStore<'a> {
    conn: &'a Connection,
}

impl<'a> DmStore<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    // --- DM Channels ---

    #[instrument(skip(self, channel))]
    pub fn create_channel(&self, channel: &DmChannel) -> Result<()> {
        self.conn.execute(
            "INSERT INTO dm_channels (id, channel_type, name, icon_hash, owner_id, created_at, last_message_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                channel.id.to_string(),
                channel.channel_type as u8,
                channel.name,
                channel.icon_hash,
                channel.owner_id.map(|o| o.to_string()),
                channel.created_at.to_rfc3339(),
                channel.last_message_at.map(|t| t.to_rfc3339()),
            ],
        )?;
        Ok(())
    }

    #[instrument(skip(self))]
    pub fn find_channel(&self, id: Uuid) -> Result<Option<DmChannel>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, channel_type, name, icon_hash, owner_id, created_at, last_message_at
             FROM dm_channels WHERE id = ?1",
        )?;

        let channel = stmt
            .query_row(params![id.to_string()], |row| {
                Ok(DmChannel {
                    id: parse_uuid(&row.get::<_, String>(0)?)?,
                    channel_type: dm_channel_type_from_u8(row.get::<_, u8>(1)?),
                    name: row.get(2)?,
                    icon_hash: row.get(3)?,
                    owner_id: parse_uuid_opt(row.get::<_, Option<String>>(4)?)?,
                    created_at: parse_datetime(&row.get::<_, String>(5)?)?,
                    last_message_at: parse_datetime_opt(row.get::<_, Option<String>>(6)?)?,
                })
            })
            .optional()?;

        Ok(channel)
    }

    /// Find existing 1-on-1 DM channel between two users
    #[instrument(skip(self))]
    pub fn find_direct_channel(&self, user_a: Uuid, user_b: Uuid) -> Result<Option<DmChannel>> {
        let mut stmt = self.conn.prepare(
            "SELECT dc.id, dc.channel_type, dc.name, dc.icon_hash, dc.owner_id, dc.created_at, dc.last_message_at
             FROM dm_channels dc
             WHERE dc.channel_type = 0
               AND EXISTS (SELECT 1 FROM dm_participants WHERE channel_id = dc.id AND user_id = ?1)
               AND EXISTS (SELECT 1 FROM dm_participants WHERE channel_id = dc.id AND user_id = ?2)",
        )?;

        let channel = stmt
            .query_row(params![user_a.to_string(), user_b.to_string()], |row| {
                Ok(DmChannel {
                    id: parse_uuid(&row.get::<_, String>(0)?)?,
                    channel_type: dm_channel_type_from_u8(row.get::<_, u8>(1)?),
                    name: row.get(2)?,
                    icon_hash: row.get(3)?,
                    owner_id: parse_uuid_opt(row.get::<_, Option<String>>(4)?)?,
                    created_at: parse_datetime(&row.get::<_, String>(5)?)?,
                    last_message_at: parse_datetime_opt(row.get::<_, Option<String>>(6)?)?,
                })
            })
            .optional()?;

        Ok(channel)
    }

    /// List all DM channels for a user, ordered by most recent activity
    #[instrument(skip(self))]
    pub fn list_channels_for_user(&self, user_id: Uuid) -> Result<Vec<DmChannel>> {
        let mut stmt = self.conn.prepare(
            "SELECT dc.id, dc.channel_type, dc.name, dc.icon_hash, dc.owner_id, dc.created_at, dc.last_message_at
             FROM dm_channels dc
             INNER JOIN dm_participants dp ON dp.channel_id = dc.id
             WHERE dp.user_id = ?1
             ORDER BY COALESCE(dc.last_message_at, dc.created_at) DESC",
        )?;

        let channels = stmt
            .query_map(params![user_id.to_string()], |row| {
                Ok(DmChannel {
                    id: parse_uuid(&row.get::<_, String>(0)?)?,
                    channel_type: dm_channel_type_from_u8(row.get::<_, u8>(1)?),
                    name: row.get(2)?,
                    icon_hash: row.get(3)?,
                    owner_id: parse_uuid_opt(row.get::<_, Option<String>>(4)?)?,
                    created_at: parse_datetime(&row.get::<_, String>(5)?)?,
                    last_message_at: parse_datetime_opt(row.get::<_, Option<String>>(6)?)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(channels)
    }

    #[instrument(skip(self))]
    pub fn delete_channel(&self, id: Uuid) -> Result<()> {
        self.conn.execute(
            "DELETE FROM dm_channels WHERE id = ?1",
            params![id.to_string()],
        )?;
        Ok(())
    }

    // --- Participants ---

    #[instrument(skip(self, participant))]
    pub fn add_participant(&self, participant: &DmParticipant) -> Result<()> {
        self.conn.execute(
            "INSERT INTO dm_participants (id, channel_id, user_id, joined_at)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                participant.id.to_string(),
                participant.channel_id.to_string(),
                participant.user_id.to_string(),
                participant.joined_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    #[instrument(skip(self))]
    pub fn remove_participant(&self, channel_id: Uuid, user_id: Uuid) -> Result<()> {
        self.conn.execute(
            "DELETE FROM dm_participants WHERE channel_id = ?1 AND user_id = ?2",
            params![channel_id.to_string(), user_id.to_string()],
        )?;
        Ok(())
    }

    #[instrument(skip(self))]
    pub fn list_participants(&self, channel_id: Uuid) -> Result<Vec<DmParticipant>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, channel_id, user_id, joined_at
             FROM dm_participants WHERE channel_id = ?1",
        )?;

        let participants = stmt
            .query_map(params![channel_id.to_string()], |row| {
                Ok(DmParticipant {
                    id: parse_uuid(&row.get::<_, String>(0)?)?,
                    channel_id: parse_uuid(&row.get::<_, String>(1)?)?,
                    user_id: parse_uuid(&row.get::<_, String>(2)?)?,
                    joined_at: parse_datetime(&row.get::<_, String>(3)?)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(participants)
    }

    // --- Direct Messages ---

    #[instrument(skip(self, message))]
    pub fn create_message(&self, message: &DirectMessage) -> Result<()> {
        self.conn.execute(
            "INSERT INTO direct_messages (id, channel_id, sender_id, content, created_at, edited_at, is_deleted, reply_to)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                message.id.to_string(),
                message.channel_id.to_string(),
                message.sender_id.to_string(),
                message.content,
                message.created_at.to_rfc3339(),
                message.edited_at.map(|t| t.to_rfc3339()),
                message.is_deleted as i32,
                message.reply_to.map(|r| r.to_string()),
            ],
        )?;
        // Update last_message_at on the channel
        self.conn.execute(
            "UPDATE dm_channels SET last_message_at = ?1 WHERE id = ?2",
            params![message.created_at.to_rfc3339(), message.channel_id.to_string()],
        )?;
        Ok(())
    }

    #[instrument(skip(self))]
    pub fn find_message(&self, id: Uuid) -> Result<Option<DirectMessage>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, channel_id, sender_id, content, created_at, edited_at, is_deleted, reply_to
             FROM direct_messages WHERE id = ?1",
        )?;

        let message = stmt
            .query_row(params![id.to_string()], |row| {
                Ok(DirectMessage {
                    id: parse_uuid(&row.get::<_, String>(0)?)?,
                    channel_id: parse_uuid(&row.get::<_, String>(1)?)?,
                    sender_id: parse_uuid(&row.get::<_, String>(2)?)?,
                    content: row.get(3)?,
                    created_at: parse_datetime(&row.get::<_, String>(4)?)?,
                    edited_at: parse_datetime_opt(row.get::<_, Option<String>>(5)?)?,
                    is_deleted: row.get::<_, i32>(6)? != 0,
                    reply_to: parse_uuid_opt(row.get::<_, Option<String>>(7)?)?,
                })
            })
            .optional()?;

        Ok(message)
    }

    #[instrument(skip(self))]
    pub fn list_messages(
        &self,
        channel_id: Uuid,
        limit: u32,
        before: Option<DateTime<Utc>>,
    ) -> Result<Vec<DirectMessage>> {
        let query = if before.is_some() {
            "SELECT id, channel_id, sender_id, content, created_at, edited_at, is_deleted, reply_to
             FROM direct_messages
             WHERE channel_id = ?1 AND is_deleted = 0 AND created_at < ?2
             ORDER BY created_at DESC LIMIT ?3"
        } else {
            "SELECT id, channel_id, sender_id, content, created_at, edited_at, is_deleted, reply_to
             FROM direct_messages
             WHERE channel_id = ?1 AND is_deleted = 0
             ORDER BY created_at DESC LIMIT ?2"
        };

        let mut stmt = self.conn.prepare(query)?;

        let messages: Vec<DirectMessage> = if let Some(before_time) = before {
            stmt.query_map(
                params![channel_id.to_string(), before_time.to_rfc3339(), limit],
                Self::map_dm,
            )?
            .collect::<std::result::Result<Vec<_>, _>>()?
        } else {
            stmt.query_map(
                params![channel_id.to_string(), limit],
                Self::map_dm,
            )?
            .collect::<std::result::Result<Vec<_>, _>>()?
        };

        let mut messages = messages;
        messages.reverse();
        Ok(messages)
    }

    fn map_dm(row: &rusqlite::Row<'_>) -> rusqlite::Result<DirectMessage> {
        Ok(DirectMessage {
            id: parse_uuid(&row.get::<_, String>(0)?)?,
            channel_id: parse_uuid(&row.get::<_, String>(1)?)?,
            sender_id: parse_uuid(&row.get::<_, String>(2)?)?,
            content: row.get(3)?,
            created_at: parse_datetime(&row.get::<_, String>(4)?)?,
            edited_at: parse_datetime_opt(row.get::<_, Option<String>>(5)?)?,
            is_deleted: row.get::<_, i32>(6)? != 0,
            reply_to: parse_uuid_opt(row.get::<_, Option<String>>(7)?)?,
        })
    }

    #[instrument(skip(self, new_content))]
    pub fn update_message_content(&self, message_id: Uuid, new_content: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE direct_messages SET content = ?1, edited_at = ?2 WHERE id = ?3",
            params![new_content, Utc::now().to_rfc3339(), message_id.to_string()],
        )?;
        Ok(())
    }

    #[instrument(skip(self))]
    pub fn delete_message(&self, message_id: Uuid) -> Result<()> {
        self.conn.execute(
            "UPDATE direct_messages SET is_deleted = 1 WHERE id = ?1",
            params![message_id.to_string()],
        )?;
        Ok(())
    }
}
