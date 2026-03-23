//! Moderation storage — bans, audit log

use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use tracing::instrument;
use uuid::Uuid;

use super::parse::{parse_datetime, parse_datetime_opt, parse_uuid, parse_uuid_opt, OptionalExt};
use crate::error::Result;
use crate::models::{AuditAction, AuditLogEntry, AuditTargetType, Ban};

pub struct ModerationStore<'a> {
    conn: &'a Connection,
}

impl<'a> ModerationStore<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    // --- Bans ---

    #[instrument(skip(self, ban))]
    pub fn create_ban(&self, ban: &Ban) -> Result<()> {
        self.conn.execute(
            "INSERT INTO bans (id, hall_id, user_id, banned_by, reason, created_at, expires_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                ban.id.to_string(),
                ban.hall_id.to_string(),
                ban.user_id.to_string(),
                ban.banned_by.to_string(),
                ban.reason,
                ban.created_at.to_rfc3339(),
                ban.expires_at.map(|t| t.to_rfc3339()),
            ],
        )?;
        Ok(())
    }

    #[instrument(skip(self))]
    pub fn find_ban(&self, hall_id: Uuid, user_id: Uuid) -> Result<Option<Ban>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, hall_id, user_id, banned_by, reason, created_at, expires_at
             FROM bans WHERE hall_id = ?1 AND user_id = ?2",
        )?;

        let ban = stmt
            .query_row(
                params![hall_id.to_string(), user_id.to_string()],
                |row| {
                    Ok(Ban {
                        id: parse_uuid(&row.get::<_, String>(0)?)?,
                        hall_id: parse_uuid(&row.get::<_, String>(1)?)?,
                        user_id: parse_uuid(&row.get::<_, String>(2)?)?,
                        banned_by: parse_uuid(&row.get::<_, String>(3)?)?,
                        reason: row.get(4)?,
                        created_at: parse_datetime(&row.get::<_, String>(5)?)?,
                        expires_at: parse_datetime_opt(row.get::<_, Option<String>>(6)?)?,
                    })
                },
            )
            .optional()?;

        Ok(ban)
    }

    /// Check if a user is currently banned (active ban)
    #[instrument(skip(self))]
    pub fn is_banned(&self, hall_id: Uuid, user_id: Uuid) -> Result<bool> {
        let ban = self.find_ban(hall_id, user_id)?;
        Ok(ban.map(|b| b.is_active()).unwrap_or(false))
    }

    #[instrument(skip(self))]
    pub fn list_bans(&self, hall_id: Uuid) -> Result<Vec<Ban>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, hall_id, user_id, banned_by, reason, created_at, expires_at
             FROM bans WHERE hall_id = ?1 ORDER BY created_at DESC",
        )?;

        let bans = stmt
            .query_map(params![hall_id.to_string()], |row| {
                Ok(Ban {
                    id: parse_uuid(&row.get::<_, String>(0)?)?,
                    hall_id: parse_uuid(&row.get::<_, String>(1)?)?,
                    user_id: parse_uuid(&row.get::<_, String>(2)?)?,
                    banned_by: parse_uuid(&row.get::<_, String>(3)?)?,
                    reason: row.get(4)?,
                    created_at: parse_datetime(&row.get::<_, String>(5)?)?,
                    expires_at: parse_datetime_opt(row.get::<_, Option<String>>(6)?)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(bans)
    }

    /// Remove ban (unban)
    #[instrument(skip(self))]
    pub fn delete_ban(&self, hall_id: Uuid, user_id: Uuid) -> Result<()> {
        self.conn.execute(
            "DELETE FROM bans WHERE hall_id = ?1 AND user_id = ?2",
            params![hall_id.to_string(), user_id.to_string()],
        )?;
        Ok(())
    }

    /// Clean up expired bans
    #[instrument(skip(self))]
    pub fn cleanup_expired_bans(&self) -> Result<u64> {
        let count = self.conn.execute(
            "DELETE FROM bans WHERE expires_at IS NOT NULL AND expires_at < ?1",
            params![Utc::now().to_rfc3339()],
        )?;
        Ok(count as u64)
    }

    // --- Audit Log ---

    #[instrument(skip(self, entry))]
    pub fn create_audit_entry(&self, entry: &AuditLogEntry) -> Result<()> {
        self.conn.execute(
            "INSERT INTO audit_log (id, hall_id, actor_id, action_type, target_type, target_id, changes, reason, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                entry.id.to_string(),
                entry.hall_id.to_string(),
                entry.actor_id.to_string(),
                entry.action_type.as_str(),
                entry.target_type.as_ref().map(|t| t.as_str().to_string()),
                entry.target_id.map(|t| t.to_string()),
                entry.changes,
                entry.reason,
                entry.created_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    /// List audit log entries for a hall, with pagination
    #[instrument(skip(self))]
    pub fn list_audit_log(
        &self,
        hall_id: Uuid,
        limit: u32,
        before: Option<DateTime<Utc>>,
    ) -> Result<Vec<AuditLogEntry>> {
        let query = if before.is_some() {
            "SELECT id, hall_id, actor_id, action_type, target_type, target_id, changes, reason, created_at
             FROM audit_log WHERE hall_id = ?1 AND created_at < ?2
             ORDER BY created_at DESC LIMIT ?3"
        } else {
            "SELECT id, hall_id, actor_id, action_type, target_type, target_id, changes, reason, created_at
             FROM audit_log WHERE hall_id = ?1
             ORDER BY created_at DESC LIMIT ?2"
        };

        let mut stmt = self.conn.prepare(query)?;

        let entries: Vec<AuditLogEntry> = if let Some(before_time) = before {
            stmt.query_map(
                params![hall_id.to_string(), before_time.to_rfc3339(), limit],
                Self::map_audit_entry,
            )?
            .collect::<std::result::Result<Vec<_>, _>>()?
        } else {
            stmt.query_map(
                params![hall_id.to_string(), limit],
                Self::map_audit_entry,
            )?
            .collect::<std::result::Result<Vec<_>, _>>()?
        };

        Ok(entries)
    }

    /// List audit log entries by action type
    #[instrument(skip(self))]
    pub fn list_audit_by_action(
        &self,
        hall_id: Uuid,
        action_type: &AuditAction,
        limit: u32,
    ) -> Result<Vec<AuditLogEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, hall_id, actor_id, action_type, target_type, target_id, changes, reason, created_at
             FROM audit_log WHERE hall_id = ?1 AND action_type = ?2
             ORDER BY created_at DESC LIMIT ?3",
        )?;

        let entries = stmt
            .query_map(
                params![hall_id.to_string(), action_type.as_str(), limit],
                Self::map_audit_entry,
            )?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(entries)
    }

    fn map_audit_entry(row: &rusqlite::Row<'_>) -> rusqlite::Result<AuditLogEntry> {
        let action_str: String = row.get(3)?;
        let target_type_str: Option<String> = row.get(4)?;

        Ok(AuditLogEntry {
            id: parse_uuid(&row.get::<_, String>(0)?)?,
            hall_id: parse_uuid(&row.get::<_, String>(1)?)?,
            actor_id: parse_uuid(&row.get::<_, String>(2)?)?,
            action_type: AuditAction::from_str(&action_str).unwrap_or(AuditAction::HallUpdate),
            target_type: target_type_str.and_then(|s| AuditTargetType::from_str(&s)),
            target_id: parse_uuid_opt(row.get::<_, Option<String>>(5)?)?,
            changes: row.get(6)?,
            reason: row.get(7)?,
            created_at: parse_datetime(&row.get::<_, String>(8)?)?,
        })
    }
}
