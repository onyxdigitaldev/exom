//! Message storage operations

use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use tracing::instrument;
use uuid::Uuid;

use super::parse::{parse_datetime, parse_datetime_opt, parse_uuid, parse_uuid_opt, role_from_u8, OptionalExt};
use crate::error::Result;
use crate::models::{HallRole, Message, MessageDisplay};

pub struct MessageStore<'a> {
    conn: &'a Connection,
}

impl<'a> MessageStore<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    /// Create a new message
    #[instrument(skip(self, message), fields(channel_id = %message.channel_id, sender_id = %message.sender_id))]
    pub fn create(&self, message: &Message) -> Result<()> {
        self.conn.execute(
            "INSERT INTO messages (id, channel_id, hall_id, sender_id, content, created_at, edited_at, is_deleted, reply_to, thread_id, is_pinned)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                message.id.to_string(),
                message.channel_id.to_string(),
                message.hall_id.to_string(),
                message.sender_id.to_string(),
                message.content,
                message.created_at.to_rfc3339(),
                message.edited_at.map(|t| t.to_rfc3339()),
                message.is_deleted as i32,
                message.reply_to.map(|r| r.to_string()),
                message.thread_id.map(|t| t.to_string()),
                message.is_pinned as i32,
            ],
        )?;
        Ok(())
    }

    /// Get message by ID
    #[instrument(skip(self))]
    pub fn find_by_id(&self, id: Uuid) -> Result<Option<Message>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, channel_id, hall_id, sender_id, content, created_at, edited_at, is_deleted, reply_to, thread_id, is_pinned
             FROM messages WHERE id = ?1",
        )?;

        let message = stmt
            .query_row(params![id.to_string()], |row| {
                Ok(Message {
                    id: parse_uuid(&row.get::<_, String>(0)?)?,
                    channel_id: parse_uuid(&row.get::<_, String>(1)?)?,
                    hall_id: parse_uuid(&row.get::<_, String>(2)?)?,
                    sender_id: parse_uuid(&row.get::<_, String>(3)?)?,
                    content: row.get(4)?,
                    created_at: parse_datetime(&row.get::<_, String>(5)?)?,
                    edited_at: parse_datetime_opt(row.get::<_, Option<String>>(6)?)?,
                    is_deleted: row.get::<_, i32>(7)? != 0,
                    reply_to: parse_uuid_opt(row.get::<_, Option<String>>(8)?)?,
                    thread_id: parse_uuid_opt(row.get::<_, Option<String>>(9)?)?,
                    is_pinned: row.get::<_, i32>(10)? != 0,
                })
            })
            .optional()?;

        Ok(message)
    }

    /// List messages for a channel with display info
    #[instrument(skip(self))]
    pub fn list_for_channel(
        &self,
        channel_id: Uuid,
        limit: u32,
        before: Option<DateTime<Utc>>,
    ) -> Result<Vec<MessageDisplay>> {
        let query = if before.is_some() {
            "SELECT m.id, m.sender_id, u.username, mb.role, m.content, m.created_at, m.edited_at, m.reply_to, m.thread_id, m.is_pinned,
                    (SELECT COUNT(*) FROM messages t WHERE t.thread_id = m.id AND t.is_deleted = 0) as thread_count
             FROM messages m
             INNER JOIN users u ON u.id = m.sender_id
             LEFT JOIN memberships mb ON mb.user_id = m.sender_id AND mb.hall_id = m.hall_id
             WHERE m.channel_id = ?1 AND m.is_deleted = 0 AND m.thread_id IS NULL AND m.created_at < ?2
             ORDER BY m.created_at DESC
             LIMIT ?3"
        } else {
            "SELECT m.id, m.sender_id, u.username, mb.role, m.content, m.created_at, m.edited_at, m.reply_to, m.thread_id, m.is_pinned,
                    (SELECT COUNT(*) FROM messages t WHERE t.thread_id = m.id AND t.is_deleted = 0) as thread_count
             FROM messages m
             INNER JOIN users u ON u.id = m.sender_id
             LEFT JOIN memberships mb ON mb.user_id = m.sender_id AND mb.hall_id = m.hall_id
             WHERE m.channel_id = ?1 AND m.is_deleted = 0 AND m.thread_id IS NULL
             ORDER BY m.created_at DESC
             LIMIT ?2"
        };

        let mut stmt = self.conn.prepare(query)?;

        let messages: Vec<MessageDisplay> = if let Some(before_time) = before {
            stmt.query_map(
                params![channel_id.to_string(), before_time.to_rfc3339(), limit],
                Self::map_message_display,
            )?
            .collect::<std::result::Result<Vec<_>, _>>()?
        } else {
            stmt.query_map(
                params![channel_id.to_string(), limit],
                Self::map_message_display,
            )?
            .collect::<std::result::Result<Vec<_>, _>>()?
        };

        let mut messages = messages;
        messages.reverse();
        Ok(messages)
    }

    /// List thread messages (replies to a specific message)
    #[instrument(skip(self))]
    pub fn list_for_thread(
        &self,
        thread_id: Uuid,
        limit: u32,
        before: Option<DateTime<Utc>>,
    ) -> Result<Vec<MessageDisplay>> {
        let query = if before.is_some() {
            "SELECT m.id, m.sender_id, u.username, mb.role, m.content, m.created_at, m.edited_at, m.reply_to, m.thread_id, m.is_pinned, 0
             FROM messages m
             INNER JOIN users u ON u.id = m.sender_id
             LEFT JOIN memberships mb ON mb.user_id = m.sender_id AND mb.hall_id = m.hall_id
             WHERE m.thread_id = ?1 AND m.is_deleted = 0 AND m.created_at < ?2
             ORDER BY m.created_at DESC
             LIMIT ?3"
        } else {
            "SELECT m.id, m.sender_id, u.username, mb.role, m.content, m.created_at, m.edited_at, m.reply_to, m.thread_id, m.is_pinned, 0
             FROM messages m
             INNER JOIN users u ON u.id = m.sender_id
             LEFT JOIN memberships mb ON mb.user_id = m.sender_id AND mb.hall_id = m.hall_id
             WHERE m.thread_id = ?1 AND m.is_deleted = 0
             ORDER BY m.created_at DESC
             LIMIT ?2"
        };

        let mut stmt = self.conn.prepare(query)?;

        let messages: Vec<MessageDisplay> = if let Some(before_time) = before {
            stmt.query_map(
                params![thread_id.to_string(), before_time.to_rfc3339(), limit],
                Self::map_message_display,
            )?
            .collect::<std::result::Result<Vec<_>, _>>()?
        } else {
            stmt.query_map(
                params![thread_id.to_string(), limit],
                Self::map_message_display,
            )?
            .collect::<std::result::Result<Vec<_>, _>>()?
        };

        let mut messages = messages;
        messages.reverse();
        Ok(messages)
    }

    fn map_message_display(row: &rusqlite::Row<'_>) -> rusqlite::Result<MessageDisplay> {
        Ok(MessageDisplay {
            id: parse_uuid(&row.get::<_, String>(0)?)?,
            sender_id: parse_uuid(&row.get::<_, String>(1)?)?,
            sender_username: row.get(2)?,
            sender_role: row
                .get::<_, Option<u8>>(3)?
                .map(role_from_u8)
                .unwrap_or(HallRole::HallFellow),
            content: row.get(4)?,
            timestamp: parse_datetime(&row.get::<_, String>(5)?)?,
            is_edited: row.get::<_, Option<String>>(6)?.is_some(),
            reply_to: parse_uuid_opt(row.get::<_, Option<String>>(7)?)?,
            thread_id: parse_uuid_opt(row.get::<_, Option<String>>(8)?)?,
            is_pinned: row.get::<_, i32>(9)? != 0,
            reaction_count: 0, // filled by separate query when needed
            thread_reply_count: row.get::<_, u32>(10)?,
        })
    }

    /// Update message content
    #[instrument(skip(self, new_content))]
    pub fn update_content(&self, message_id: Uuid, new_content: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE messages SET content = ?1, edited_at = ?2 WHERE id = ?3",
            params![new_content, Utc::now().to_rfc3339(), message_id.to_string()],
        )?;
        Ok(())
    }

    /// Soft delete message
    #[instrument(skip(self))]
    pub fn delete(&self, message_id: Uuid) -> Result<()> {
        self.conn.execute(
            "UPDATE messages SET is_deleted = 1 WHERE id = ?1",
            params![message_id.to_string()],
        )?;
        Ok(())
    }

    /// Pin a message
    #[instrument(skip(self))]
    pub fn pin(&self, message_id: Uuid) -> Result<()> {
        self.conn.execute(
            "UPDATE messages SET is_pinned = 1 WHERE id = ?1",
            params![message_id.to_string()],
        )?;
        Ok(())
    }

    /// Unpin a message
    #[instrument(skip(self))]
    pub fn unpin(&self, message_id: Uuid) -> Result<()> {
        self.conn.execute(
            "UPDATE messages SET is_pinned = 0 WHERE id = ?1",
            params![message_id.to_string()],
        )?;
        Ok(())
    }

    /// List pinned messages in a channel
    #[instrument(skip(self))]
    pub fn list_pinned(&self, channel_id: Uuid) -> Result<Vec<MessageDisplay>> {
        let mut stmt = self.conn.prepare(
            "SELECT m.id, m.sender_id, u.username, mb.role, m.content, m.created_at, m.edited_at, m.reply_to, m.thread_id, m.is_pinned, 0
             FROM messages m
             INNER JOIN users u ON u.id = m.sender_id
             LEFT JOIN memberships mb ON mb.user_id = m.sender_id AND mb.hall_id = m.hall_id
             WHERE m.channel_id = ?1 AND m.is_pinned = 1 AND m.is_deleted = 0
             ORDER BY m.created_at DESC",
        )?;

        let messages = stmt
            .query_map(params![channel_id.to_string()], Self::map_message_display)?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(messages)
    }

    /// Get message count for a channel
    #[instrument(skip(self))]
    pub fn count_for_channel(&self, channel_id: Uuid) -> Result<u64> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM messages WHERE channel_id = ?1 AND is_deleted = 0",
            params![channel_id.to_string()],
            |row| row.get(0),
        )?;
        Ok(count as u64)
    }

    /// Count thread replies
    #[instrument(skip(self))]
    pub fn count_thread_replies(&self, thread_id: Uuid) -> Result<u64> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM messages WHERE thread_id = ?1 AND is_deleted = 0",
            params![thread_id.to_string()],
            |row| row.get(0),
        )?;
        Ok(count as u64)
    }
}
