//! Relationship storage operations

use rusqlite::{params, Connection};
use tracing::instrument;
use uuid::Uuid;

use super::parse::{parse_datetime, parse_uuid, OptionalExt};
use crate::error::Result;
use crate::models::{Relationship, RelationshipInfo, RelationshipType};

fn relationship_type_from_u8(v: u8) -> RelationshipType {
    match v {
        1 => RelationshipType::Friend,
        2 => RelationshipType::Blocked,
        3 => RelationshipType::PendingOutgoing,
        4 => RelationshipType::PendingIncoming,
        _ => RelationshipType::Friend,
    }
}

pub struct RelationshipStore<'a> {
    conn: &'a Connection,
}

impl<'a> RelationshipStore<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    #[instrument(skip(self, rel))]
    pub fn create(&self, rel: &Relationship) -> Result<()> {
        self.conn.execute(
            "INSERT INTO relationships (id, user_id, target_id, relationship_type, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                rel.id.to_string(),
                rel.user_id.to_string(),
                rel.target_id.to_string(),
                rel.relationship_type as u8,
                rel.created_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    #[instrument(skip(self))]
    pub fn find(&self, user_id: Uuid, target_id: Uuid) -> Result<Option<Relationship>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, user_id, target_id, relationship_type, created_at
             FROM relationships WHERE user_id = ?1 AND target_id = ?2",
        )?;

        let rel = stmt
            .query_row(
                params![user_id.to_string(), target_id.to_string()],
                |row| {
                    Ok(Relationship {
                        id: parse_uuid(&row.get::<_, String>(0)?)?,
                        user_id: parse_uuid(&row.get::<_, String>(1)?)?,
                        target_id: parse_uuid(&row.get::<_, String>(2)?)?,
                        relationship_type: relationship_type_from_u8(row.get::<_, u8>(3)?),
                        created_at: parse_datetime(&row.get::<_, String>(4)?)?,
                    })
                },
            )
            .optional()?;

        Ok(rel)
    }

    #[instrument(skip(self))]
    pub fn update_type(
        &self,
        user_id: Uuid,
        target_id: Uuid,
        new_type: RelationshipType,
    ) -> Result<()> {
        self.conn.execute(
            "UPDATE relationships SET relationship_type = ?1 WHERE user_id = ?2 AND target_id = ?3",
            params![
                new_type as u8,
                user_id.to_string(),
                target_id.to_string(),
            ],
        )?;
        Ok(())
    }

    #[instrument(skip(self))]
    pub fn delete(&self, user_id: Uuid, target_id: Uuid) -> Result<()> {
        self.conn.execute(
            "DELETE FROM relationships WHERE user_id = ?1 AND target_id = ?2",
            params![user_id.to_string(), target_id.to_string()],
        )?;
        Ok(())
    }

    /// List all relationships of a given type for a user, with user info
    #[instrument(skip(self))]
    pub fn list_by_type(
        &self,
        user_id: Uuid,
        rel_type: RelationshipType,
    ) -> Result<Vec<RelationshipInfo>> {
        let mut stmt = self.conn.prepare(
            "SELECT r.target_id, u.username, p.display_name, p.avatar_hash, p.status, r.relationship_type, r.created_at
             FROM relationships r
             INNER JOIN users u ON u.id = r.target_id
             LEFT JOIN user_profiles p ON p.user_id = r.target_id
             WHERE r.user_id = ?1 AND r.relationship_type = ?2
             ORDER BY u.username",
        )?;

        let rels = stmt
            .query_map(
                params![user_id.to_string(), rel_type as u8],
                |row| {
                    let status: u8 = row.get::<_, Option<u8>>(4)?.unwrap_or(0);
                    Ok(RelationshipInfo {
                        user_id: parse_uuid(&row.get::<_, String>(0)?)?,
                        username: row.get(1)?,
                        display_name: row.get(2)?,
                        avatar_hash: row.get(3)?,
                        is_online: status == 1 || status == 2 || status == 3,
                        relationship_type: relationship_type_from_u8(row.get::<_, u8>(5)?),
                        since: parse_datetime(&row.get::<_, String>(6)?)?,
                    })
                },
            )?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(rels)
    }

    /// List all relationships for a user (all types)
    #[instrument(skip(self))]
    pub fn list_all(&self, user_id: Uuid) -> Result<Vec<Relationship>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, user_id, target_id, relationship_type, created_at
             FROM relationships WHERE user_id = ?1
             ORDER BY created_at DESC",
        )?;

        let rels = stmt
            .query_map(params![user_id.to_string()], |row| {
                Ok(Relationship {
                    id: parse_uuid(&row.get::<_, String>(0)?)?,
                    user_id: parse_uuid(&row.get::<_, String>(1)?)?,
                    target_id: parse_uuid(&row.get::<_, String>(2)?)?,
                    relationship_type: relationship_type_from_u8(row.get::<_, u8>(3)?),
                    created_at: parse_datetime(&row.get::<_, String>(4)?)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(rels)
    }

    /// Check if two users are friends (mutual)
    #[instrument(skip(self))]
    pub fn are_friends(&self, user_a: Uuid, user_b: Uuid) -> Result<bool> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM relationships WHERE user_id = ?1 AND target_id = ?2 AND relationship_type = ?3",
            params![user_a.to_string(), user_b.to_string(), RelationshipType::Friend as u8],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    /// Check if user has blocked target
    #[instrument(skip(self))]
    pub fn is_blocked(&self, user_id: Uuid, target_id: Uuid) -> Result<bool> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM relationships WHERE user_id = ?1 AND target_id = ?2 AND relationship_type = ?3",
            params![user_id.to_string(), target_id.to_string(), RelationshipType::Blocked as u8],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }
}
