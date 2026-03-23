//! WebSocket connection handler
//!
//! Manages the lifecycle of a single client connection:
//! 1. Wait for authentication
//! 2. Spawn read/write tasks
//! 3. Route incoming messages
//! 4. Clean up on disconnect

use std::sync::Arc;
use std::time::Duration;

use axum::extract::ws::{Message, WebSocket};
use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use tokio::time::timeout;
use tracing::{error, info, warn};
use uuid::Uuid;

use exom_protocol::{
    decode_client, encode_server, ClientMessage, ErrorCode, ServerMessage,
};

use crate::auth::validate_auth;
use crate::router;
use crate::state::RelayState;

/// Authentication timeout
const AUTH_TIMEOUT: Duration = Duration::from_secs(10);

/// Heartbeat timeout (disconnect if no heartbeat for 90s)
const HEARTBEAT_TIMEOUT: Duration = Duration::from_secs(90);

/// Handle a new WebSocket connection
pub async fn handle_connection(socket: WebSocket, state: Arc<RelayState>) {
    let conn_id = Uuid::new_v4();

    let (mut ws_sender, mut ws_receiver) = socket.split();

    // Step 1: Wait for authentication
    let user_id = match timeout(AUTH_TIMEOUT, authenticate(&mut ws_receiver, &state)).await {
        Ok(Ok(uid)) => uid,
        Ok(Err(err_msg)) => {
            let _ = ws_sender
                .send(Message::Binary(
                    encode_server(&err_msg).unwrap_or_default().into(),
                ))
                .await;
            return;
        }
        Err(_) => {
            warn!(conn_id = %conn_id, "authentication timed out");
            return;
        }
    };

    info!(conn_id = %conn_id, user_id = %user_id, "client authenticated");

    // Send auth success
    let auth_ok = ServerMessage::AuthResult {
        success: true,
        user_id: Some(user_id),
        error: None,
    };
    if ws_sender
        .send(Message::Binary(
            encode_server(&auth_ok).unwrap_or_default().into(),
        ))
        .await
        .is_err()
    {
        return;
    }

    // Step 2: Set up channels
    let (tx, mut rx) = mpsc::unbounded_channel::<ServerMessage>();

    state.register_client(conn_id, user_id, tx).await;

    // Step 3: Spawn write task (relay -> client)
    let write_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            match encode_server(&msg) {
                Ok(bytes) => {
                    if ws_sender.send(Message::Binary(bytes.into())).await.is_err() {
                        break;
                    }
                }
                Err(e) => {
                    error!("failed to encode server message: {}", e);
                }
            }
        }
    });

    // Step 4: Read loop (client -> relay)
    let read_state = state.clone();
    let read_task = tokio::spawn(async move {
        let mut last_heartbeat = tokio::time::Instant::now();

        loop {
            match timeout(HEARTBEAT_TIMEOUT, ws_receiver.next()).await {
                Ok(Some(Ok(msg))) => {
                    last_heartbeat = tokio::time::Instant::now();

                    match msg {
                        Message::Binary(bytes) => {
                            match decode_client(&bytes) {
                                Ok(client_msg) => {
                                    router::route(&read_state, conn_id, user_id, client_msg)
                                        .await;
                                }
                                Err(e) => {
                                    warn!(conn_id = %conn_id, "decode error: {}", e);
                                }
                            }
                        }
                        Message::Ping(_data) => {
                            // Handled by tungstenite/axum automatically
                        }
                        Message::Close(_) => {
                            info!(conn_id = %conn_id, "client sent close");
                            break;
                        }
                        _ => {}
                    }
                }
                Ok(Some(Err(e))) => {
                    warn!(conn_id = %conn_id, "ws error: {}", e);
                    break;
                }
                Ok(None) => {
                    info!(conn_id = %conn_id, "connection closed");
                    break;
                }
                Err(_) => {
                    warn!(conn_id = %conn_id, "heartbeat timeout, disconnecting");
                    break;
                }
            }
        }
    });

    // Wait for either task to finish
    tokio::select! {
        _ = write_task => {},
        _ = read_task => {},
    }

    // Step 5: Clean up
    // Notify halls this user went offline
    let halls: Vec<Uuid> = {
        let clients = state.clients.read().await;
        clients
            .get(&conn_id)
            .map(|c| c.halls.iter().copied().collect())
            .unwrap_or_default()
    };

    for hall_id in &halls {
        state
            .broadcast_to_hall(
                *hall_id,
                Some(conn_id),
                ServerMessage::MemberOffline {
                    hall_id: *hall_id,
                    user_id,
                },
            )
            .await;
    }

    // Handle voice disconnect
    let voice_info = {
        let clients = state.clients.read().await;
        clients.get(&conn_id).and_then(|c| c.voice_channel)
    };
    if let Some((hall_id, channel_id)) = voice_info {
        state
            .broadcast_to_hall(
                hall_id,
                Some(conn_id),
                ServerMessage::VoiceUserLeft {
                    hall_id,
                    channel_id,
                    user_id,
                },
            )
            .await;
    }

    state.remove_client(conn_id).await;

    info!(conn_id = %conn_id, user_id = %user_id, "client disconnected");
}

/// Wait for the first message to be an Authenticate message
async fn authenticate(
    ws_receiver: &mut futures_util::stream::SplitStream<WebSocket>,
    state: &Arc<RelayState>,
) -> Result<Uuid, ServerMessage> {
    while let Some(msg) = ws_receiver.next().await {
        match msg {
            Ok(Message::Binary(bytes)) => {
                match decode_client(&bytes) {
                    Ok(ClientMessage::Authenticate(payload)) => {
                        match validate_auth(&payload, &state.secret) {
                            Ok(user_id) => return Ok(user_id),
                            Err(e) => {
                                let (_code, msg) = match e {
                                    crate::auth::AuthError::InvalidSignature => {
                                        (ErrorCode::AuthFailed, "Invalid authentication token")
                                    }
                                    crate::auth::AuthError::TokenExpired => {
                                        (ErrorCode::TokenExpired, "Authentication token expired")
                                    }
                                    crate::auth::AuthError::VersionMismatch { .. } => {
                                        (ErrorCode::VersionMismatch, "Protocol version mismatch")
                                    }
                                };
                                return Err(ServerMessage::AuthResult {
                                    success: false,
                                    user_id: None,
                                    error: Some(msg.to_string()),
                                });
                            }
                        }
                    }
                    Ok(_) => {
                        return Err(ServerMessage::Error {
                            code: ErrorCode::AuthFailed,
                            message: "First message must be Authenticate".to_string(),
                            context: None,
                        });
                    }
                    Err(e) => {
                        return Err(ServerMessage::Error {
                            code: ErrorCode::AuthFailed,
                            message: format!("Failed to decode: {}", e),
                            context: None,
                        });
                    }
                }
            }
            Ok(Message::Close(_)) => {
                return Err(ServerMessage::Error {
                    code: ErrorCode::AuthFailed,
                    message: "Connection closed before authentication".to_string(),
                    context: None,
                });
            }
            Ok(_) => continue,
            Err(e) => {
                return Err(ServerMessage::Error {
                    code: ErrorCode::InternalError,
                    message: format!("WebSocket error: {}", e),
                    context: None,
                });
            }
        }
    }

    Err(ServerMessage::Error {
        code: ErrorCode::AuthFailed,
        message: "Connection closed".to_string(),
        context: None,
    })
}
