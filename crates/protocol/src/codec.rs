//! Wire codec — MessagePack serialization/deserialization
//!
//! Messages are framed as binary WebSocket messages containing
//! MessagePack-encoded ClientMessage or ServerMessage.

use crate::messages::{ClientMessage, ServerMessage};

#[derive(Debug, thiserror::Error)]
pub enum CodecError {
    #[error("Encode error: {0}")]
    Encode(#[from] rmp_serde::encode::Error),

    #[error("Decode error: {0}")]
    Decode(#[from] rmp_serde::decode::Error),

    #[error("Message too large: {size} bytes (max {max})")]
    TooLarge { size: usize, max: usize },
}

/// Maximum message size (1 MB)
pub const MAX_MESSAGE_SIZE: usize = 1024 * 1024;

/// Encode a ClientMessage to binary (MessagePack)
pub fn encode_client(msg: &ClientMessage) -> Result<Vec<u8>, CodecError> {
    let bytes = rmp_serde::to_vec(msg)?;
    if bytes.len() > MAX_MESSAGE_SIZE {
        return Err(CodecError::TooLarge {
            size: bytes.len(),
            max: MAX_MESSAGE_SIZE,
        });
    }
    Ok(bytes)
}

/// Decode binary to a ClientMessage
pub fn decode_client(bytes: &[u8]) -> Result<ClientMessage, CodecError> {
    if bytes.len() > MAX_MESSAGE_SIZE {
        return Err(CodecError::TooLarge {
            size: bytes.len(),
            max: MAX_MESSAGE_SIZE,
        });
    }
    Ok(rmp_serde::from_slice(bytes)?)
}

/// Encode a ServerMessage to binary (MessagePack)
pub fn encode_server(msg: &ServerMessage) -> Result<Vec<u8>, CodecError> {
    let bytes = rmp_serde::to_vec(msg)?;
    if bytes.len() > MAX_MESSAGE_SIZE {
        return Err(CodecError::TooLarge {
            size: bytes.len(),
            max: MAX_MESSAGE_SIZE,
        });
    }
    Ok(bytes)
}

/// Decode binary to a ServerMessage
pub fn decode_server(bytes: &[u8]) -> Result<ServerMessage, CodecError> {
    if bytes.len() > MAX_MESSAGE_SIZE {
        return Err(CodecError::TooLarge {
            size: bytes.len(),
            max: MAX_MESSAGE_SIZE,
        });
    }
    Ok(rmp_serde::from_slice(bytes)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::messages::MessagePayload;
    use uuid::Uuid;

    #[test]
    fn test_client_roundtrip() {
        let msg = ClientMessage::ChannelMessage {
            hall_id: Uuid::new_v4(),
            channel_id: Uuid::new_v4(),
            message: MessagePayload::new("hello world".into()),
        };

        let bytes = encode_client(&msg).unwrap();
        let decoded = decode_client(&bytes).unwrap();

        match decoded {
            ClientMessage::ChannelMessage { message, .. } => {
                assert_eq!(message.content, "hello world");
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn test_server_roundtrip() {
        let msg = ServerMessage::ChannelMessage {
            hall_id: Uuid::new_v4(),
            channel_id: Uuid::new_v4(),
            sender_id: Uuid::new_v4(),
            message: MessagePayload::new("test".into()),
        };

        let bytes = encode_server(&msg).unwrap();
        let decoded = decode_server(&bytes).unwrap();

        match decoded {
            ServerMessage::ChannelMessage { message, .. } => {
                assert_eq!(message.content, "test");
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn test_heartbeat_small() {
        let msg = ClientMessage::Heartbeat;
        let bytes = encode_client(&msg).unwrap();
        // Heartbeat should be tiny
        assert!(bytes.len() < 32);
    }
}
