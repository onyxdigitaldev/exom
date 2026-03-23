//! Exom Wire Protocol
//!
//! Defines all message types exchanged between clients and the relay.
//! Serialized with MessagePack (rmp-serde) for compact binary transport.

pub mod messages;
pub mod codec;

pub use messages::*;
pub use codec::*;
