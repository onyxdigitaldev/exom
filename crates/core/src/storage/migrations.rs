//! Database migration system
//!
//! Tracks schema versions and applies migrations in order.

use rusqlite::Connection;
use tracing::{info, instrument};

use crate::error::Result;

/// A database migration
pub struct Migration {
    /// Version number (must be sequential starting from 1)
    pub version: u32,
    /// Description of what this migration does
    pub description: &'static str,
    /// SQL to run for this migration
    pub sql: &'static str,
}

/// All migrations in order
const MIGRATIONS: &[Migration] = &[
    Migration {
        version: 1,
        description: "Initial schema",
        sql: r#"
            -- Users table
            CREATE TABLE IF NOT EXISTS users (
                id TEXT PRIMARY KEY,
                username TEXT NOT NULL UNIQUE,
                password_hash TEXT NOT NULL,
                created_at TEXT NOT NULL,
                last_login TEXT
            );

            -- Sessions table
            CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                user_id TEXT NOT NULL,
                created_at TEXT NOT NULL,
                expires_at TEXT NOT NULL,
                FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
            );

            -- Halls table
            CREATE TABLE IF NOT EXISTS halls (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT,
                owner_id TEXT NOT NULL,
                created_at TEXT NOT NULL,
                active_parlor TEXT,
                current_host_id TEXT,
                election_epoch INTEGER NOT NULL DEFAULT 0,
                FOREIGN KEY (owner_id) REFERENCES users(id)
            );

            -- Memberships table
            CREATE TABLE IF NOT EXISTS memberships (
                id TEXT PRIMARY KEY,
                user_id TEXT NOT NULL,
                hall_id TEXT NOT NULL,
                role INTEGER NOT NULL,
                joined_at TEXT NOT NULL,
                is_online INTEGER NOT NULL DEFAULT 0,
                FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
                FOREIGN KEY (hall_id) REFERENCES halls(id) ON DELETE CASCADE,
                UNIQUE(user_id, hall_id)
            );

            -- Messages table
            CREATE TABLE IF NOT EXISTS messages (
                id TEXT PRIMARY KEY,
                hall_id TEXT NOT NULL,
                sender_id TEXT NOT NULL,
                content TEXT NOT NULL,
                created_at TEXT NOT NULL,
                edited_at TEXT,
                is_deleted INTEGER NOT NULL DEFAULT 0,
                FOREIGN KEY (hall_id) REFERENCES halls(id) ON DELETE CASCADE,
                FOREIGN KEY (sender_id) REFERENCES users(id)
            );

            -- Invites table
            CREATE TABLE IF NOT EXISTS invites (
                id TEXT PRIMARY KEY,
                hall_id TEXT NOT NULL,
                token TEXT NOT NULL UNIQUE,
                created_by TEXT NOT NULL,
                role INTEGER NOT NULL,
                created_at TEXT NOT NULL,
                expires_at TEXT,
                max_uses INTEGER,
                use_count INTEGER NOT NULL DEFAULT 0,
                is_revoked INTEGER NOT NULL DEFAULT 0,
                FOREIGN KEY (hall_id) REFERENCES halls(id) ON DELETE CASCADE,
                FOREIGN KEY (created_by) REFERENCES users(id)
            );
        "#,
    },
    Migration {
        version: 2,
        description: "Add indexes for query performance",
        sql: r#"
            -- Session indexes
            CREATE INDEX IF NOT EXISTS idx_sessions_user ON sessions(user_id);
            CREATE INDEX IF NOT EXISTS idx_sessions_expires ON sessions(expires_at);

            -- Membership indexes
            CREATE INDEX IF NOT EXISTS idx_memberships_user ON memberships(user_id);
            CREATE INDEX IF NOT EXISTS idx_memberships_hall ON memberships(hall_id);

            -- Message indexes
            CREATE INDEX IF NOT EXISTS idx_messages_hall ON messages(hall_id);
            CREATE INDEX IF NOT EXISTS idx_messages_created ON messages(created_at);
            CREATE INDEX IF NOT EXISTS idx_messages_hall_created ON messages(hall_id, created_at);
            CREATE INDEX IF NOT EXISTS idx_messages_sender ON messages(sender_id);

            -- Invite indexes
            CREATE INDEX IF NOT EXISTS idx_invites_token ON invites(token);
            CREATE INDEX IF NOT EXISTS idx_invites_hall ON invites(hall_id);
        "#,
    },
    Migration {
        version: 3,
        description: "Add channels, update messages for channel support, add user profiles, DMs, friends, reactions, attachments, bans, audit log, notifications, emoji, webhooks, voice state",
        sql: r#"
            -- Channels table
            CREATE TABLE IF NOT EXISTS channels (
                id TEXT PRIMARY KEY,
                hall_id TEXT NOT NULL,
                name TEXT NOT NULL,
                topic TEXT,
                channel_type INTEGER NOT NULL DEFAULT 0,
                parent_id TEXT,
                position INTEGER NOT NULL DEFAULT 0,
                slowmode_seconds INTEGER NOT NULL DEFAULT 0,
                nsfw INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                last_message_at TEXT,
                FOREIGN KEY (hall_id) REFERENCES halls(id) ON DELETE CASCADE,
                FOREIGN KEY (parent_id) REFERENCES channels(id) ON DELETE SET NULL
            );

            CREATE INDEX IF NOT EXISTS idx_channels_hall ON channels(hall_id);
            CREATE INDEX IF NOT EXISTS idx_channels_parent ON channels(parent_id);
            CREATE INDEX IF NOT EXISTS idx_channels_hall_position ON channels(hall_id, position);

            -- Migrate messages: add channel_id, reply_to, thread_id, is_pinned columns
            -- For existing messages, we create a default "general" channel per hall
            ALTER TABLE messages ADD COLUMN channel_id TEXT REFERENCES channels(id);
            ALTER TABLE messages ADD COLUMN reply_to TEXT REFERENCES messages(id);
            ALTER TABLE messages ADD COLUMN thread_id TEXT REFERENCES messages(id);
            ALTER TABLE messages ADD COLUMN is_pinned INTEGER NOT NULL DEFAULT 0;

            CREATE INDEX IF NOT EXISTS idx_messages_channel ON messages(channel_id);
            CREATE INDEX IF NOT EXISTS idx_messages_channel_created ON messages(channel_id, created_at);
            CREATE INDEX IF NOT EXISTS idx_messages_thread ON messages(thread_id);
            CREATE INDEX IF NOT EXISTS idx_messages_pinned ON messages(channel_id, is_pinned);

            -- User profiles
            CREATE TABLE IF NOT EXISTS user_profiles (
                user_id TEXT PRIMARY KEY,
                display_name TEXT,
                avatar_hash TEXT,
                bio TEXT,
                status INTEGER NOT NULL DEFAULT 0,
                custom_status_text TEXT,
                custom_status_emoji TEXT,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
            );

            -- DM channels
            CREATE TABLE IF NOT EXISTS dm_channels (
                id TEXT PRIMARY KEY,
                channel_type INTEGER NOT NULL DEFAULT 0,
                name TEXT,
                icon_hash TEXT,
                owner_id TEXT,
                created_at TEXT NOT NULL,
                last_message_at TEXT,
                FOREIGN KEY (owner_id) REFERENCES users(id)
            );

            -- DM participants
            CREATE TABLE IF NOT EXISTS dm_participants (
                id TEXT PRIMARY KEY,
                channel_id TEXT NOT NULL,
                user_id TEXT NOT NULL,
                joined_at TEXT NOT NULL,
                FOREIGN KEY (channel_id) REFERENCES dm_channels(id) ON DELETE CASCADE,
                FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
                UNIQUE(channel_id, user_id)
            );

            CREATE INDEX IF NOT EXISTS idx_dm_participants_user ON dm_participants(user_id);
            CREATE INDEX IF NOT EXISTS idx_dm_participants_channel ON dm_participants(channel_id);

            -- Direct messages
            CREATE TABLE IF NOT EXISTS direct_messages (
                id TEXT PRIMARY KEY,
                channel_id TEXT NOT NULL,
                sender_id TEXT NOT NULL,
                content TEXT NOT NULL,
                created_at TEXT NOT NULL,
                edited_at TEXT,
                is_deleted INTEGER NOT NULL DEFAULT 0,
                reply_to TEXT,
                FOREIGN KEY (channel_id) REFERENCES dm_channels(id) ON DELETE CASCADE,
                FOREIGN KEY (sender_id) REFERENCES users(id),
                FOREIGN KEY (reply_to) REFERENCES direct_messages(id)
            );

            CREATE INDEX IF NOT EXISTS idx_dm_channel ON direct_messages(channel_id);
            CREATE INDEX IF NOT EXISTS idx_dm_channel_created ON direct_messages(channel_id, created_at);

            -- Relationships (friends, blocks)
            CREATE TABLE IF NOT EXISTS relationships (
                id TEXT PRIMARY KEY,
                user_id TEXT NOT NULL,
                target_id TEXT NOT NULL,
                relationship_type INTEGER NOT NULL,
                created_at TEXT NOT NULL,
                FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
                FOREIGN KEY (target_id) REFERENCES users(id) ON DELETE CASCADE,
                UNIQUE(user_id, target_id)
            );

            CREATE INDEX IF NOT EXISTS idx_relationships_user ON relationships(user_id);
            CREATE INDEX IF NOT EXISTS idx_relationships_target ON relationships(target_id);

            -- Message reactions
            CREATE TABLE IF NOT EXISTS reactions (
                id TEXT PRIMARY KEY,
                message_id TEXT NOT NULL,
                user_id TEXT NOT NULL,
                emoji TEXT NOT NULL,
                is_custom INTEGER NOT NULL DEFAULT 0,
                custom_emoji_id TEXT,
                created_at TEXT NOT NULL,
                FOREIGN KEY (message_id) REFERENCES messages(id) ON DELETE CASCADE,
                FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
                UNIQUE(message_id, user_id, emoji)
            );

            CREATE INDEX IF NOT EXISTS idx_reactions_message ON reactions(message_id);

            -- Message attachments
            CREATE TABLE IF NOT EXISTS attachments (
                id TEXT PRIMARY KEY,
                message_id TEXT NOT NULL,
                filename TEXT NOT NULL,
                size_bytes INTEGER NOT NULL,
                content_type TEXT,
                hash TEXT NOT NULL,
                width INTEGER,
                height INTEGER,
                created_at TEXT NOT NULL,
                FOREIGN KEY (message_id) REFERENCES messages(id) ON DELETE CASCADE
            );

            CREATE INDEX IF NOT EXISTS idx_attachments_message ON attachments(message_id);

            -- Message embeds
            CREATE TABLE IF NOT EXISTS embeds (
                id TEXT PRIMARY KEY,
                message_id TEXT NOT NULL,
                embed_type TEXT NOT NULL,
                title TEXT,
                description TEXT,
                url TEXT,
                color INTEGER,
                thumbnail_url TEXT,
                image_url TEXT,
                author_name TEXT,
                author_url TEXT,
                footer_text TEXT,
                created_at TEXT NOT NULL,
                FOREIGN KEY (message_id) REFERENCES messages(id) ON DELETE CASCADE
            );

            CREATE INDEX IF NOT EXISTS idx_embeds_message ON embeds(message_id);

            -- Bans
            CREATE TABLE IF NOT EXISTS bans (
                id TEXT PRIMARY KEY,
                hall_id TEXT NOT NULL,
                user_id TEXT NOT NULL,
                banned_by TEXT NOT NULL,
                reason TEXT,
                created_at TEXT NOT NULL,
                expires_at TEXT,
                FOREIGN KEY (hall_id) REFERENCES halls(id) ON DELETE CASCADE,
                FOREIGN KEY (user_id) REFERENCES users(id),
                FOREIGN KEY (banned_by) REFERENCES users(id),
                UNIQUE(hall_id, user_id)
            );

            CREATE INDEX IF NOT EXISTS idx_bans_hall ON bans(hall_id);
            CREATE INDEX IF NOT EXISTS idx_bans_user ON bans(user_id);

            -- Audit log
            CREATE TABLE IF NOT EXISTS audit_log (
                id TEXT PRIMARY KEY,
                hall_id TEXT NOT NULL,
                actor_id TEXT NOT NULL,
                action_type TEXT NOT NULL,
                target_type TEXT,
                target_id TEXT,
                changes TEXT,
                reason TEXT,
                created_at TEXT NOT NULL,
                FOREIGN KEY (hall_id) REFERENCES halls(id) ON DELETE CASCADE,
                FOREIGN KEY (actor_id) REFERENCES users(id)
            );

            CREATE INDEX IF NOT EXISTS idx_audit_log_hall ON audit_log(hall_id);
            CREATE INDEX IF NOT EXISTS idx_audit_log_hall_created ON audit_log(hall_id, created_at);
            CREATE INDEX IF NOT EXISTS idx_audit_log_actor ON audit_log(actor_id);

            -- Notification / read state
            CREATE TABLE IF NOT EXISTS read_states (
                id TEXT PRIMARY KEY,
                user_id TEXT NOT NULL,
                channel_id TEXT NOT NULL,
                last_read_message_id TEXT,
                mention_count INTEGER NOT NULL DEFAULT 0,
                FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
                FOREIGN KEY (channel_id) REFERENCES channels(id) ON DELETE CASCADE,
                UNIQUE(user_id, channel_id)
            );

            CREATE INDEX IF NOT EXISTS idx_read_states_user ON read_states(user_id);

            -- Notification settings per channel
            CREATE TABLE IF NOT EXISTS notification_settings (
                id TEXT PRIMARY KEY,
                user_id TEXT NOT NULL,
                channel_id TEXT NOT NULL,
                mute_level INTEGER NOT NULL DEFAULT 0,
                mute_until TEXT,
                FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
                FOREIGN KEY (channel_id) REFERENCES channels(id) ON DELETE CASCADE,
                UNIQUE(user_id, channel_id)
            );

            -- Mentions
            CREATE TABLE IF NOT EXISTS mentions (
                id TEXT PRIMARY KEY,
                message_id TEXT NOT NULL,
                user_id TEXT NOT NULL,
                mention_type INTEGER NOT NULL DEFAULT 0,
                FOREIGN KEY (message_id) REFERENCES messages(id) ON DELETE CASCADE,
                FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
            );

            CREATE INDEX IF NOT EXISTS idx_mentions_user ON mentions(user_id);
            CREATE INDEX IF NOT EXISTS idx_mentions_message ON mentions(message_id);

            -- Custom emoji
            CREATE TABLE IF NOT EXISTS custom_emoji (
                id TEXT PRIMARY KEY,
                hall_id TEXT NOT NULL,
                name TEXT NOT NULL,
                image_hash TEXT NOT NULL,
                creator_id TEXT NOT NULL,
                animated INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                FOREIGN KEY (hall_id) REFERENCES halls(id) ON DELETE CASCADE,
                FOREIGN KEY (creator_id) REFERENCES users(id),
                UNIQUE(hall_id, name)
            );

            CREATE INDEX IF NOT EXISTS idx_custom_emoji_hall ON custom_emoji(hall_id);

            -- Webhooks
            CREATE TABLE IF NOT EXISTS webhooks (
                id TEXT PRIMARY KEY,
                channel_id TEXT NOT NULL,
                hall_id TEXT NOT NULL,
                name TEXT NOT NULL,
                avatar_hash TEXT,
                token TEXT NOT NULL UNIQUE,
                creator_id TEXT NOT NULL,
                created_at TEXT NOT NULL,
                FOREIGN KEY (channel_id) REFERENCES channels(id) ON DELETE CASCADE,
                FOREIGN KEY (hall_id) REFERENCES halls(id) ON DELETE CASCADE,
                FOREIGN KEY (creator_id) REFERENCES users(id)
            );

            CREATE INDEX IF NOT EXISTS idx_webhooks_channel ON webhooks(channel_id);
            CREATE INDEX IF NOT EXISTS idx_webhooks_hall ON webhooks(hall_id);

            -- Voice state
            CREATE TABLE IF NOT EXISTS voice_states (
                user_id TEXT NOT NULL,
                channel_id TEXT NOT NULL,
                hall_id TEXT NOT NULL,
                self_mute INTEGER NOT NULL DEFAULT 0,
                self_deaf INTEGER NOT NULL DEFAULT 0,
                server_mute INTEGER NOT NULL DEFAULT 0,
                server_deaf INTEGER NOT NULL DEFAULT 0,
                streaming INTEGER NOT NULL DEFAULT 0,
                video INTEGER NOT NULL DEFAULT 0,
                connected_at TEXT NOT NULL,
                PRIMARY KEY (user_id, channel_id),
                FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
                FOREIGN KEY (channel_id) REFERENCES channels(id) ON DELETE CASCADE,
                FOREIGN KEY (hall_id) REFERENCES halls(id) ON DELETE CASCADE
            );

            CREATE INDEX IF NOT EXISTS idx_voice_states_channel ON voice_states(channel_id);
            CREATE INDEX IF NOT EXISTS idx_voice_states_hall ON voice_states(hall_id);

            -- Permission overrides (per-channel)
            CREATE TABLE IF NOT EXISTS permission_overrides (
                id TEXT PRIMARY KEY,
                channel_id TEXT NOT NULL,
                target_type INTEGER NOT NULL,
                target_id TEXT NOT NULL,
                allow_bits INTEGER NOT NULL DEFAULT 0,
                deny_bits INTEGER NOT NULL DEFAULT 0,
                FOREIGN KEY (channel_id) REFERENCES channels(id) ON DELETE CASCADE,
                UNIQUE(channel_id, target_type, target_id)
            );

            CREATE INDEX IF NOT EXISTS idx_perm_overrides_channel ON permission_overrides(channel_id);

            -- Hall discovery
            CREATE TABLE IF NOT EXISTS hall_discovery (
                hall_id TEXT PRIMARY KEY,
                description TEXT,
                category TEXT,
                tags TEXT,
                member_count_approx INTEGER NOT NULL DEFAULT 0,
                is_listed INTEGER NOT NULL DEFAULT 0,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (hall_id) REFERENCES halls(id) ON DELETE CASCADE
            );

            CREATE INDEX IF NOT EXISTS idx_hall_discovery_listed ON hall_discovery(is_listed);
            CREATE INDEX IF NOT EXISTS idx_hall_discovery_category ON hall_discovery(category);

            -- Vanity invites
            CREATE TABLE IF NOT EXISTS vanity_invites (
                hall_id TEXT PRIMARY KEY,
                slug TEXT NOT NULL UNIQUE,
                FOREIGN KEY (hall_id) REFERENCES halls(id) ON DELETE CASCADE
            );

            -- Hall customization columns
            ALTER TABLE halls ADD COLUMN icon_hash TEXT;
            ALTER TABLE halls ADD COLUMN banner_hash TEXT;
            ALTER TABLE halls ADD COLUMN splash_hash TEXT;
        "#,
    },
];

/// Initialize the migrations table
fn init_migrations_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY,
            description TEXT NOT NULL,
            applied_at TEXT NOT NULL
        )",
        [],
    )?;
    Ok(())
}

/// Get the current schema version
fn get_current_version(conn: &Connection) -> Result<u32> {
    let version: Option<u32> = conn
        .query_row("SELECT MAX(version) FROM schema_migrations", [], |row| {
            row.get(0)
        })
        .unwrap_or(None);
    Ok(version.unwrap_or(0))
}

/// Record that a migration was applied
fn record_migration(conn: &Connection, migration: &Migration) -> Result<()> {
    conn.execute(
        "INSERT INTO schema_migrations (version, description, applied_at) VALUES (?1, ?2, ?3)",
        rusqlite::params![
            migration.version,
            migration.description,
            chrono::Utc::now().to_rfc3339()
        ],
    )?;
    Ok(())
}

/// Run all pending migrations
#[instrument(skip(conn))]
pub fn run_migrations(conn: &Connection) -> Result<()> {
    init_migrations_table(conn)?;

    let current_version = get_current_version(conn)?;
    info!(current_version, "Checking for pending migrations");

    for migration in MIGRATIONS {
        if migration.version > current_version {
            info!(
                version = migration.version,
                description = migration.description,
                "Applying migration"
            );

            conn.execute_batch(migration.sql)?;
            record_migration(conn, migration)?;

            info!(version = migration.version, "Migration complete");
        }
    }

    let new_version = get_current_version(conn)?;
    if new_version > current_version {
        info!(
            from = current_version,
            to = new_version,
            "Database schema updated"
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Get the latest migration version (test helper)
    fn latest_version() -> u32 {
        MIGRATIONS.last().map(|m| m.version).unwrap_or(0)
    }

    #[test]
    fn test_migrations_run() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();

        let version = get_current_version(&conn).unwrap();
        assert_eq!(version, latest_version());
    }

    #[test]
    fn test_migrations_idempotent() {
        let conn = Connection::open_in_memory().unwrap();

        // Run twice
        run_migrations(&conn).unwrap();
        run_migrations(&conn).unwrap();

        let version = get_current_version(&conn).unwrap();
        assert_eq!(version, latest_version());
    }

    #[test]
    fn test_migrations_sequential() {
        // Verify migrations are numbered sequentially
        for (i, migration) in MIGRATIONS.iter().enumerate() {
            assert_eq!(
                migration.version as usize,
                i + 1,
                "Migration {} should have version {}",
                migration.description,
                i + 1
            );
        }
    }
}
