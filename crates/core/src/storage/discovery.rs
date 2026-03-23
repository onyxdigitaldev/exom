//! Hall discovery and vanity invite storage

use rusqlite::{params, Connection};
use tracing::instrument;
use uuid::Uuid;

use super::parse::{parse_datetime, parse_uuid, OptionalExt};
use crate::error::Result;
use crate::models::{DiscoveryResult, HallDiscovery, VanityInvite};

pub struct DiscoveryStore<'a> {
    conn: &'a Connection,
}

impl<'a> DiscoveryStore<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    // --- Discovery ---

    #[instrument(skip(self, discovery))]
    pub fn upsert(&self, discovery: &HallDiscovery) -> Result<()> {
        self.conn.execute(
            "INSERT INTO hall_discovery (hall_id, description, category, tags, member_count_approx, is_listed, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(hall_id) DO UPDATE SET
                description = excluded.description,
                category = excluded.category,
                tags = excluded.tags,
                member_count_approx = excluded.member_count_approx,
                is_listed = excluded.is_listed,
                updated_at = excluded.updated_at",
            params![
                discovery.hall_id.to_string(),
                discovery.description,
                discovery.category,
                discovery.tags,
                discovery.member_count_approx,
                discovery.is_listed as i32,
                discovery.updated_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    #[instrument(skip(self))]
    pub fn find(&self, hall_id: Uuid) -> Result<Option<HallDiscovery>> {
        let mut stmt = self.conn.prepare(
            "SELECT hall_id, description, category, tags, member_count_approx, is_listed, updated_at
             FROM hall_discovery WHERE hall_id = ?1",
        )?;

        let discovery = stmt
            .query_row(params![hall_id.to_string()], |row| {
                Ok(HallDiscovery {
                    hall_id: parse_uuid(&row.get::<_, String>(0)?)?,
                    description: row.get(1)?,
                    category: row.get(2)?,
                    tags: row.get(3)?,
                    member_count_approx: row.get(4)?,
                    is_listed: row.get::<_, i32>(5)? != 0,
                    updated_at: parse_datetime(&row.get::<_, String>(6)?)?,
                })
            })
            .optional()?;

        Ok(discovery)
    }

    /// Search listed halls by name or description
    #[instrument(skip(self))]
    pub fn search(&self, query: &str, limit: u32) -> Result<Vec<DiscoveryResult>> {
        let pattern = format!("%{}%", query);
        let mut stmt = self.conn.prepare(
            "SELECT h.id, h.name, d.description, h.icon_hash, h.splash_hash, d.category, d.member_count_approx
             FROM hall_discovery d
             INNER JOIN halls h ON h.id = d.hall_id
             WHERE d.is_listed = 1 AND (h.name LIKE ?1 OR d.description LIKE ?1)
             ORDER BY d.member_count_approx DESC
             LIMIT ?2",
        )?;

        let results = stmt
            .query_map(params![pattern, limit], |row| {
                Ok(DiscoveryResult {
                    hall_id: parse_uuid(&row.get::<_, String>(0)?)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    icon_hash: row.get(3)?,
                    splash_hash: row.get(4)?,
                    category: row.get(5)?,
                    member_count_approx: row.get(6)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(results)
    }

    /// Browse by category
    #[instrument(skip(self))]
    pub fn browse_category(&self, category: &str, limit: u32) -> Result<Vec<DiscoveryResult>> {
        let mut stmt = self.conn.prepare(
            "SELECT h.id, h.name, d.description, h.icon_hash, h.splash_hash, d.category, d.member_count_approx
             FROM hall_discovery d
             INNER JOIN halls h ON h.id = d.hall_id
             WHERE d.is_listed = 1 AND d.category = ?1
             ORDER BY d.member_count_approx DESC
             LIMIT ?2",
        )?;

        let results = stmt
            .query_map(params![category, limit], |row| {
                Ok(DiscoveryResult {
                    hall_id: parse_uuid(&row.get::<_, String>(0)?)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    icon_hash: row.get(3)?,
                    splash_hash: row.get(4)?,
                    category: row.get(5)?,
                    member_count_approx: row.get(6)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(results)
    }

    #[instrument(skip(self))]
    pub fn delete(&self, hall_id: Uuid) -> Result<()> {
        self.conn.execute(
            "DELETE FROM hall_discovery WHERE hall_id = ?1",
            params![hall_id.to_string()],
        )?;
        Ok(())
    }

    // --- Vanity Invites ---

    #[instrument(skip(self, vanity))]
    pub fn create_vanity(&self, vanity: &VanityInvite) -> Result<()> {
        self.conn.execute(
            "INSERT INTO vanity_invites (hall_id, slug) VALUES (?1, ?2)",
            params![vanity.hall_id.to_string(), vanity.slug],
        )?;
        Ok(())
    }

    #[instrument(skip(self))]
    pub fn find_vanity_by_slug(&self, slug: &str) -> Result<Option<VanityInvite>> {
        let mut stmt = self.conn.prepare(
            "SELECT hall_id, slug FROM vanity_invites WHERE slug = ?1",
        )?;

        let vanity = stmt
            .query_row(params![slug], |row| {
                Ok(VanityInvite {
                    hall_id: parse_uuid(&row.get::<_, String>(0)?)?,
                    slug: row.get(1)?,
                })
            })
            .optional()?;

        Ok(vanity)
    }

    #[instrument(skip(self))]
    pub fn find_vanity_by_hall(&self, hall_id: Uuid) -> Result<Option<VanityInvite>> {
        let mut stmt = self.conn.prepare(
            "SELECT hall_id, slug FROM vanity_invites WHERE hall_id = ?1",
        )?;

        let vanity = stmt
            .query_row(params![hall_id.to_string()], |row| {
                Ok(VanityInvite {
                    hall_id: parse_uuid(&row.get::<_, String>(0)?)?,
                    slug: row.get(1)?,
                })
            })
            .optional()?;

        Ok(vanity)
    }

    #[instrument(skip(self))]
    pub fn update_vanity_slug(&self, hall_id: Uuid, new_slug: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE vanity_invites SET slug = ?1 WHERE hall_id = ?2",
            params![new_slug, hall_id.to_string()],
        )?;
        Ok(())
    }

    #[instrument(skip(self))]
    pub fn delete_vanity(&self, hall_id: Uuid) -> Result<()> {
        self.conn.execute(
            "DELETE FROM vanity_invites WHERE hall_id = ?1",
            params![hall_id.to_string()],
        )?;
        Ok(())
    }
}
