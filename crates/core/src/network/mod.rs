//! Network client — connects to the relay server
//!
//! Provides the RelayClient that the app layer uses to send/receive
//! real-time events through the relay.

mod client;
mod token;

pub use client::{RelayClient, RelayClientConfig, ConnectionState};
pub use token::generate_auth_token;
