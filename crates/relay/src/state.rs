//! Relay state — shared state across all connections

use std::collections::{HashMap, HashSet};

use tokio::sync::{Mutex, RwLock};
use uuid::Uuid;

use exom_protocol::ServerMessage;

use crate::queue::MessageQueue;
use crate::ratelimit::{RateLimitKind, RateLimiter};

/// Per-connection sender handle
pub type ClientSender = tokio::sync::mpsc::UnboundedSender<ServerMessage>;

/// A connected client
#[derive(Debug)]
pub struct ConnectedClient {
    pub user_id: Uuid,
    pub sender: ClientSender,
    /// Halls this client is subscribed to
    pub halls: HashSet<Uuid>,
    /// Voice channel (if in one)
    pub voice_channel: Option<(Uuid, Uuid)>, // (hall_id, channel_id)
}

/// Shared relay state
pub struct RelayState {
    /// All connected clients, keyed by connection ID
    pub clients: RwLock<HashMap<Uuid, ConnectedClient>>,
    /// Reverse lookup: user_id -> connection_id (a user can only have one connection)
    pub user_connections: RwLock<HashMap<Uuid, Uuid>>,
    /// Hall subscriptions: hall_id -> set of connection_ids
    pub hall_members: RwLock<HashMap<Uuid, HashSet<Uuid>>>,
    /// Hall member registry: hall_id -> all member user_ids (for offline queuing)
    pub hall_all_members: RwLock<HashMap<Uuid, HashSet<Uuid>>>,
    /// DM channel participants: dm_channel_id -> set of user_ids
    pub dm_participants: RwLock<HashMap<Uuid, HashSet<Uuid>>>,
    /// Voice channels: (hall_id, channel_id) -> set of user_ids
    pub voice_channels: RwLock<HashMap<(Uuid, Uuid), HashSet<Uuid>>>,
    /// Offline message queue
    pub queue: Mutex<MessageQueue>,
    /// Relay secret for token validation
    pub secret: String,
    /// File storage base path
    pub file_storage_path: std::path::PathBuf,
    /// Rate limiter
    pub rate_limiter: Mutex<RateLimiter>,
}

impl RelayState {
    pub fn new(db_path: &str, secret: String) -> Result<Self, Box<dyn std::error::Error>> {
        let queue = MessageQueue::open(db_path)?;
        let file_path = std::path::PathBuf::from("data/files");
        std::fs::create_dir_all(&file_path)?;
        Ok(Self {
            clients: RwLock::new(HashMap::new()),
            user_connections: RwLock::new(HashMap::new()),
            hall_members: RwLock::new(HashMap::new()),
            hall_all_members: RwLock::new(HashMap::new()),
            dm_participants: RwLock::new(HashMap::new()),
            voice_channels: RwLock::new(HashMap::new()),
            queue: Mutex::new(queue),
            secret,
            file_storage_path: file_path,
            rate_limiter: Mutex::new(RateLimiter::new()),
        })
    }

    /// Register a new client connection
    pub async fn register_client(
        &self,
        conn_id: Uuid,
        user_id: Uuid,
        sender: ClientSender,
    ) {
        // Disconnect any existing connection for this user
        if let Some(old_conn_id) = self.user_connections.read().await.get(&user_id).copied() {
            self.remove_client(old_conn_id).await;
        }

        let client = ConnectedClient {
            user_id,
            sender,
            halls: HashSet::new(),
            voice_channel: None,
        };

        self.clients.write().await.insert(conn_id, client);
        self.user_connections.write().await.insert(user_id, conn_id);
    }

    /// Remove a client connection and clean up all subscriptions
    pub async fn remove_client(&self, conn_id: Uuid) {
        let client = self.clients.write().await.remove(&conn_id);

        if let Some(client) = client {
            self.user_connections.write().await.remove(&client.user_id);

            // Remove from all hall subscriptions
            let mut hall_members = self.hall_members.write().await;
            for hall_id in &client.halls {
                if let Some(members) = hall_members.get_mut(hall_id) {
                    members.remove(&conn_id);
                    if members.is_empty() {
                        hall_members.remove(hall_id);
                    }
                }
            }

            // Remove from voice channel
            if let Some((hall_id, channel_id)) = client.voice_channel {
                let mut voice = self.voice_channels.write().await;
                if let Some(users) = voice.get_mut(&(hall_id, channel_id)) {
                    users.remove(&client.user_id);
                    if users.is_empty() {
                        voice.remove(&(hall_id, channel_id));
                    }
                }
            }
        }
    }

    /// Subscribe a connection to a Hall
    pub async fn join_hall(&self, conn_id: Uuid, hall_id: Uuid) {
        if let Some(client) = self.clients.write().await.get_mut(&conn_id) {
            client.halls.insert(hall_id);
        }
        self.hall_members
            .write()
            .await
            .entry(hall_id)
            .or_default()
            .insert(conn_id);
    }

    /// Unsubscribe a connection from a Hall
    pub async fn leave_hall(&self, conn_id: Uuid, hall_id: Uuid) {
        if let Some(client) = self.clients.write().await.get_mut(&conn_id) {
            client.halls.remove(&hall_id);
        }
        let mut hall_members = self.hall_members.write().await;
        if let Some(members) = hall_members.get_mut(&hall_id) {
            members.remove(&conn_id);
            if members.is_empty() {
                hall_members.remove(&hall_id);
            }
        }
    }

    /// Broadcast a message to all clients in a Hall (except the sender)
    pub async fn broadcast_to_hall(
        &self,
        hall_id: Uuid,
        exclude_conn: Option<Uuid>,
        msg: ServerMessage,
    ) {
        let hall_members = self.hall_members.read().await;
        let clients = self.clients.read().await;

        if let Some(members) = hall_members.get(&hall_id) {
            for conn_id in members {
                if Some(*conn_id) == exclude_conn {
                    continue;
                }
                if let Some(client) = clients.get(conn_id) {
                    let _ = client.sender.send(msg.clone());
                }
            }
        }
    }

    /// Send a message to a specific user (by user_id)
    pub async fn send_to_user(&self, user_id: Uuid, msg: ServerMessage) -> bool {
        let user_conns = self.user_connections.read().await;
        if let Some(conn_id) = user_conns.get(&user_id) {
            let clients = self.clients.read().await;
            if let Some(client) = clients.get(conn_id) {
                return client.sender.send(msg).is_ok();
            }
        }
        false
    }

    /// Check if a user is currently connected
    pub async fn is_online(&self, user_id: Uuid) -> bool {
        self.user_connections.read().await.contains_key(&user_id)
    }

    /// Get all online user_ids in a Hall
    pub async fn online_members_in_hall(&self, hall_id: Uuid) -> Vec<Uuid> {
        let hall_members = self.hall_members.read().await;
        let clients = self.clients.read().await;

        hall_members
            .get(&hall_id)
            .map(|members| {
                members
                    .iter()
                    .filter_map(|conn_id| clients.get(conn_id).map(|c| c.user_id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get connection ID for a user
    pub async fn get_conn_id(&self, user_id: Uuid) -> Option<Uuid> {
        self.user_connections.read().await.get(&user_id).copied()
    }

    /// Join a voice channel
    pub async fn join_voice(
        &self,
        conn_id: Uuid,
        user_id: Uuid,
        hall_id: Uuid,
        channel_id: Uuid,
    ) {
        // Leave any current voice channel first
        self.leave_voice(conn_id, user_id).await;

        if let Some(client) = self.clients.write().await.get_mut(&conn_id) {
            client.voice_channel = Some((hall_id, channel_id));
        }

        self.voice_channels
            .write()
            .await
            .entry((hall_id, channel_id))
            .or_default()
            .insert(user_id);
    }

    /// Leave voice channel
    pub async fn leave_voice(&self, conn_id: Uuid, user_id: Uuid) {
        let voice_channel = {
            let mut clients = self.clients.write().await;
            if let Some(client) = clients.get_mut(&conn_id) {
                client.voice_channel.take()
            } else {
                None
            }
        };

        if let Some((hall_id, channel_id)) = voice_channel {
            let mut voice = self.voice_channels.write().await;
            if let Some(users) = voice.get_mut(&(hall_id, channel_id)) {
                users.remove(&user_id);
                if users.is_empty() {
                    voice.remove(&(hall_id, channel_id));
                }
            }
        }
    }

    /// Get all users in a voice channel
    pub async fn users_in_voice(&self, hall_id: Uuid, channel_id: Uuid) -> Vec<Uuid> {
        self.voice_channels
            .read()
            .await
            .get(&(hall_id, channel_id))
            .map(|users| users.iter().copied().collect())
            .unwrap_or_default()
    }

    // --- DM routing ---

    /// Register DM channel participants
    pub async fn register_dm_participants(&self, channel_id: Uuid, participants: Vec<Uuid>) {
        self.dm_participants
            .write()
            .await
            .insert(channel_id, participants.into_iter().collect());
    }

    /// Get DM channel participants
    pub async fn get_dm_participants(&self, channel_id: Uuid) -> Vec<Uuid> {
        self.dm_participants
            .read()
            .await
            .get(&channel_id)
            .map(|p| p.iter().copied().collect())
            .unwrap_or_default()
    }

    /// Send a message to all DM participants (except sender), queue if offline
    pub async fn send_to_dm_participants(
        &self,
        channel_id: Uuid,
        exclude_user: Uuid,
        msg: &ServerMessage,
    ) {
        let participants = self.get_dm_participants(channel_id).await;
        for user_id in participants {
            if user_id == exclude_user {
                continue;
            }
            if !self.send_to_user(user_id, msg.clone()).await {
                // User is offline — queue it
                let queue = self.queue.lock().await;
                let _ = queue.enqueue(user_id, msg);
            }
        }
    }

    // --- Hall member registry ---

    /// Set the full member list for a Hall (for offline queuing)
    pub async fn sync_hall_members(&self, hall_id: Uuid, member_ids: Vec<Uuid>) -> u32 {
        let count = member_ids.len() as u32;
        self.hall_all_members
            .write()
            .await
            .insert(hall_id, member_ids.into_iter().collect());
        count
    }

    /// Queue a message for all offline members of a hall
    pub async fn queue_for_offline_members(
        &self,
        hall_id: Uuid,
        exclude_user: Uuid,
        msg: &ServerMessage,
    ) {
        let all_members = {
            self.hall_all_members
                .read()
                .await
                .get(&hall_id)
                .cloned()
                .unwrap_or_default()
        };

        let online_members: HashSet<Uuid> = self.online_members_in_hall(hall_id).await.into_iter().collect();

        let queue = self.queue.lock().await;
        for user_id in all_members {
            if user_id == exclude_user {
                continue;
            }
            if !online_members.contains(&user_id) {
                let _ = queue.enqueue(user_id, msg);
            }
        }
    }

    // --- Rate limiting ---

    /// Check if an action is allowed under rate limits
    pub async fn check_rate_limit(&self, user_id: Uuid, kind: RateLimitKind) -> bool {
        self.rate_limiter.lock().await.check(user_id, kind)
    }

    /// Clean up stale rate limit buckets
    pub async fn cleanup_rate_limits(&self) {
        self.rate_limiter.lock().await.cleanup();
    }
}
