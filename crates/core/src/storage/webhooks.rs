//! Webhook storage

use rusqlite::{params, Connection};
use tracing::instrument;
use uuid::Uuid;

use super::parse::{parse_datetime, parse_uuid, OptionalExt};
use crate::error::Result;
use crate::models::Webhook;

pub struct WebhookStore<'a> {
    conn: &'a Connection,
}

impl<'a> WebhookStore<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    #[instrument(skip(self, webhook))]
    pub fn create(&self, webhook: &Webhook) -> Result<()> {
        self.conn.execute(
            "INSERT INTO webhooks (id, channel_id, hall_id, name, avatar_hash, token, creator_id, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                webhook.id.to_string(),
                webhook.channel_id.to_string(),
                webhook.hall_id.to_string(),
                webhook.name,
                webhook.avatar_hash,
                webhook.token,
                webhook.creator_id.to_string(),
                webhook.created_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    #[instrument(skip(self))]
    pub fn find_by_id(&self, id: Uuid) -> Result<Option<Webhook>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, channel_id, hall_id, name, avatar_hash, token, creator_id, created_at
             FROM webhooks WHERE id = ?1",
        )?;

        let webhook = stmt
            .query_row(params![id.to_string()], |row| {
                Ok(Webhook {
                    id: parse_uuid(&row.get::<_, String>(0)?)?,
                    channel_id: parse_uuid(&row.get::<_, String>(1)?)?,
                    hall_id: parse_uuid(&row.get::<_, String>(2)?)?,
                    name: row.get(3)?,
                    avatar_hash: row.get(4)?,
                    token: row.get(5)?,
                    creator_id: parse_uuid(&row.get::<_, String>(6)?)?,
                    created_at: parse_datetime(&row.get::<_, String>(7)?)?,
                })
            })
            .optional()?;

        Ok(webhook)
    }

    #[instrument(skip(self))]
    pub fn find_by_token(&self, token: &str) -> Result<Option<Webhook>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, channel_id, hall_id, name, avatar_hash, token, creator_id, created_at
             FROM webhooks WHERE token = ?1",
        )?;

        let webhook = stmt
            .query_row(params![token], |row| {
                Ok(Webhook {
                    id: parse_uuid(&row.get::<_, String>(0)?)?,
                    channel_id: parse_uuid(&row.get::<_, String>(1)?)?,
                    hall_id: parse_uuid(&row.get::<_, String>(2)?)?,
                    name: row.get(3)?,
                    avatar_hash: row.get(4)?,
                    token: row.get(5)?,
                    creator_id: parse_uuid(&row.get::<_, String>(6)?)?,
                    created_at: parse_datetime(&row.get::<_, String>(7)?)?,
                })
            })
            .optional()?;

        Ok(webhook)
    }

    #[instrument(skip(self))]
    pub fn list_for_channel(&self, channel_id: Uuid) -> Result<Vec<Webhook>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, channel_id, hall_id, name, avatar_hash, token, creator_id, created_at
             FROM webhooks WHERE channel_id = ?1 ORDER BY created_at",
        )?;

        let webhooks = stmt
            .query_map(params![channel_id.to_string()], |row| {
                Ok(Webhook {
                    id: parse_uuid(&row.get::<_, String>(0)?)?,
                    channel_id: parse_uuid(&row.get::<_, String>(1)?)?,
                    hall_id: parse_uuid(&row.get::<_, String>(2)?)?,
                    name: row.get(3)?,
                    avatar_hash: row.get(4)?,
                    token: row.get(5)?,
                    creator_id: parse_uuid(&row.get::<_, String>(6)?)?,
                    created_at: parse_datetime(&row.get::<_, String>(7)?)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(webhooks)
    }

    #[instrument(skip(self))]
    pub fn list_for_hall(&self, hall_id: Uuid) -> Result<Vec<Webhook>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, channel_id, hall_id, name, avatar_hash, token, creator_id, created_at
             FROM webhooks WHERE hall_id = ?1 ORDER BY created_at",
        )?;

        let webhooks = stmt
            .query_map(params![hall_id.to_string()], |row| {
                Ok(Webhook {
                    id: parse_uuid(&row.get::<_, String>(0)?)?,
                    channel_id: parse_uuid(&row.get::<_, String>(1)?)?,
                    hall_id: parse_uuid(&row.get::<_, String>(2)?)?,
                    name: row.get(3)?,
                    avatar_hash: row.get(4)?,
                    token: row.get(5)?,
                    creator_id: parse_uuid(&row.get::<_, String>(6)?)?,
                    created_at: parse_datetime(&row.get::<_, String>(7)?)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(webhooks)
    }

    #[instrument(skip(self))]
    pub fn update_name(&self, id: Uuid, name: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE webhooks SET name = ?1 WHERE id = ?2",
            params![name, id.to_string()],
        )?;
        Ok(())
    }

    #[instrument(skip(self))]
    pub fn delete(&self, id: Uuid) -> Result<()> {
        self.conn.execute(
            "DELETE FROM webhooks WHERE id = ?1",
            params![id.to_string()],
        )?;
        Ok(())
    }
}
