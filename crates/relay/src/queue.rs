//! Offline message queue — SQLite-backed durable queue
//!
//! When a user is offline, messages destined for them are queued.
//! On reconnect, the client drains the queue. Entries expire after 7 days.

use chrono::{Duration, Utc};
use rusqlite::{params, Connection};
use uuid::Uuid;

use exom_protocol::{QueuedMessage, ServerMessage};

/// Offline message queue backed by SQLite
pub struct MessageQueue {
    conn: Connection,
}

impl MessageQueue {
    /// Open or create the queue database
    pub fn open(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let conn = Connection::open(path)?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS message_queue (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id TEXT NOT NULL,
                payload BLOB NOT NULL,
                created_at TEXT NOT NULL,
                delivered INTEGER NOT NULL DEFAULT 0
            );
            CREATE INDEX IF NOT EXISTS idx_queue_user ON message_queue(user_id, delivered);
            CREATE INDEX IF NOT EXISTS idx_queue_created ON message_queue(created_at);",
        )?;
        Ok(Self { conn })
    }

    /// Queue a message for an offline user
    pub fn enqueue(&self, user_id: Uuid, msg: &ServerMessage) -> Result<(), Box<dyn std::error::Error>> {
        let payload = rmp_serde::to_vec(msg)?;
        self.conn.execute(
            "INSERT INTO message_queue (user_id, payload, created_at) VALUES (?1, ?2, ?3)",
            params![user_id.to_string(), payload, Utc::now().to_rfc3339()],
        )?;
        Ok(())
    }

    /// Drain all pending messages for a user (marks as delivered)
    pub fn drain(&self, user_id: Uuid) -> Result<Vec<QueuedMessage>, Box<dyn std::error::Error>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, payload, created_at FROM message_queue
             WHERE user_id = ?1 AND delivered = 0
             ORDER BY id ASC",
        )?;

        let rows: Vec<(i64, Vec<u8>, String)> = stmt
            .query_map(params![user_id.to_string()], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?))
            })?
            .collect::<Result<Vec<_>, _>>()?;

        let mut messages = Vec::new();
        let mut ids = Vec::new();

        for (id, payload, created_at_str) in rows {
            if let Ok(event) = rmp_serde::from_slice::<ServerMessage>(&payload) {
                let queued_at = chrono::DateTime::parse_from_rfc3339(&created_at_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now());

                messages.push(QueuedMessage { queued_at, event });
                ids.push(id);
            }
        }

        // Mark as delivered
        if !ids.is_empty() {
            let placeholders: Vec<String> = ids.iter().map(|id| id.to_string()).collect();
            let sql = format!(
                "UPDATE message_queue SET delivered = 1 WHERE id IN ({})",
                placeholders.join(",")
            );
            self.conn.execute_batch(&sql)?;
        }

        Ok(messages)
    }

    /// Clean up old/delivered messages
    pub fn cleanup(&self) -> Result<u64, Box<dyn std::error::Error>> {
        let cutoff = (Utc::now() - Duration::days(7)).to_rfc3339();

        // Delete delivered messages immediately, expired after 7 days
        let count = self.conn.execute(
            "DELETE FROM message_queue WHERE delivered = 1 OR created_at < ?1",
            params![cutoff],
        )?;

        Ok(count as u64)
    }

    /// Count pending messages for a user
    pub fn pending_count(&self, user_id: Uuid) -> Result<u64, Box<dyn std::error::Error>> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM message_queue WHERE user_id = ?1 AND delivered = 0",
            params![user_id.to_string()],
            |row| row.get(0),
        )?;
        Ok(count as u64)
    }
}
