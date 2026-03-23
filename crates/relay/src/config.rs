//! Relay configuration

use serde::{Deserialize, Serialize};
use std::path::Path;

/// Relay server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelayConfig {
    /// Listen port
    pub port: u16,
    /// SQLite database path for message queue
    pub db_path: String,
    /// HMAC secret for auth tokens
    pub secret: String,
    /// File storage directory
    pub file_storage_path: String,
    /// Maximum file upload size in bytes
    pub max_file_size: usize,
    /// Queue message TTL in days
    pub queue_ttl_days: u32,
    /// Log level (trace, debug, info, warn, error)
    pub log_level: String,
}

impl Default for RelayConfig {
    fn default() -> Self {
        Self {
            port: 9400,
            db_path: "relay.db".to_string(),
            secret: "change-me-in-production".to_string(),
            file_storage_path: "data/files".to_string(),
            max_file_size: 25 * 1024 * 1024,
            queue_ttl_days: 7,
            log_level: "info".to_string(),
        }
    }
}

impl RelayConfig {
    /// Load config from a TOML file, falling back to defaults
    pub fn load(path: &Path) -> Self {
        if path.exists() {
            match std::fs::read_to_string(path) {
                Ok(contents) => match toml_parse(&contents) {
                    Some(config) => return config,
                    None => {
                        eprintln!("Warning: failed to parse config file, using defaults");
                    }
                },
                Err(e) => {
                    eprintln!("Warning: failed to read config file: {}", e);
                }
            }
        }
        Self::default()
    }

    /// Apply environment variable overrides
    pub fn apply_env(&mut self) {
        if let Ok(v) = std::env::var("EXOM_RELAY_PORT") {
            if let Ok(port) = v.parse() {
                self.port = port;
            }
        }
        if let Ok(v) = std::env::var("EXOM_RELAY_SECRET") {
            self.secret = v;
        }
        if let Ok(v) = std::env::var("EXOM_RELAY_DB") {
            self.db_path = v;
        }
        if let Ok(v) = std::env::var("EXOM_RELAY_FILES") {
            self.file_storage_path = v;
        }
        if let Ok(v) = std::env::var("EXOM_RELAY_LOG") {
            self.log_level = v;
        }
    }

    /// Generate a default config file
    pub fn write_default(path: &Path) -> std::io::Result<()> {
        let config = Self::default();
        let contents = format!(
            r#"# Exom Relay Configuration

port = {}
db_path = "{}"
secret = "{}"
file_storage_path = "{}"
max_file_size = {}
queue_ttl_days = {}
log_level = "{}"
"#,
            config.port,
            config.db_path,
            config.secret,
            config.file_storage_path,
            config.max_file_size,
            config.queue_ttl_days,
            config.log_level,
        );
        std::fs::write(path, contents)
    }
}

/// Simple TOML parser (avoids pulling in the toml crate for basic key=value)
fn toml_parse(contents: &str) -> Option<RelayConfig> {
    let mut config = RelayConfig::default();

    for line in contents.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut parts = line.splitn(2, '=');
        let key = parts.next()?.trim();
        let value = parts.next()?.trim().trim_matches('"');

        match key {
            "port" => config.port = value.parse().ok()?,
            "db_path" => config.db_path = value.to_string(),
            "secret" => config.secret = value.to_string(),
            "file_storage_path" => config.file_storage_path = value.to_string(),
            "max_file_size" => config.max_file_size = value.parse().ok()?,
            "queue_ttl_days" => config.queue_ttl_days = value.parse().ok()?,
            "log_level" => config.log_level = value.to_string(),
            _ => {}
        }
    }

    Some(config)
}
