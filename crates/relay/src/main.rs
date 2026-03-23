//! Exom Relay Server
//!
//! Thin WebSocket relay that routes messages between clients.
//! Does NOT store message history or run business logic.
//! Responsibilities:
//!   - WebSocket connection management
//!   - Route messages between Hall members
//!   - Track presence (online/offline)
//!   - Queue messages for offline users
//!   - Forward WebRTC signaling for voice/video

mod auth;
mod connection;
mod queue;
mod router;
mod state;

use std::net::SocketAddr;
use std::sync::Arc;

use axum::{
    extract::{
        ws::WebSocketUpgrade,
        State,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use state::RelayState;

#[tokio::main]
async fn main() {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting tracing subscriber");

    // Parse CLI args
    let port: u16 = std::env::args()
        .nth(1)
        .and_then(|p| p.parse().ok())
        .unwrap_or(9400);

    let db_path = std::env::args()
        .nth(2)
        .unwrap_or_else(|| "relay.db".to_string());

    let secret = std::env::var("EXOM_RELAY_SECRET")
        .unwrap_or_else(|_| "dev-secret-change-me".to_string());

    // Initialize relay state
    let state = Arc::new(RelayState::new(&db_path, secret).expect("failed to initialize relay"));

    let app = Router::new()
        .route("/ws", get(ws_handler))
        .route("/health", get(health))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("Exom relay listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<RelayState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| connection::handle_connection(socket, state))
}

async fn health() -> &'static str {
    "ok"
}
