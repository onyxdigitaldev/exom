/// Exom Sidecar — local HTTP API wrapping exom-core.
///
/// Runs on localhost:9401. The Electron renderer communicates with this
/// process for all database operations. Real-time events flow directly
/// from the renderer to the relay via WebSocket — the sidecar is not
/// involved in real-time message routing.

mod routes;
pub mod validation;
mod state;

use std::net::SocketAddr;
use std::sync::Arc;

use axum::{routing::{delete, get, post, put}, Router};
use tower_http::cors::{Any, CorsLayer};
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use state::AppState;

#[tokio::main]
async fn main() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting tracing subscriber");

    let port: u16 = std::env::args()
        .nth(1)
        .and_then(|p| p.parse().ok())
        .unwrap_or(9401);

    let relay_secret = std::env::var("EXOM_RELAY_SECRET")
        .unwrap_or_else(|_| "dev-secret-change-me".to_string());

    let state = Arc::new(
        AppState::new(relay_secret).expect("failed to initialize sidecar state"),
    );

    // CORS: Only allow requests from the local Electron renderer.
    // In production, this should be locked to file:// or the app's origin.
    let cors = CorsLayer::new()
        .allow_origin("http://localhost:5173".parse::<axum::http::HeaderValue>().unwrap())
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        // Auth
        .route("/api/auth/login", post(routes::auth::login))
        .route("/api/auth/register", post(routes::auth::register))
        .route("/api/auth/logout", post(routes::auth::logout))
        .route("/api/auth/me", get(routes::auth::current_user))
        // Halls
        .route("/api/halls", get(routes::halls::list_halls))
        .route("/api/halls", post(routes::halls::create_hall))
        .route("/api/halls/:id", get(routes::halls::get_hall))
        .route("/api/halls/:id/leave", delete(routes::halls::leave_hall))
        // Channels
        .route("/api/halls/:hall_id/channels", get(routes::channels::list_channels))
        .route("/api/halls/:hall_id/channels", post(routes::channels::create_channel))
        .route("/api/channels/:id", delete(routes::channels::delete_channel))
        // Messages
        .route("/api/halls/:hall_id/channels/:channel_id/messages", get(routes::messages::list_messages))
        .route("/api/halls/:hall_id/channels/:channel_id/messages", post(routes::messages::send_message))
        .route("/api/messages/:id", put(routes::messages::edit_message))
        .route("/api/messages/:id", delete(routes::messages::delete_message))
        .route("/api/messages/:id/pin", post(routes::messages::pin_message))
        .route("/api/messages/:id/pin", delete(routes::messages::unpin_message))
        // Members
        .route("/api/halls/:hall_id/members", get(routes::members::list_members))
        .route("/api/halls/:hall_id/members/:user_id/role", put(routes::members::update_role))
        .route("/api/halls/:hall_id/members/:user_id", delete(routes::members::kick_member))
        // Invites
        .route("/api/halls/:hall_id/invites", post(routes::invites::create_invite))
        .route("/api/halls/:hall_id/invites", get(routes::invites::list_invites))
        .route("/api/invites/:token/accept", post(routes::invites::accept_invite))
        .route("/api/invites/:id", delete(routes::invites::revoke_invite))
        // Profiles
        .route("/api/profiles/:user_id", get(routes::profiles::get_profile))
        .route("/api/profiles/me", put(routes::profiles::update_profile))
        // DMs
        .route("/api/dms", post(routes::dms::create_dm))
        .route("/api/dms", get(routes::dms::list_dms))
        .route("/api/dms/:channel_id/messages", get(routes::dms::list_dm_messages))
        .route("/api/dms/:channel_id/messages", post(routes::dms::send_dm))
        // Reactions
        .route("/api/messages/:message_id/reactions", post(routes::reactions::add_reaction))
        .route("/api/messages/:message_id/reactions/:emoji", delete(routes::reactions::remove_reaction))
        .route("/api/messages/:message_id/reactions", get(routes::reactions::list_reactions))
        // Moderation
        .route("/api/halls/:hall_id/bans", post(routes::moderation::ban_user))
        .route("/api/halls/:hall_id/bans/:user_id", delete(routes::moderation::unban_user))
        .route("/api/halls/:hall_id/bans", get(routes::moderation::list_bans))
        .route("/api/halls/:hall_id/audit-log", get(routes::moderation::list_audit_log))
        // Notifications
        .route("/api/channels/:channel_id/read", post(routes::notifications::mark_read))
        .route("/api/notifications/unread", get(routes::notifications::list_unread))
        // Discovery
        .route("/api/discovery", get(routes::discovery::search_halls))
        // Relay token
        .route("/api/relay/token", post(routes::relay_token::generate_token))
        // Health
        .route("/health", get(health))
        .layer(cors)
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));

    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(e) if e.kind() == std::io::ErrorKind::AddrInUse => {
            info!(port = port, "Port already in use — sidecar may already be running");
            std::process::exit(0);
        }
        Err(e) => {
            tracing::error!("Failed to bind port {}: {}", port, e);
            std::process::exit(1);
        }
    };

    info!(port = port, "Exom sidecar listening");

    let shutdown = async {
        tokio::signal::ctrl_c().await.ok();
        info!("sidecar shutting down");
    };

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown)
        .await
        .unwrap();
}

async fn health() -> &'static str {
    "ok"
}
