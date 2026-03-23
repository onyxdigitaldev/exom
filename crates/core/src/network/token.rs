//! Auth token generation for relay authentication

use chrono::Utc;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use uuid::Uuid;

use exom_protocol::{AuthPayload, PROTOCOL_VERSION};

type HmacSha256 = Hmac<Sha256>;

/// Generate an authentication payload for connecting to the relay
pub fn generate_auth_token(user_id: Uuid, secret: &str) -> AuthPayload {
    let timestamp = Utc::now();
    let message = format!("{}:{}", user_id, timestamp.to_rfc3339());

    let mut mac =
        HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC accepts any key length");
    mac.update(message.as_bytes());
    let result = mac.finalize();

    let token = base64_encode(result.into_bytes());

    AuthPayload {
        user_id,
        token,
        timestamp,
        protocol_version: PROTOCOL_VERSION,
    }
}

fn base64_encode(bytes: impl AsRef<[u8]>) -> String {
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
