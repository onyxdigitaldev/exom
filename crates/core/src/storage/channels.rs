//! Channel storage operations

use rusqlite::{params, Connection};
use tracing::instrument;
use uuid::Uuid;

use super::parse::{
    channel_type_from_u8, parse_datetime, parse_datetime_opt, parse_uuid, parse_uuid_opt,
    OptionalExt,
};
use crate::error::Result;
use crate::models::{Channel, ChannelInfo, ChannelType};

pub struct ChannelStore<'a> {
    conn: &'a Connection,
}

impl<'a> ChannelStore<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    /// Create a new channel
    #[instrument(skip(self, channel), fields(hall_id = %channel.hall_id, name = %channel.name))]
    pub fn create(&self, channel: &Channel) -> Result<()> {
        self.conn.execute(
            "INSERT INTO channels (id, hall_id, name, topic, channel_type, parent_id, position, slowmode_seconds, nsfw, created_at, last_message_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                channel.id.to_string(),
                channel.hall_id.to_string(),
                channel.name,
                channel.topic,
                channel.channel_type as u8,
                channel.parent_id.map(|p| p.to_string()),
                channel.position,
                channel.slowmode_seconds,
                channel.nsfw as i32,
                channel.created_at.to_rfc3339(),
                channel.last_message_at.map(|t| t.to_rfc3339()),
            ],
        )?;
        Ok(())
    }

    /// Find channel by ID
    #[instrument(skip(self))]
    pub fn find_by_id(&self, id: Uuid) -> Result<Option<Channel>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, hall_id, name, topic, channel_type, parent_id, position, slowmode_seconds, nsfw, created_at, last_message_at
             FROM channels WHERE id = ?1",
        )?;

        let channel = stmt
            .query_row(params![id.to_string()], |row| {
                Ok(Channel {
                    id: parse_uuid(&row.get::<_, String>(0)?)?,
                    hall_id: parse_uuid(&row.get::<_, String>(1)?)?,
                    name: row.get(2)?,
                    topic: row.get(3)?,
                    channel_type: channel_type_from_u8(row.get::<_, u8>(4)?),
                    parent_id: parse_uuid_opt(row.get::<_, Option<String>>(5)?)?,
                    position: row.get(6)?,
                    slowmode_seconds: row.get(7)?,
                    nsfw: row.get::<_, i32>(8)? != 0,
                    created_at: parse_datetime(&row.get::<_, String>(9)?)?,
                    last_message_at: parse_datetime_opt(row.get::<_, Option<String>>(10)?)?,
                })
            })
            .optional()?;

        Ok(channel)
    }

    /// List all channels in a Hall, ordered by position
    #[instrument(skip(self))]
    pub fn list_for_hall(&self, hall_id: Uuid) -> Result<Vec<Channel>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, hall_id, name, topic, channel_type, parent_id, position, slowmode_seconds, nsfw, created_at, last_message_at
             FROM channels WHERE hall_id = ?1
             ORDER BY position ASC, created_at ASC",
        )?;

        let channels = stmt
            .query_map(params![hall_id.to_string()], |row| {
                Ok(Channel {
                    id: parse_uuid(&row.get::<_, String>(0)?)?,
                    hall_id: parse_uuid(&row.get::<_, String>(1)?)?,
                    name: row.get(2)?,
                    topic: row.get(3)?,
                    channel_type: channel_type_from_u8(row.get::<_, u8>(4)?),
                    parent_id: parse_uuid_opt(row.get::<_, Option<String>>(5)?)?,
                    position: row.get(6)?,
                    slowmode_seconds: row.get(7)?,
                    nsfw: row.get::<_, i32>(8)? != 0,
                    created_at: parse_datetime(&row.get::<_, String>(9)?)?,
                    last_message_at: parse_datetime_opt(row.get::<_, Option<String>>(10)?)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(channels)
    }

    /// List channels by type within a Hall
    #[instrument(skip(self))]
    pub fn list_by_type(&self, hall_id: Uuid, channel_type: ChannelType) -> Result<Vec<Channel>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, hall_id, name, topic, channel_type, parent_id, position, slowmode_seconds, nsfw, created_at, last_message_at
             FROM channels WHERE hall_id = ?1 AND channel_type = ?2
             ORDER BY position ASC, created_at ASC",
        )?;

        let channels = stmt
            .query_map(
                params![hall_id.to_string(), channel_type as u8],
                |row| {
                    Ok(Channel {
                        id: parse_uuid(&row.get::<_, String>(0)?)?,
                        hall_id: parse_uuid(&row.get::<_, String>(1)?)?,
                        name: row.get(2)?,
                        topic: row.get(3)?,
                        channel_type: channel_type_from_u8(row.get::<_, u8>(4)?),
                        parent_id: parse_uuid_opt(row.get::<_, Option<String>>(5)?)?,
                        position: row.get(6)?,
                        slowmode_seconds: row.get(7)?,
                        nsfw: row.get::<_, i32>(8)? != 0,
                        created_at: parse_datetime(&row.get::<_, String>(9)?)?,
                        last_message_at: parse_datetime_opt(row.get::<_, Option<String>>(10)?)?,
                    })
                },
            )?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(channels)
    }

    /// List children of a category channel
    #[instrument(skip(self))]
    pub fn list_children(&self, parent_id: Uuid) -> Result<Vec<Channel>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, hall_id, name, topic, channel_type, parent_id, position, slowmode_seconds, nsfw, created_at, last_message_at
             FROM channels WHERE parent_id = ?1
             ORDER BY position ASC, created_at ASC",
        )?;

        let channels = stmt
            .query_map(params![parent_id.to_string()], |row| {
                Ok(Channel {
                    id: parse_uuid(&row.get::<_, String>(0)?)?,
                    hall_id: parse_uuid(&row.get::<_, String>(1)?)?,
                    name: row.get(2)?,
                    topic: row.get(3)?,
                    channel_type: channel_type_from_u8(row.get::<_, u8>(4)?),
                    parent_id: parse_uuid_opt(row.get::<_, Option<String>>(5)?)?,
                    position: row.get(6)?,
                    slowmode_seconds: row.get(7)?,
                    nsfw: row.get::<_, i32>(8)? != 0,
                    created_at: parse_datetime(&row.get::<_, String>(9)?)?,
                    last_message_at: parse_datetime_opt(row.get::<_, Option<String>>(10)?)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(channels)
    }

    /// Update channel
    #[instrument(skip(self, channel), fields(channel_id = %channel.id))]
    pub fn update(&self, channel: &Channel) -> Result<()> {
        self.conn.execute(
            "UPDATE channels SET name = ?1, topic = ?2, parent_id = ?3, position = ?4, slowmode_seconds = ?5, nsfw = ?6, last_message_at = ?7
             WHERE id = ?8",
            params![
                channel.name,
                channel.topic,
                channel.parent_id.map(|p| p.to_string()),
                channel.position,
                channel.slowmode_seconds,
                channel.nsfw as i32,
                channel.last_message_at.map(|t| t.to_rfc3339()),
                channel.id.to_string(),
            ],
        )?;
        Ok(())
    }

    /// Delete channel
    #[instrument(skip(self))]
    pub fn delete(&self, channel_id: Uuid) -> Result<()> {
        self.conn.execute(
            "DELETE FROM channels WHERE id = ?1",
            params![channel_id.to_string()],
        )?;
        Ok(())
    }

    /// Update channel positions (batch)
    #[instrument(skip(self, positions))]
    pub fn update_positions(&self, positions: &[(Uuid, i32)]) -> Result<()> {
        let mut stmt = self
            .conn
            .prepare("UPDATE channels SET position = ?1 WHERE id = ?2")?;
        for (id, position) in positions {
            stmt.execute(params![position, id.to_string()])?;
        }
        Ok(())
    }

    /// Move a channel to a different category
    #[instrument(skip(self))]
    pub fn move_to_category(
        &self,
        channel_id: Uuid,
        new_parent_id: Option<Uuid>,
        new_position: i32,
    ) -> Result<()> {
        self.conn.execute(
            "UPDATE channels SET parent_id = ?1, position = ?2 WHERE id = ?3",
            params![
                new_parent_id.map(|p| p.to_string()),
                new_position,
                channel_id.to_string(),
            ],
        )?;
        Ok(())
    }

    /// Update last_message_at timestamp
    #[instrument(skip(self))]
    pub fn touch_last_message(&self, channel_id: Uuid) -> Result<()> {
        self.conn.execute(
            "UPDATE channels SET last_message_at = ?1 WHERE id = ?2",
            params![
                chrono::Utc::now().to_rfc3339(),
                channel_id.to_string(),
            ],
        )?;
        Ok(())
    }

    /// Count channels in a Hall
    #[instrument(skip(self))]
    pub fn count_for_hall(&self, hall_id: Uuid) -> Result<u64> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM channels WHERE hall_id = ?1",
            params![hall_id.to_string()],
            |row| row.get(0),
        )?;
        Ok(count as u64)
    }

    /// Get channel info for display
    #[instrument(skip(self))]
    pub fn list_info_for_hall(&self, hall_id: Uuid) -> Result<Vec<ChannelInfo>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, channel_type, topic, parent_id, position
             FROM channels WHERE hall_id = ?1
             ORDER BY position ASC, created_at ASC",
        )?;

        let channels = stmt
            .query_map(params![hall_id.to_string()], |row| {
                Ok(ChannelInfo {
                    id: parse_uuid(&row.get::<_, String>(0)?)?,
                    name: row.get(1)?,
                    channel_type: channel_type_from_u8(row.get::<_, u8>(2)?),
                    topic: row.get(3)?,
                    parent_id: parse_uuid_opt(row.get::<_, Option<String>>(4)?)?,
                    position: row.get(5)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(channels)
    }
}
