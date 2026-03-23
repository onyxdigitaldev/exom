//! Relay client — WebSocket connection with reconnection logic

use std::sync::Arc;
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use tokio::sync::{broadcast, mpsc, RwLock};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{error, info, warn};
use uuid::Uuid;

use exom_protocol::{
    decode_server, encode_client, ClientMessage, ServerMessage,
};

use super::token::generate_auth_token;

/// Connection state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Authenticating,
    Connected,
    Reconnecting,
}

/// Configuration for the relay client
#[derive(Debug, Clone)]
pub struct RelayClientConfig {
    /// WebSocket URL of the relay (e.g., "ws://localhost:9400/ws")
    pub relay_url: String,
    /// Shared secret for token generation
    pub secret: String,
    /// User ID to authenticate as
    pub user_id: Uuid,
    /// Maximum reconnection attempts (0 = infinite)
    pub max_reconnect_attempts: u32,
    /// Initial reconnection delay
    pub reconnect_delay: Duration,
    /// Maximum reconnection delay
    pub max_reconnect_delay: Duration,
    /// Heartbeat interval
    pub heartbeat_interval: Duration,
}

impl Default for RelayClientConfig {
    fn default() -> Self {
        Self {
            relay_url: "ws://localhost:9400/ws".to_string(),
            secret: String::new(),
            user_id: Uuid::nil(),
            max_reconnect_attempts: 0, // infinite
            reconnect_delay: Duration::from_secs(1),
            max_reconnect_delay: Duration::from_secs(30),
            heartbeat_interval: Duration::from_secs(30),
        }
    }
}

/// Relay client — manages the WebSocket connection to the relay
pub struct RelayClient {
    config: RelayClientConfig,
    state: Arc<RwLock<ConnectionState>>,
    /// Channel for sending messages TO the relay
    outgoing_tx: mpsc::UnboundedSender<ClientMessage>,
    outgoing_rx: Arc<tokio::sync::Mutex<mpsc::UnboundedReceiver<ClientMessage>>>,
    /// Channel for receiving messages FROM the relay
    incoming_tx: broadcast::Sender<ServerMessage>,
}

impl RelayClient {
    /// Create a new relay client (does not connect yet)
    pub fn new(config: RelayClientConfig) -> Self {
        let (outgoing_tx, outgoing_rx) = mpsc::unbounded_channel();
        let (incoming_tx, _) = broadcast::channel(256);

        Self {
            config,
            state: Arc::new(RwLock::new(ConnectionState::Disconnected)),
            outgoing_tx,
            outgoing_rx: Arc::new(tokio::sync::Mutex::new(outgoing_rx)),
            incoming_tx,
        }
    }

    /// Subscribe to incoming server messages
    pub fn subscribe(&self) -> broadcast::Receiver<ServerMessage> {
        self.incoming_tx.subscribe()
    }

    /// Get current connection state
    pub async fn connection_state(&self) -> ConnectionState {
        *self.state.read().await
    }

    /// Send a message to the relay
    pub fn send(&self, msg: ClientMessage) -> Result<(), mpsc::error::SendError<ClientMessage>> {
        self.outgoing_tx.send(msg)
    }

    /// Connect to the relay and start the connection loop
    /// This spawns background tasks and returns immediately.
    pub fn connect(&self) -> tokio::task::JoinHandle<()> {
        let config = self.config.clone();
        let state = self.state.clone();
        let outgoing_rx = self.outgoing_rx.clone();
        let incoming_tx = self.incoming_tx.clone();
        let outgoing_tx = self.outgoing_tx.clone();

        tokio::spawn(async move {
            let mut attempt = 0u32;
            let mut delay = config.reconnect_delay;

            loop {
                *state.write().await = ConnectionState::Connecting;
                info!(url = %config.relay_url, attempt = attempt, "connecting to relay");

                match connect_async(&config.relay_url).await {
                    Ok((ws_stream, _)) => {
                        attempt = 0;
                        delay = config.reconnect_delay;

                        *state.write().await = ConnectionState::Authenticating;

                        let (mut write, mut read) = ws_stream.split();

                        // Send auth
                        let auth = generate_auth_token(config.user_id, &config.secret);
                        let auth_msg = ClientMessage::Authenticate(auth);
                        match encode_client(&auth_msg) {
                            Ok(bytes) => {
                                if write.send(Message::Binary(bytes.into())).await.is_err() {
                                    error!("failed to send auth");
                                    *state.write().await = ConnectionState::Reconnecting;
                                    tokio::time::sleep(delay).await;
                                    continue;
                                }
                            }
                            Err(e) => {
                                error!("failed to encode auth: {}", e);
                                break;
                            }
                        }

                        // Wait for auth response
                        match read.next().await {
                            Some(Ok(Message::Binary(bytes))) => {
                                match decode_server(&bytes) {
                                    Ok(ServerMessage::AuthResult {
                                        success: true, ..
                                    }) => {
                                        info!("authenticated successfully");
                                        *state.write().await = ConnectionState::Connected;
                                    }
                                    Ok(ServerMessage::AuthResult {
                                        success: false,
                                        error,
                                        ..
                                    }) => {
                                        error!("auth failed: {:?}", error);
                                        *state.write().await = ConnectionState::Disconnected;
                                        break; // Don't retry auth failures
                                    }
                                    Ok(other) => {
                                        error!("unexpected auth response: {:?}", other);
                                        *state.write().await = ConnectionState::Reconnecting;
                                        tokio::time::sleep(delay).await;
                                        continue;
                                    }
                                    Err(e) => {
                                        error!("failed to decode auth response: {}", e);
                                        *state.write().await = ConnectionState::Reconnecting;
                                        tokio::time::sleep(delay).await;
                                        continue;
                                    }
                                }
                            }
                            _ => {
                                error!("connection closed during auth");
                                *state.write().await = ConnectionState::Reconnecting;
                                tokio::time::sleep(delay).await;
                                continue;
                            }
                        }

                        // Drain offline queue
                        let _ = outgoing_tx.send(ClientMessage::QueueDrain);

                        // Connected — run read/write loops
                        let read_incoming_tx = incoming_tx.clone();
                        let read_state = state.clone();

                        let read_task = tokio::spawn(async move {
                            while let Some(msg) = read.next().await {
                                match msg {
                                    Ok(Message::Binary(bytes)) => {
                                        match decode_server(&bytes) {
                                            Ok(server_msg) => {
                                                let _ = read_incoming_tx.send(server_msg);
                                            }
                                            Err(e) => {
                                                warn!("decode error: {}", e);
                                            }
                                        }
                                    }
                                    Ok(Message::Close(_)) => {
                                        info!("relay closed connection");
                                        break;
                                    }
                                    Ok(_) => {}
                                    Err(e) => {
                                        warn!("ws error: {}", e);
                                        break;
                                    }
                                }
                            }
                            *read_state.write().await = ConnectionState::Reconnecting;
                        });

                        let write_state = state.clone();
                        let heartbeat = config.heartbeat_interval;
                        let write_outgoing_rx = outgoing_rx.clone();

                        let write_task = tokio::spawn(async move {
                            let mut rx = write_outgoing_rx.lock().await;
                            let mut heartbeat_interval =
                                tokio::time::interval(heartbeat);

                            loop {
                                tokio::select! {
                                    Some(msg) = rx.recv() => {
                                        match encode_client(&msg) {
                                            Ok(bytes) => {
                                                if write.send(Message::Binary(bytes.into())).await.is_err() {
                                                    break;
                                                }
                                            }
                                            Err(e) => {
                                                error!("encode error: {}", e);
                                            }
                                        }
                                    }
                                    _ = heartbeat_interval.tick() => {
                                        if *write_state.read().await == ConnectionState::Connected {
                                            let hb = encode_client(&ClientMessage::Heartbeat).unwrap_or_default();
                                            if write.send(Message::Binary(hb.into())).await.is_err() {
                                                break;
                                            }
                                        }
                                    }
                                }
                            }
                            *write_state.write().await = ConnectionState::Reconnecting;
                        });

                        // Wait for either task to end
                        tokio::select! {
                            _ = read_task => {},
                            _ = write_task => {},
                        }

                        info!("connection lost, will reconnect");
                    }
                    Err(e) => {
                        warn!("connection failed: {}", e);
                    }
                }

                // Reconnect logic
                *state.write().await = ConnectionState::Reconnecting;
                attempt += 1;

                if config.max_reconnect_attempts > 0
                    && attempt >= config.max_reconnect_attempts
                {
                    error!("max reconnection attempts reached");
                    *state.write().await = ConnectionState::Disconnected;
                    break;
                }

                info!(delay_ms = delay.as_millis(), "reconnecting...");
                tokio::time::sleep(delay).await;

                // Exponential backoff
                delay = std::cmp::min(delay * 2, config.max_reconnect_delay);
            }
        })
    }

    // --- Convenience methods ---

    /// Join a Hall (subscribe to its events)
    pub fn join_hall(&self, hall_id: Uuid) {
        let _ = self.send(ClientMessage::HallJoin { hall_id });
    }

    /// Leave a Hall
    pub fn leave_hall(&self, hall_id: Uuid) {
        let _ = self.send(ClientMessage::HallLeave { hall_id });
    }

    /// Send a channel message
    pub fn send_message(
        &self,
        hall_id: Uuid,
        channel_id: Uuid,
        message: exom_protocol::MessagePayload,
    ) {
        let _ = self.send(ClientMessage::ChannelMessage {
            hall_id,
            channel_id,
            message,
        });
    }

    /// Start typing indicator
    pub fn start_typing(&self, hall_id: Uuid, channel_id: Uuid) {
        let _ = self.send(ClientMessage::TypingStart {
            hall_id,
            channel_id,
        });
    }

    /// Update presence
    pub fn update_presence(&self, status: u8, custom_text: Option<String>) {
        let _ = self.send(ClientMessage::PresenceUpdate {
            status,
            custom_text,
        });
    }

    /// Join voice channel
    pub fn join_voice(&self, hall_id: Uuid, channel_id: Uuid) {
        let _ = self.send(ClientMessage::VoiceJoin {
            hall_id,
            channel_id,
        });
    }

    /// Leave voice channel
    pub fn leave_voice(&self) {
        let _ = self.send(ClientMessage::VoiceLeave);
    }
}
