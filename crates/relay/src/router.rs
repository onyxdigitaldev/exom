//! Message router — handles incoming client messages and produces responses
//!
//! This is the brain of the relay. It receives decoded ClientMessages,
//! performs routing logic, and emits ServerMessages to the right recipients.

use std::sync::Arc;

use chrono::Utc;
use tracing::{info, warn};
use uuid::Uuid;

use exom_protocol::{ClientMessage, ErrorCode, ServerMessage};

use crate::state::RelayState;

/// Process a client message and route it
pub async fn route(
    state: &Arc<RelayState>,
    conn_id: Uuid,
    user_id: Uuid,
    msg: ClientMessage,
) {
    match msg {
        ClientMessage::Heartbeat => {
            send_to_conn(state, conn_id, ServerMessage::HeartbeatAck {
                server_time: Utc::now(),
            })
            .await;
        }

        ClientMessage::HallJoin { hall_id } => {
            state.join_hall(conn_id, hall_id).await;

            let online = state.online_members_in_hall(hall_id).await;

            // Tell the joining client who's online
            send_to_conn(state, conn_id, ServerMessage::HallJoined {
                hall_id,
                online_members: online.clone(),
            })
            .await;

            // Tell everyone else this user came online
            state
                .broadcast_to_hall(
                    hall_id,
                    Some(conn_id),
                    ServerMessage::MemberOnline {
                        hall_id,
                        user_id,
                    },
                )
                .await;

            info!(user_id = %user_id, hall_id = %hall_id, "user joined hall");
        }

        ClientMessage::HallLeave { hall_id } => {
            state.leave_hall(conn_id, hall_id).await;

            send_to_conn(state, conn_id, ServerMessage::HallLeft { hall_id }).await;

            state
                .broadcast_to_hall(
                    hall_id,
                    None,
                    ServerMessage::MemberOffline {
                        hall_id,
                        user_id,
                    },
                )
                .await;

            info!(user_id = %user_id, hall_id = %hall_id, "user left hall");
        }

        ClientMessage::ChannelMessage {
            hall_id,
            channel_id,
            message,
        } => {
            if !is_in_hall(state, conn_id, hall_id).await {
                send_error(state, conn_id, ErrorCode::NotInHall, "Not subscribed to this hall").await;
                return;
            }

            // Broadcast to all hall members (including sender for confirmation)
            let server_msg = ServerMessage::ChannelMessage {
                hall_id,
                channel_id,
                sender_id: user_id,
                message: message.clone(),
            };

            // Send to all online members
            state.broadcast_to_hall(hall_id, Some(conn_id), server_msg.clone()).await;

            // Queue for offline members
            queue_for_offline_hall_members(state, hall_id, conn_id, &server_msg).await;
        }

        ClientMessage::MessageEdit {
            hall_id,
            channel_id,
            message_id,
            new_content,
        } => {
            if !is_in_hall(state, conn_id, hall_id).await {
                send_error(state, conn_id, ErrorCode::NotInHall, "Not subscribed to this hall").await;
                return;
            }

            state
                .broadcast_to_hall(
                    hall_id,
                    None,
                    ServerMessage::MessageEdited {
                        hall_id,
                        channel_id,
                        message_id,
                        new_content,
                        edited_at: Utc::now(),
                    },
                )
                .await;
        }

        ClientMessage::MessageDelete {
            hall_id,
            channel_id,
            message_id,
        } => {
            if !is_in_hall(state, conn_id, hall_id).await {
                send_error(state, conn_id, ErrorCode::NotInHall, "Not subscribed to this hall").await;
                return;
            }

            state
                .broadcast_to_hall(
                    hall_id,
                    None,
                    ServerMessage::MessageDeleted {
                        hall_id,
                        channel_id,
                        message_id,
                    },
                )
                .await;
        }

        ClientMessage::ReactionAdd {
            hall_id,
            channel_id,
            message_id,
            emoji,
            is_custom,
        } => {
            if !is_in_hall(state, conn_id, hall_id).await {
                return;
            }

            state
                .broadcast_to_hall(
                    hall_id,
                    None,
                    ServerMessage::ReactionAdded {
                        hall_id,
                        channel_id,
                        message_id,
                        user_id,
                        emoji,
                        is_custom,
                    },
                )
                .await;
        }

        ClientMessage::ReactionRemove {
            hall_id,
            channel_id,
            message_id,
            emoji,
        } => {
            if !is_in_hall(state, conn_id, hall_id).await {
                return;
            }

            state
                .broadcast_to_hall(
                    hall_id,
                    None,
                    ServerMessage::ReactionRemoved {
                        hall_id,
                        channel_id,
                        message_id,
                        user_id,
                        emoji,
                    },
                )
                .await;
        }

        ClientMessage::TypingStart {
            hall_id,
            channel_id,
        } => {
            // Ephemeral — never queued
            if !is_in_hall(state, conn_id, hall_id).await {
                return;
            }

            state
                .broadcast_to_hall(
                    hall_id,
                    Some(conn_id),
                    ServerMessage::TypingStarted {
                        hall_id,
                        channel_id,
                        user_id,
                    },
                )
                .await;
        }

        ClientMessage::PresenceUpdate {
            status,
            custom_text,
        } => {
            // Broadcast to all halls this user is in
            let clients = state.clients.read().await;
            let halls: Vec<Uuid> = clients
                .get(&conn_id)
                .map(|c| c.halls.iter().copied().collect())
                .unwrap_or_default();
            drop(clients);

            let msg = ServerMessage::PresenceUpdated {
                user_id,
                status,
                custom_text,
            };

            for hall_id in halls {
                state.broadcast_to_hall(hall_id, Some(conn_id), msg.clone()).await;
            }
        }

        ClientMessage::MarkRead {
            hall_id: _,
            channel_id: _,
            message_id: _,
        } => {
            // Read receipts are processed client-side, relay just acks
            // No broadcast needed
        }

        ClientMessage::MessagePin {
            hall_id,
            channel_id,
            message_id,
        } => {
            if !is_in_hall(state, conn_id, hall_id).await {
                return;
            }

            state
                .broadcast_to_hall(
                    hall_id,
                    None,
                    ServerMessage::MessagePinned {
                        hall_id,
                        channel_id,
                        message_id,
                        pinned_by: user_id,
                    },
                )
                .await;
        }

        ClientMessage::MessageUnpin {
            hall_id,
            channel_id,
            message_id,
        } => {
            if !is_in_hall(state, conn_id, hall_id).await {
                return;
            }

            state
                .broadcast_to_hall(
                    hall_id,
                    None,
                    ServerMessage::MessageUnpinned {
                        hall_id,
                        channel_id,
                        message_id,
                    },
                )
                .await;
        }

        // --- Voice signaling ---

        ClientMessage::VoiceJoin {
            hall_id,
            channel_id,
        } => {
            state.join_voice(conn_id, user_id, hall_id, channel_id).await;

            // Notify all hall members
            state
                .broadcast_to_hall(
                    hall_id,
                    Some(conn_id),
                    ServerMessage::VoiceUserJoined {
                        hall_id,
                        channel_id,
                        user_id,
                    },
                )
                .await;

            info!(user_id = %user_id, channel_id = %channel_id, "user joined voice");
        }

        ClientMessage::VoiceLeave => {
            // Get current voice channel before leaving
            let voice_info = {
                let clients = state.clients.read().await;
                clients.get(&conn_id).and_then(|c| c.voice_channel)
            };

            state.leave_voice(conn_id, user_id).await;

            if let Some((hall_id, channel_id)) = voice_info {
                state
                    .broadcast_to_hall(
                        hall_id,
                        None,
                        ServerMessage::VoiceUserLeft {
                            hall_id,
                            channel_id,
                            user_id,
                        },
                    )
                    .await;
            }
        }

        ClientMessage::VoiceStateUpdate {
            self_mute,
            self_deaf,
            video,
            streaming,
        } => {
            let voice_info = {
                let clients = state.clients.read().await;
                clients.get(&conn_id).and_then(|c| c.voice_channel)
            };

            if let Some((hall_id, channel_id)) = voice_info {
                state
                    .broadcast_to_hall(
                        hall_id,
                        Some(conn_id),
                        ServerMessage::VoiceStateUpdated {
                            hall_id,
                            channel_id,
                            user_id,
                            self_mute,
                            self_deaf,
                            video,
                            streaming,
                        },
                    )
                    .await;
            }
        }

        ClientMessage::VoiceOffer {
            target_user_id,
            sdp,
        } => {
            state
                .send_to_user(
                    target_user_id,
                    ServerMessage::VoiceOffer {
                        from_user_id: user_id,
                        sdp,
                    },
                )
                .await;
        }

        ClientMessage::VoiceAnswer {
            target_user_id,
            sdp,
        } => {
            state
                .send_to_user(
                    target_user_id,
                    ServerMessage::VoiceAnswer {
                        from_user_id: user_id,
                        sdp,
                    },
                )
                .await;
        }

        ClientMessage::VoiceIceCandidate {
            target_user_id,
            candidate,
        } => {
            state
                .send_to_user(
                    target_user_id,
                    ServerMessage::VoiceIceCandidate {
                        from_user_id: user_id,
                        candidate,
                    },
                )
                .await;
        }

        // --- DMs ---

        ClientMessage::DirectMessage {
            channel_id,
            message,
        } => {
            // DMs go directly to the target user(s)
            // The client knows the participants; relay just forwards
            // For now, broadcast to the channel_id as a "room"
            // The client-side DM system handles participant resolution
            let _msg = ServerMessage::DirectMessage {
                channel_id,
                sender_id: user_id,
                message,
            };
            // DM delivery is user-to-user, not hall-based
            // Queue if offline (the client provides target user IDs in a future extension)
            // For now this is a placeholder — full DM routing requires participant lookup
            warn!("DM routing not yet fully implemented");
        }

        // --- Sync ---

        ClientMessage::SyncRequest { hall_id, since: _ } => {
            // Sync is handled client-side via local SQLite
            // The relay only provides queued messages that were missed
            // Send empty sync response (client pulls from local DB)
            send_to_conn(
                state,
                conn_id,
                ServerMessage::SyncResponse {
                    hall_id,
                    events: Vec::new(),
                },
            )
            .await;
        }

        ClientMessage::QueueDrain => {
            let messages = {
                let queue = state.queue.lock().await;
                queue.drain(user_id).unwrap_or_default()
            };

            if !messages.is_empty() {
                info!(user_id = %user_id, count = messages.len(), "draining offline queue");
                send_to_conn(
                    state,
                    conn_id,
                    ServerMessage::QueuedMessages { messages },
                )
                .await;
            }
        }

        _ => {
            send_error(state, conn_id, ErrorCode::UnknownMessage, "Unknown message type").await;
        }
    }
}

/// Check if a connection is subscribed to a hall
async fn is_in_hall(state: &Arc<RelayState>, conn_id: Uuid, hall_id: Uuid) -> bool {
    let clients = state.clients.read().await;
    clients
        .get(&conn_id)
        .map(|c| c.halls.contains(&hall_id))
        .unwrap_or(false)
}

/// Send a message to a specific connection
async fn send_to_conn(state: &Arc<RelayState>, conn_id: Uuid, msg: ServerMessage) {
    let clients = state.clients.read().await;
    if let Some(client) = clients.get(&conn_id) {
        let _ = client.sender.send(msg);
    }
}

/// Send an error to a connection
async fn send_error(
    state: &Arc<RelayState>,
    conn_id: Uuid,
    code: ErrorCode,
    message: &str,
) {
    send_to_conn(
        state,
        conn_id,
        ServerMessage::Error {
            code,
            message: message.to_string(),
            context: None,
        },
    )
    .await;
}

/// Queue a message for all offline members of a hall
async fn queue_for_offline_hall_members(
    _state: &Arc<RelayState>,
    _hall_id: Uuid,
    _exclude_conn: Uuid,
    _msg: &ServerMessage,
) {
    // This is a simplified version — in production, we'd need the full
    // member list from the authoritative source (the host client).
    // For now, the relay only queues for users it has seen connect before.
    // TODO: Implement hall member registry at relay level
}
