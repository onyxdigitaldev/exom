//! Custom emoji storage

use rusqlite::{params, Connection};
use tracing::instrument;
use uuid::Uuid;

use super::parse::{parse_datetime, parse_uuid, OptionalExt};
use crate::error::Result;
use crate::models::CustomEmoji;

pub struct EmojiStore<'a> {
    conn: &'a Connection,
}

impl<'a> EmojiStore<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    #[instrument(skip(self, emoji))]
    pub fn create(&self, emoji: &CustomEmoji) -> Result<()> {
        self.conn.execute(
            "INSERT INTO custom_emoji (id, hall_id, name, image_hash, creator_id, animated, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                emoji.id.to_string(),
                emoji.hall_id.to_string(),
                emoji.name,
                emoji.image_hash,
                emoji.creator_id.to_string(),
                emoji.animated as i32,
                emoji.created_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    #[instrument(skip(self))]
    pub fn find_by_id(&self, id: Uuid) -> Result<Option<CustomEmoji>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, hall_id, name, image_hash, creator_id, animated, created_at
             FROM custom_emoji WHERE id = ?1",
        )?;

        let emoji = stmt
            .query_row(params![id.to_string()], |row| {
                Ok(CustomEmoji {
                    id: parse_uuid(&row.get::<_, String>(0)?)?,
                    hall_id: parse_uuid(&row.get::<_, String>(1)?)?,
                    name: row.get(2)?,
                    image_hash: row.get(3)?,
                    creator_id: parse_uuid(&row.get::<_, String>(4)?)?,
                    animated: row.get::<_, i32>(5)? != 0,
                    created_at: parse_datetime(&row.get::<_, String>(6)?)?,
                })
            })
            .optional()?;

        Ok(emoji)
    }

    #[instrument(skip(self))]
    pub fn find_by_name(&self, hall_id: Uuid, name: &str) -> Result<Option<CustomEmoji>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, hall_id, name, image_hash, creator_id, animated, created_at
             FROM custom_emoji WHERE hall_id = ?1 AND name = ?2",
        )?;

        let emoji = stmt
            .query_row(params![hall_id.to_string(), name], |row| {
                Ok(CustomEmoji {
                    id: parse_uuid(&row.get::<_, String>(0)?)?,
                    hall_id: parse_uuid(&row.get::<_, String>(1)?)?,
                    name: row.get(2)?,
                    image_hash: row.get(3)?,
                    creator_id: parse_uuid(&row.get::<_, String>(4)?)?,
                    animated: row.get::<_, i32>(5)? != 0,
                    created_at: parse_datetime(&row.get::<_, String>(6)?)?,
                })
            })
            .optional()?;

        Ok(emoji)
    }

    #[instrument(skip(self))]
    pub fn list_for_hall(&self, hall_id: Uuid) -> Result<Vec<CustomEmoji>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, hall_id, name, image_hash, creator_id, animated, created_at
             FROM custom_emoji WHERE hall_id = ?1 ORDER BY name",
        )?;

        let emojis = stmt
            .query_map(params![hall_id.to_string()], |row| {
                Ok(CustomEmoji {
                    id: parse_uuid(&row.get::<_, String>(0)?)?,
                    hall_id: parse_uuid(&row.get::<_, String>(1)?)?,
                    name: row.get(2)?,
                    image_hash: row.get(3)?,
                    creator_id: parse_uuid(&row.get::<_, String>(4)?)?,
                    animated: row.get::<_, i32>(5)? != 0,
                    created_at: parse_datetime(&row.get::<_, String>(6)?)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(emojis)
    }

    #[instrument(skip(self))]
    pub fn delete(&self, id: Uuid) -> Result<()> {
        self.conn.execute(
            "DELETE FROM custom_emoji WHERE id = ?1",
            params![id.to_string()],
        )?;
        Ok(())
    }

    #[instrument(skip(self))]
    pub fn count_for_hall(&self, hall_id: Uuid) -> Result<u64> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM custom_emoji WHERE hall_id = ?1",
            params![hall_id.to_string()],
            |row| row.get(0),
        )?;
        Ok(count as u64)
    }
}
