//! Permission override storage

use rusqlite::{params, Connection};
use tracing::instrument;
use uuid::Uuid;

use super::parse::{parse_uuid, OptionalExt};
use crate::error::Result;
use crate::models::{OverrideTargetType, PermissionOverride};

fn target_type_from_u8(v: u8) -> OverrideTargetType {
    match v {
        0 => OverrideTargetType::Role,
        1 => OverrideTargetType::Member,
        _ => OverrideTargetType::Role,
    }
}

pub struct PermissionOverrideStore<'a> {
    conn: &'a Connection,
}

impl<'a> PermissionOverrideStore<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    #[instrument(skip(self, perm_override))]
    pub fn upsert(&self, perm_override: &PermissionOverride) -> Result<()> {
        self.conn.execute(
            "INSERT INTO permission_overrides (id, channel_id, target_type, target_id, allow_bits, deny_bits)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(channel_id, target_type, target_id) DO UPDATE SET
                allow_bits = excluded.allow_bits,
                deny_bits = excluded.deny_bits",
            params![
                perm_override.id.to_string(),
                perm_override.channel_id.to_string(),
                perm_override.target_type as u8,
                perm_override.target_id,
                perm_override.allow_bits as i64,
                perm_override.deny_bits as i64,
            ],
        )?;
        Ok(())
    }

    #[instrument(skip(self))]
    pub fn find(
        &self,
        channel_id: Uuid,
        target_type: OverrideTargetType,
        target_id: &str,
    ) -> Result<Option<PermissionOverride>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, channel_id, target_type, target_id, allow_bits, deny_bits
             FROM permission_overrides WHERE channel_id = ?1 AND target_type = ?2 AND target_id = ?3",
        )?;

        let perm = stmt
            .query_row(
                params![channel_id.to_string(), target_type as u8, target_id],
                |row| {
                    Ok(PermissionOverride {
                        id: parse_uuid(&row.get::<_, String>(0)?)?,
                        channel_id: parse_uuid(&row.get::<_, String>(1)?)?,
                        target_type: target_type_from_u8(row.get::<_, u8>(2)?),
                        target_id: row.get(3)?,
                        allow_bits: row.get::<_, i64>(4)? as u64,
                        deny_bits: row.get::<_, i64>(5)? as u64,
                    })
                },
            )
            .optional()?;

        Ok(perm)
    }

    /// List all overrides for a channel
    #[instrument(skip(self))]
    pub fn list_for_channel(&self, channel_id: Uuid) -> Result<Vec<PermissionOverride>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, channel_id, target_type, target_id, allow_bits, deny_bits
             FROM permission_overrides WHERE channel_id = ?1",
        )?;

        let overrides = stmt
            .query_map(params![channel_id.to_string()], |row| {
                Ok(PermissionOverride {
                    id: parse_uuid(&row.get::<_, String>(0)?)?,
                    channel_id: parse_uuid(&row.get::<_, String>(1)?)?,
                    target_type: target_type_from_u8(row.get::<_, u8>(2)?),
                    target_id: row.get(3)?,
                    allow_bits: row.get::<_, i64>(4)? as u64,
                    deny_bits: row.get::<_, i64>(5)? as u64,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(overrides)
    }

    #[instrument(skip(self))]
    pub fn delete(
        &self,
        channel_id: Uuid,
        target_type: OverrideTargetType,
        target_id: &str,
    ) -> Result<()> {
        self.conn.execute(
            "DELETE FROM permission_overrides WHERE channel_id = ?1 AND target_type = ?2 AND target_id = ?3",
            params![channel_id.to_string(), target_type as u8, target_id],
        )?;
        Ok(())
    }

    /// Delete all overrides for a channel
    #[instrument(skip(self))]
    pub fn delete_all_for_channel(&self, channel_id: Uuid) -> Result<()> {
        self.conn.execute(
            "DELETE FROM permission_overrides WHERE channel_id = ?1",
            params![channel_id.to_string()],
        )?;
        Ok(())
    }
}
