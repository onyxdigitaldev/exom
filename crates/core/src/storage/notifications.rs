//! Notification storage — read states, mentions, settings

use rusqlite::{params, Connection};
use tracing::instrument;
use uuid::Uuid;

use super::parse::{parse_datetime_opt, parse_uuid, parse_uuid_opt, OptionalExt};
use crate::error::Result;
use crate::models::{Mention, MentionType, MuteLevel, NotificationSettings, ReadState};

fn mute_level_from_u8(v: u8) -> MuteLevel {
    match v {
        0 => MuteLevel::All,
        1 => MuteLevel::MentionsOnly,
        2 => MuteLevel::Nothing,
        _ => MuteLevel::All,
    }
}

fn mention_type_from_u8(v: u8) -> MentionType {
    match v {
        0 => MentionType::User,
        1 => MentionType::Role,
        2 => MentionType::Everyone,
        3 => MentionType::Here,
        _ => MentionType::User,
    }
}

pub struct NotificationStore<'a> {
    conn: &'a Connection,
}

impl<'a> NotificationStore<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    // --- Read States ---

    #[instrument(skip(self))]
    pub fn upsert_read_state(&self, state: &ReadState) -> Result<()> {
        self.conn.execute(
            "INSERT INTO read_states (id, user_id, channel_id, last_read_message_id, mention_count)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(user_id, channel_id) DO UPDATE SET
                last_read_message_id = excluded.last_read_message_id,
                mention_count = excluded.mention_count",
            params![
                state.id.to_string(),
                state.user_id.to_string(),
                state.channel_id.to_string(),
                state.last_read_message_id.map(|m| m.to_string()),
                state.mention_count,
            ],
        )?;
        Ok(())
    }

    #[instrument(skip(self))]
    pub fn find_read_state(&self, user_id: Uuid, channel_id: Uuid) -> Result<Option<ReadState>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, user_id, channel_id, last_read_message_id, mention_count
             FROM read_states WHERE user_id = ?1 AND channel_id = ?2",
        )?;

        let state = stmt
            .query_row(
                params![user_id.to_string(), channel_id.to_string()],
                |row| {
                    Ok(ReadState {
                        id: parse_uuid(&row.get::<_, String>(0)?)?,
                        user_id: parse_uuid(&row.get::<_, String>(1)?)?,
                        channel_id: parse_uuid(&row.get::<_, String>(2)?)?,
                        last_read_message_id: parse_uuid_opt(
                            row.get::<_, Option<String>>(3)?,
                        )?,
                        mention_count: row.get(4)?,
                    })
                },
            )
            .optional()?;

        Ok(state)
    }

    /// Mark channel as read (update last_read_message_id, reset mention_count)
    #[instrument(skip(self))]
    pub fn mark_read(&self, user_id: Uuid, channel_id: Uuid, message_id: Uuid) -> Result<()> {
        self.conn.execute(
            "INSERT INTO read_states (id, user_id, channel_id, last_read_message_id, mention_count)
             VALUES (?1, ?2, ?3, ?4, 0)
             ON CONFLICT(user_id, channel_id) DO UPDATE SET
                last_read_message_id = excluded.last_read_message_id,
                mention_count = 0",
            params![
                Uuid::new_v4().to_string(),
                user_id.to_string(),
                channel_id.to_string(),
                message_id.to_string(),
            ],
        )?;
        Ok(())
    }

    /// Increment mention count for a user in a channel
    #[instrument(skip(self))]
    pub fn increment_mentions(&self, user_id: Uuid, channel_id: Uuid) -> Result<()> {
        self.conn.execute(
            "INSERT INTO read_states (id, user_id, channel_id, last_read_message_id, mention_count)
             VALUES (?1, ?2, ?3, NULL, 1)
             ON CONFLICT(user_id, channel_id) DO UPDATE SET
                mention_count = mention_count + 1",
            params![
                Uuid::new_v4().to_string(),
                user_id.to_string(),
                channel_id.to_string(),
            ],
        )?;
        Ok(())
    }

    /// List all channels with unread state for a user
    #[instrument(skip(self))]
    pub fn list_unread_channels(&self, user_id: Uuid) -> Result<Vec<ReadState>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, user_id, channel_id, last_read_message_id, mention_count
             FROM read_states WHERE user_id = ?1 AND mention_count > 0",
        )?;

        let states = stmt
            .query_map(params![user_id.to_string()], |row| {
                Ok(ReadState {
                    id: parse_uuid(&row.get::<_, String>(0)?)?,
                    user_id: parse_uuid(&row.get::<_, String>(1)?)?,
                    channel_id: parse_uuid(&row.get::<_, String>(2)?)?,
                    last_read_message_id: parse_uuid_opt(row.get::<_, Option<String>>(3)?)?,
                    mention_count: row.get(4)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(states)
    }

    // --- Notification Settings ---

    #[instrument(skip(self, settings))]
    pub fn upsert_settings(&self, settings: &NotificationSettings) -> Result<()> {
        self.conn.execute(
            "INSERT INTO notification_settings (id, user_id, channel_id, mute_level, mute_until)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(user_id, channel_id) DO UPDATE SET
                mute_level = excluded.mute_level,
                mute_until = excluded.mute_until",
            params![
                settings.id.to_string(),
                settings.user_id.to_string(),
                settings.channel_id.to_string(),
                settings.mute_level as u8,
                settings.mute_until.map(|t| t.to_rfc3339()),
            ],
        )?;
        Ok(())
    }

    #[instrument(skip(self))]
    pub fn find_settings(
        &self,
        user_id: Uuid,
        channel_id: Uuid,
    ) -> Result<Option<NotificationSettings>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, user_id, channel_id, mute_level, mute_until
             FROM notification_settings WHERE user_id = ?1 AND channel_id = ?2",
        )?;

        let settings = stmt
            .query_row(
                params![user_id.to_string(), channel_id.to_string()],
                |row| {
                    Ok(NotificationSettings {
                        id: parse_uuid(&row.get::<_, String>(0)?)?,
                        user_id: parse_uuid(&row.get::<_, String>(1)?)?,
                        channel_id: parse_uuid(&row.get::<_, String>(2)?)?,
                        mute_level: mute_level_from_u8(row.get::<_, u8>(3)?),
                        mute_until: parse_datetime_opt(row.get::<_, Option<String>>(4)?)?,
                    })
                },
            )
            .optional()?;

        Ok(settings)
    }

    // --- Mentions ---

    #[instrument(skip(self, mention))]
    pub fn create_mention(&self, mention: &Mention) -> Result<()> {
        self.conn.execute(
            "INSERT INTO mentions (id, message_id, user_id, mention_type)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                mention.id.to_string(),
                mention.message_id.to_string(),
                mention.user_id.to_string(),
                mention.mention_type as u8,
            ],
        )?;
        Ok(())
    }

    #[instrument(skip(self))]
    pub fn list_mentions_for_message(&self, message_id: Uuid) -> Result<Vec<Mention>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, message_id, user_id, mention_type
             FROM mentions WHERE message_id = ?1",
        )?;

        let mentions = stmt
            .query_map(params![message_id.to_string()], |row| {
                Ok(Mention {
                    id: parse_uuid(&row.get::<_, String>(0)?)?,
                    message_id: parse_uuid(&row.get::<_, String>(1)?)?,
                    user_id: parse_uuid(&row.get::<_, String>(2)?)?,
                    mention_type: mention_type_from_u8(row.get::<_, u8>(3)?),
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(mentions)
    }
}
