//! Relay authentication — HMAC token validation
//!
//! Clients generate tokens locally: HMAC-SHA256(user_id + timestamp, secret)
//! The relay validates the signature and checks token freshness (5 min window).

use chrono::{Duration, Utc};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use uuid::Uuid;

use exom_protocol::AuthPayload;

type HmacSha256 = Hmac<Sha256>;

/// Token validity window
const TOKEN_EXPIRY_SECONDS: i64 = 300; // 5 minutes

/// Validate an authentication payload
pub fn validate_auth(payload: &AuthPayload, secret: &str) -> Result<Uuid, AuthError> {
    // Check protocol version
    if payload.protocol_version != exom_protocol::PROTOCOL_VERSION {
        return Err(AuthError::VersionMismatch {
            client: payload.protocol_version,
            server: exom_protocol::PROTOCOL_VERSION,
        });
    }

    // Check token freshness
    let age = Utc::now().signed_duration_since(payload.timestamp);
    if age > Duration::seconds(TOKEN_EXPIRY_SECONDS) || age < Duration::seconds(-30) {
        return Err(AuthError::TokenExpired);
    }

    // Validate HMAC signature
    let expected = generate_token(payload.user_id, payload.timestamp, secret);
    if payload.token != expected {
        return Err(AuthError::InvalidSignature);
    }

    Ok(payload.user_id)
}

/// Generate a token (used by both client and relay)
pub fn generate_token(
    user_id: Uuid,
    timestamp: chrono::DateTime<chrono::Utc>,
    secret: &str,
) -> String {
    let message = format!("{}:{}", user_id, timestamp.to_rfc3339());

    let mut mac =
        HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC accepts any key length");
    mac.update(message.as_bytes());
    let result = mac.finalize();

    base64::encode(result.into_bytes())
}

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Invalid signature")]
    InvalidSignature,

    #[error("Token expired")]
    TokenExpired,

    #[error("Protocol version mismatch: client={client}, server={server}")]
    VersionMismatch { client: u32, server: u32 },
}

// base64 encode helper (avoid pulling in the full base64 crate just for this)
mod base64 {
    pub fn encode(bytes: impl AsRef<[u8]>) -> String {
        
        let bytes = bytes.as_ref();
        const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut result = String::with_capacity((bytes.len() + 2) / 3 * 4);
        for chunk in bytes.chunks(3) {
            let b0 = chunk[0] as u32;
            let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
            let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
            let triple = (b0 << 16) | (b1 << 8) | b2;
            result.push(CHARS[((triple >> 18) & 0x3F) as usize] as char);
            result.push(CHARS[((triple >> 12) & 0x3F) as usize] as char);
            if chunk.len() > 1 {
                result.push(CHARS[((triple >> 6) & 0x3F) as usize] as char);
            } else {
                result.push('=');
            }
            if chunk.len() > 2 {
                result.push(CHARS[(triple & 0x3F) as usize] as char);
            } else {
                result.push('=');
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_valid_token() {
        let secret = "test-secret";
        let user_id = Uuid::new_v4();
        let timestamp = Utc::now();
        let token = generate_token(user_id, timestamp, secret);

        let payload = AuthPayload {
            user_id,
            token,
            timestamp,
            protocol_version: exom_protocol::PROTOCOL_VERSION,
        };

        let result = validate_auth(&payload, secret);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), user_id);
    }

    #[test]
    fn test_invalid_secret() {
        let user_id = Uuid::new_v4();
        let timestamp = Utc::now();
        let token = generate_token(user_id, timestamp, "correct-secret");

        let payload = AuthPayload {
            user_id,
            token,
            timestamp,
            protocol_version: exom_protocol::PROTOCOL_VERSION,
        };

        let result = validate_auth(&payload, "wrong-secret");
        assert!(result.is_err());
    }

    #[test]
    fn test_expired_token() {
        let secret = "test-secret";
        let user_id = Uuid::new_v4();
        let timestamp = Utc::now() - Duration::seconds(600);
        let token = generate_token(user_id, timestamp, secret);

        let payload = AuthPayload {
            user_id,
            token,
            timestamp,
            protocol_version: exom_protocol::PROTOCOL_VERSION,
        };

        let result = validate_auth(&payload, secret);
        assert!(matches!(result, Err(AuthError::TokenExpired)));
    }
}
