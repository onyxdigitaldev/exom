//! Exom Relay Server

mod auth;
mod config;
mod connection;
mod files;
mod queue;
mod ratelimit;
mod router;
mod state;

use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;

use axum::{
    extract::{
        ws::WebSocketUpgrade,
        State,
    },
    response::IntoResponse,
    routing::{get, put},
    Router,
};
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use config::RelayConfig;
use state::RelayState;

#[tokio::main]
async fn main() {
    // Load config
    let config_path = std::env::args()
        .position(|a| a == "--config")
        .and_then(|i| std::env::args().nth(i + 1))
        .unwrap_or_else(|| "relay.toml".to_string());

    let mut config = RelayConfig::load(Path::new(&config_path));
    config.apply_env();

    // Handle --init flag to generate default config
    if std::env::args().any(|a| a == "--init") {
        RelayConfig::write_default(Path::new("relay.toml")).expect("failed to write config");
        println!("Default config written to relay.toml");
        return;
    }

    // Initialize logging
    let level = match config.log_level.as_str() {
        "trace" => Level::TRACE,
        "debug" => Level::DEBUG,
        "warn" => Level::WARN,
        "error" => Level::ERROR,
        _ => Level::INFO,
    };
    let subscriber = FmtSubscriber::builder().with_max_level(level).finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting tracing subscriber");

    // Initialize relay state
    let state = Arc::new(
        RelayState::new(&config.db_path, config.secret.clone())
            .expect("failed to initialize relay"),
    );

    // Spawn periodic cleanup task
    let cleanup_state = state.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(3600));
        loop {
            interval.tick().await;
            if let Ok(count) = cleanup_state.queue.lock().await.cleanup() {
                if count > 0 {
                    info!(count = count, "cleaned up expired queue messages");
                }
            }
            cleanup_state.cleanup_rate_limits().await;
        }
    });

    let app = Router::new()
        .route("/ws", get(ws_handler))
        .route("/upload/{filename}", put(files::upload_file))
        .route("/files/{hash}/{filename}", get(files::download_file))
        .route("/health", get(health))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    info!(port = config.port, "Exom relay listening");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    // Graceful shutdown on SIGTERM
    let shutdown = async {
        tokio::signal::ctrl_c().await.ok();
        info!("shutting down relay");
    };

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown)
        .await
        .unwrap();
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
