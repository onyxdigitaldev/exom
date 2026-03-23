//! User profile storage operations

use rusqlite::{params, Connection};
use tracing::instrument;
use uuid::Uuid;

use super::parse::{parse_datetime, parse_uuid, OptionalExt};
use crate::error::Result;
use crate::models::{UserProfile, UserStatus};

fn status_from_u8(v: u8) -> UserStatus {
    match v {
        0 => UserStatus::Offline,
        1 => UserStatus::Online,
        2 => UserStatus::Idle,
        3 => UserStatus::DoNotDisturb,
        4 => UserStatus::Invisible,
        _ => UserStatus::Offline,
    }
}

pub struct ProfileStore<'a> {
    conn: &'a Connection,
}

impl<'a> ProfileStore<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    #[instrument(skip(self, profile), fields(user_id = %profile.user_id))]
    pub fn upsert(&self, profile: &UserProfile) -> Result<()> {
        self.conn.execute(
            "INSERT INTO user_profiles (user_id, display_name, avatar_hash, bio, status, custom_status_text, custom_status_emoji, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
             ON CONFLICT(user_id) DO UPDATE SET
                display_name = excluded.display_name,
                avatar_hash = excluded.avatar_hash,
                bio = excluded.bio,
                status = excluded.status,
                custom_status_text = excluded.custom_status_text,
                custom_status_emoji = excluded.custom_status_emoji,
                updated_at = excluded.updated_at",
            params![
                profile.user_id.to_string(),
                profile.display_name,
                profile.avatar_hash,
                profile.bio,
                profile.status as u8,
                profile.custom_status_text,
                profile.custom_status_emoji,
                profile.updated_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    #[instrument(skip(self))]
    pub fn find_by_user(&self, user_id: Uuid) -> Result<Option<UserProfile>> {
        let mut stmt = self.conn.prepare(
            "SELECT user_id, display_name, avatar_hash, bio, status, custom_status_text, custom_status_emoji, updated_at
             FROM user_profiles WHERE user_id = ?1",
        )?;

        let profile = stmt
            .query_row(params![user_id.to_string()], |row| {
                Ok(UserProfile {
                    user_id: parse_uuid(&row.get::<_, String>(0)?)?,
                    display_name: row.get(1)?,
                    avatar_hash: row.get(2)?,
                    bio: row.get(3)?,
                    status: status_from_u8(row.get::<_, u8>(4)?),
                    custom_status_text: row.get(5)?,
                    custom_status_emoji: row.get(6)?,
                    updated_at: parse_datetime(&row.get::<_, String>(7)?)?,
                })
            })
            .optional()?;

        Ok(profile)
    }

    #[instrument(skip(self))]
    pub fn update_status(&self, user_id: Uuid, status: UserStatus) -> Result<()> {
        self.conn.execute(
            "UPDATE user_profiles SET status = ?1, updated_at = ?2 WHERE user_id = ?3",
            params![
                status as u8,
                chrono::Utc::now().to_rfc3339(),
                user_id.to_string(),
            ],
        )?;
        Ok(())
    }

    #[instrument(skip(self))]
    pub fn delete(&self, user_id: Uuid) -> Result<()> {
        self.conn.execute(
            "DELETE FROM user_profiles WHERE user_id = ?1",
            params![user_id.to_string()],
        )?;
        Ok(())
    }
}
