//! Reaction storage operations

use rusqlite::{params, Connection};
use tracing::instrument;
use uuid::Uuid;

use super::parse::{parse_uuid, parse_uuid_opt};
use crate::error::Result;
use crate::models::{Reaction, ReactionSummary};

pub struct ReactionStore<'a> {
    conn: &'a Connection,
}

impl<'a> ReactionStore<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    #[instrument(skip(self, reaction))]
    pub fn create(&self, reaction: &Reaction) -> Result<()> {
        self.conn.execute(
            "INSERT INTO reactions (id, message_id, user_id, emoji, is_custom, custom_emoji_id, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                reaction.id.to_string(),
                reaction.message_id.to_string(),
                reaction.user_id.to_string(),
                reaction.emoji,
                reaction.is_custom as i32,
                reaction.custom_emoji_id.map(|e| e.to_string()),
                reaction.created_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    /// Remove a reaction (user unreacts)
    #[instrument(skip(self))]
    pub fn delete(&self, message_id: Uuid, user_id: Uuid, emoji: &str) -> Result<()> {
        self.conn.execute(
            "DELETE FROM reactions WHERE message_id = ?1 AND user_id = ?2 AND emoji = ?3",
            params![message_id.to_string(), user_id.to_string(), emoji],
        )?;
        Ok(())
    }

    /// Remove all reactions from a message
    #[instrument(skip(self))]
    pub fn delete_all_for_message(&self, message_id: Uuid) -> Result<()> {
        self.conn.execute(
            "DELETE FROM reactions WHERE message_id = ?1",
            params![message_id.to_string()],
        )?;
        Ok(())
    }

    /// Get reaction summaries for a message (grouped by emoji with counts)
    #[instrument(skip(self))]
    pub fn list_summaries(
        &self,
        message_id: Uuid,
        current_user_id: Uuid,
    ) -> Result<Vec<ReactionSummary>> {
        let mut stmt = self.conn.prepare(
            "SELECT emoji, is_custom, custom_emoji_id, COUNT(*) as cnt,
                    MAX(CASE WHEN user_id = ?2 THEN 1 ELSE 0 END) as me
             FROM reactions
             WHERE message_id = ?1
             GROUP BY emoji
             ORDER BY MIN(created_at)",
        )?;

        let summaries = stmt
            .query_map(
                params![message_id.to_string(), current_user_id.to_string()],
                |row| {
                    Ok(ReactionSummary {
                        emoji: row.get(0)?,
                        is_custom: row.get::<_, i32>(1)? != 0,
                        custom_emoji_id: parse_uuid_opt(row.get::<_, Option<String>>(2)?)?,
                        count: row.get(3)?,
                        me: row.get::<_, i32>(4)? != 0,
                    })
                },
            )?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(summaries)
    }

    /// List users who reacted with a specific emoji
    #[instrument(skip(self))]
    pub fn list_users_for_emoji(
        &self,
        message_id: Uuid,
        emoji: &str,
    ) -> Result<Vec<Uuid>> {
        let mut stmt = self.conn.prepare(
            "SELECT user_id FROM reactions WHERE message_id = ?1 AND emoji = ?2 ORDER BY created_at",
        )?;

        let users = stmt
            .query_map(params![message_id.to_string(), emoji], |row| {
                parse_uuid(&row.get::<_, String>(0)?)
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(users)
    }

    /// Count total reactions on a message
    #[instrument(skip(self))]
    pub fn count_for_message(&self, message_id: Uuid) -> Result<u32> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM reactions WHERE message_id = ?1",
            params![message_id.to_string()],
            |row| row.get(0),
        )?;
        Ok(count as u32)
    }
}
