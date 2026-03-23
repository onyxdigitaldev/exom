//! Attachment and embed storage operations

use rusqlite::{params, Connection};
use tracing::instrument;
use uuid::Uuid;

use super::parse::{parse_datetime, parse_uuid, OptionalExt};
use crate::error::Result;
use crate::models::{Attachment, Embed, EmbedType};

pub struct AttachmentStore<'a> {
    conn: &'a Connection,
}

impl<'a> AttachmentStore<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    #[instrument(skip(self, attachment))]
    pub fn create(&self, attachment: &Attachment) -> Result<()> {
        self.conn.execute(
            "INSERT INTO attachments (id, message_id, filename, size_bytes, content_type, hash, width, height, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                attachment.id.to_string(),
                attachment.message_id.to_string(),
                attachment.filename,
                attachment.size_bytes as i64,
                attachment.content_type,
                attachment.hash,
                attachment.width.map(|w| w as i32),
                attachment.height.map(|h| h as i32),
                attachment.created_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    #[instrument(skip(self))]
    pub fn find_by_id(&self, id: Uuid) -> Result<Option<Attachment>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, message_id, filename, size_bytes, content_type, hash, width, height, created_at
             FROM attachments WHERE id = ?1",
        )?;

        let attachment = stmt
            .query_row(params![id.to_string()], |row| {
                Ok(Attachment {
                    id: parse_uuid(&row.get::<_, String>(0)?)?,
                    message_id: parse_uuid(&row.get::<_, String>(1)?)?,
                    filename: row.get(2)?,
                    size_bytes: row.get::<_, i64>(3)? as u64,
                    content_type: row.get(4)?,
                    hash: row.get(5)?,
                    width: row.get::<_, Option<i32>>(6)?.map(|w| w as u32),
                    height: row.get::<_, Option<i32>>(7)?.map(|h| h as u32),
                    created_at: parse_datetime(&row.get::<_, String>(8)?)?,
                })
            })
            .optional()?;

        Ok(attachment)
    }

    #[instrument(skip(self))]
    pub fn list_for_message(&self, message_id: Uuid) -> Result<Vec<Attachment>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, message_id, filename, size_bytes, content_type, hash, width, height, created_at
             FROM attachments WHERE message_id = ?1 ORDER BY created_at",
        )?;

        let attachments = stmt
            .query_map(params![message_id.to_string()], |row| {
                Ok(Attachment {
                    id: parse_uuid(&row.get::<_, String>(0)?)?,
                    message_id: parse_uuid(&row.get::<_, String>(1)?)?,
                    filename: row.get(2)?,
                    size_bytes: row.get::<_, i64>(3)? as u64,
                    content_type: row.get(4)?,
                    hash: row.get(5)?,
                    width: row.get::<_, Option<i32>>(6)?.map(|w| w as u32),
                    height: row.get::<_, Option<i32>>(7)?.map(|h| h as u32),
                    created_at: parse_datetime(&row.get::<_, String>(8)?)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(attachments)
    }

    #[instrument(skip(self))]
    pub fn delete(&self, id: Uuid) -> Result<()> {
        self.conn.execute(
            "DELETE FROM attachments WHERE id = ?1",
            params![id.to_string()],
        )?;
        Ok(())
    }

    // --- Embeds ---

    #[instrument(skip(self, embed))]
    pub fn create_embed(&self, embed: &Embed) -> Result<()> {
        self.conn.execute(
            "INSERT INTO embeds (id, message_id, embed_type, title, description, url, color, thumbnail_url, image_url, author_name, author_url, footer_text, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            params![
                embed.id.to_string(),
                embed.message_id.to_string(),
                embed.embed_type.as_str(),
                embed.title,
                embed.description,
                embed.url,
                embed.color.map(|c| c as i64),
                embed.thumbnail_url,
                embed.image_url,
                embed.author_name,
                embed.author_url,
                embed.footer_text,
                embed.created_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    #[instrument(skip(self))]
    pub fn list_embeds_for_message(&self, message_id: Uuid) -> Result<Vec<Embed>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, message_id, embed_type, title, description, url, color, thumbnail_url, image_url, author_name, author_url, footer_text, created_at
             FROM embeds WHERE message_id = ?1 ORDER BY created_at",
        )?;

        let embeds = stmt
            .query_map(params![message_id.to_string()], |row| {
                Ok(Embed {
                    id: parse_uuid(&row.get::<_, String>(0)?)?,
                    message_id: parse_uuid(&row.get::<_, String>(1)?)?,
                    embed_type: EmbedType::from_str(&row.get::<_, String>(2)?),
                    title: row.get(3)?,
                    description: row.get(4)?,
                    url: row.get(5)?,
                    color: row.get::<_, Option<i64>>(6)?.map(|c| c as u32),
                    thumbnail_url: row.get(7)?,
                    image_url: row.get(8)?,
                    author_name: row.get(9)?,
                    author_url: row.get(10)?,
                    footer_text: row.get(11)?,
                    created_at: parse_datetime(&row.get::<_, String>(12)?)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(embeds)
    }

    #[instrument(skip(self))]
    pub fn delete_embeds_for_message(&self, message_id: Uuid) -> Result<()> {
        self.conn.execute(
            "DELETE FROM embeds WHERE message_id = ?1",
            params![message_id.to_string()],
        )?;
        Ok(())
    }
}
