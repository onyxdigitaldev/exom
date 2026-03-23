/// Shared application state for the sidecar HTTP API.
///
/// Wraps the exom-core Database and HallChest in thread-safe handles,
/// along with the authenticated user's session context. All route handlers
/// receive this via Axum's State extractor.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use directories::ProjectDirs;
use uuid::Uuid;

use exom_core::{Database, HallChest};

/// Thread-safe application state shared across all route handlers.
pub struct AppState {
    pub db: Arc<Mutex<Database>>,
    pub chest: Arc<Mutex<HallChest>>,
    /// Currently authenticated user ID (set after login).
    pub current_user_id: Arc<Mutex<Option<Uuid>>>,
    /// Relay secret for generating auth tokens.
    pub relay_secret: String,
}

impl AppState {
    /// Initialize application state with default data paths.
    ///
    /// Database: `~/.local/share/exom/exom.db`
    /// Chest:   `~/.local/share/dev.onyx.exom/chests/`
    pub fn new(relay_secret: String) -> Result<Self, Box<dyn std::error::Error>> {
        let data_path = Self::data_path()?;
        std::fs::create_dir_all(&data_path)?;

        let db_path = data_path.join("exom.db");
        let db = Database::open(&db_path)?;
        let chest = HallChest::new()?;

        Ok(Self {
            db: Arc::new(Mutex::new(db)),
            chest: Arc::new(Mutex::new(chest)),
            current_user_id: Arc::new(Mutex::new(None)),
            relay_secret,
        })
    }

    /// Resolve the XDG data directory for Exom.
    fn data_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
        let dirs = ProjectDirs::from("dev", "onyx", "exom").ok_or("cannot determine data directory")?;
        Ok(dirs.data_dir().to_path_buf())
    }

    /// Get the currently authenticated user ID, or None.
    pub fn current_user(&self) -> Option<Uuid> {
        *self.current_user_id.lock().unwrap()
    }

    /// Set the authenticated user after successful login.
    pub fn set_current_user(&self, user_id: Option<Uuid>) {
        *self.current_user_id.lock().unwrap() = user_id;
    }
}
