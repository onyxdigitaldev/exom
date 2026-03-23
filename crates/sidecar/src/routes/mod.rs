/// Route modules for the sidecar HTTP API.
///
/// Each module handles a domain of exom-core operations,
/// exposing them as JSON endpoints for the Electron renderer.

pub mod auth;
pub mod channels;
pub mod discovery;
pub mod dms;
pub mod halls;
pub mod invites;
pub mod members;
pub mod messages;
pub mod moderation;
pub mod notifications;
pub mod profiles;
pub mod reactions;
pub mod relay_token;
