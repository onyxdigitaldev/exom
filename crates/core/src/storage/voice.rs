//! Voice state storage

use rusqlite::{params, Connection};
use tracing::instrument;
use uuid::Uuid;

use super::parse::{parse_datetime, parse_uuid, OptionalExt};
use crate::error::Result;
use crate::models::VoiceState;

pub struct VoiceStore<'a> {
    conn: &'a Connection,
}

impl<'a> VoiceStore<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    /// User joins a voice channel (upsert — moves them if already in another)
    #[instrument(skip(self, state))]
    pub fn upsert(&self, state: &VoiceState) -> Result<()> {
        // Remove from any existing voice channel first
        self.disconnect_user(state.user_id)?;

        self.conn.execute(
            "INSERT INTO voice_states (user_id, channel_id, hall_id, self_mute, self_deaf, server_mute, server_deaf, streaming, video, connected_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                state.user_id.to_string(),
                state.channel_id.to_string(),
                state.hall_id.to_string(),
                state.self_mute as i32,
                state.self_deaf as i32,
                state.server_mute as i32,
                state.server_deaf as i32,
                state.streaming as i32,
                state.video as i32,
                state.connected_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    /// Get a user's current voice state
    #[instrument(skip(self))]
    pub fn find_for_user(&self, user_id: Uuid) -> Result<Option<VoiceState>> {
        let mut stmt = self.conn.prepare(
            "SELECT user_id, channel_id, hall_id, self_mute, self_deaf, server_mute, server_deaf, streaming, video, connected_at
             FROM voice_states WHERE user_id = ?1",
        )?;

        let state = stmt
            .query_row(params![user_id.to_string()], |row| {
                Ok(VoiceState {
                    user_id: parse_uuid(&row.get::<_, String>(0)?)?,
                    channel_id: parse_uuid(&row.get::<_, String>(1)?)?,
                    hall_id: parse_uuid(&row.get::<_, String>(2)?)?,
                    self_mute: row.get::<_, i32>(3)? != 0,
                    self_deaf: row.get::<_, i32>(4)? != 0,
                    server_mute: row.get::<_, i32>(5)? != 0,
                    server_deaf: row.get::<_, i32>(6)? != 0,
                    streaming: row.get::<_, i32>(7)? != 0,
                    video: row.get::<_, i32>(8)? != 0,
                    connected_at: parse_datetime(&row.get::<_, String>(9)?)?,
                })
            })
            .optional()?;

        Ok(state)
    }

    /// List all users in a voice channel
    #[instrument(skip(self))]
    pub fn list_for_channel(&self, channel_id: Uuid) -> Result<Vec<VoiceState>> {
        let mut stmt = self.conn.prepare(
            "SELECT user_id, channel_id, hall_id, self_mute, self_deaf, server_mute, server_deaf, streaming, video, connected_at
             FROM voice_states WHERE channel_id = ?1 ORDER BY connected_at",
        )?;

        let states = stmt
            .query_map(params![channel_id.to_string()], |row| {
                Ok(VoiceState {
                    user_id: parse_uuid(&row.get::<_, String>(0)?)?,
                    channel_id: parse_uuid(&row.get::<_, String>(1)?)?,
                    hall_id: parse_uuid(&row.get::<_, String>(2)?)?,
                    self_mute: row.get::<_, i32>(3)? != 0,
                    self_deaf: row.get::<_, i32>(4)? != 0,
                    server_mute: row.get::<_, i32>(5)? != 0,
                    server_deaf: row.get::<_, i32>(6)? != 0,
                    streaming: row.get::<_, i32>(7)? != 0,
                    video: row.get::<_, i32>(8)? != 0,
                    connected_at: parse_datetime(&row.get::<_, String>(9)?)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(states)
    }

    /// List all voice states in a hall
    #[instrument(skip(self))]
    pub fn list_for_hall(&self, hall_id: Uuid) -> Result<Vec<VoiceState>> {
        let mut stmt = self.conn.prepare(
            "SELECT user_id, channel_id, hall_id, self_mute, self_deaf, server_mute, server_deaf, streaming, video, connected_at
             FROM voice_states WHERE hall_id = ?1 ORDER BY connected_at",
        )?;

        let states = stmt
            .query_map(params![hall_id.to_string()], |row| {
                Ok(VoiceState {
                    user_id: parse_uuid(&row.get::<_, String>(0)?)?,
                    channel_id: parse_uuid(&row.get::<_, String>(1)?)?,
                    hall_id: parse_uuid(&row.get::<_, String>(2)?)?,
                    self_mute: row.get::<_, i32>(3)? != 0,
                    self_deaf: row.get::<_, i32>(4)? != 0,
                    server_mute: row.get::<_, i32>(5)? != 0,
                    server_deaf: row.get::<_, i32>(6)? != 0,
                    streaming: row.get::<_, i32>(7)? != 0,
                    video: row.get::<_, i32>(8)? != 0,
                    connected_at: parse_datetime(&row.get::<_, String>(9)?)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(states)
    }

    /// Update voice state flags
    #[instrument(skip(self))]
    pub fn update(&self, state: &VoiceState) -> Result<()> {
        self.conn.execute(
            "UPDATE voice_states SET self_mute = ?1, self_deaf = ?2, server_mute = ?3, server_deaf = ?4, streaming = ?5, video = ?6
             WHERE user_id = ?7 AND channel_id = ?8",
            params![
                state.self_mute as i32,
                state.self_deaf as i32,
                state.server_mute as i32,
                state.server_deaf as i32,
                state.streaming as i32,
                state.video as i32,
                state.user_id.to_string(),
                state.channel_id.to_string(),
            ],
        )?;
        Ok(())
    }

    /// Disconnect user from voice (leave channel)
    #[instrument(skip(self))]
    pub fn disconnect_user(&self, user_id: Uuid) -> Result<()> {
        self.conn.execute(
            "DELETE FROM voice_states WHERE user_id = ?1",
            params![user_id.to_string()],
        )?;
        Ok(())
    }

    /// Count users in a voice channel
    #[instrument(skip(self))]
    pub fn count_in_channel(&self, channel_id: Uuid) -> Result<u32> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM voice_states WHERE channel_id = ?1",
            params![channel_id.to_string()],
            |row| row.get(0),
        )?;
        Ok(count as u32)
    }
}
