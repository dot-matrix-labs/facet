pub mod logger;
/// Encrypted logging system for Facet
///
/// This module provides encrypted log file storage that integrates with the user profile system.
/// All logs are encrypted using the same encryption system as other user files.
///
/// Features:
/// - Encrypted log file storage
/// - Log rotation (max 10MB per file, keeps last 3 files)
/// - Captures both Rust and frontend logs
/// - Thread-safe logging
mod storage;

pub use logger::log_frontend_message;
pub use storage::{LogEntry, LogLevel, LogStorage};

use once_cell::sync::Lazy;
use std::sync::Mutex;

/// Global log storage instance
static LOG_STORAGE: Lazy<Mutex<Option<LogStorage>>> = Lazy::new(|| Mutex::new(None));

/// Initialize the logging system for a specific user
pub fn init_for_user(username: &str, password: &str) -> Result<(), String> {
    let storage = LogStorage::new(username, password)
        .map_err(|e| format!("Failed to initialize log storage: {}", e))?;

    *LOG_STORAGE.lock().unwrap() = Some(storage);

    log::info!("Initialized encrypted logging for user: {}", username);
    Ok(())
}

/// Cleanup logging system (called on logout)
pub fn cleanup() {
    *LOG_STORAGE.lock().unwrap() = None;
    log::info!("Cleaned up encrypted logging");
}

/// Get reference to log storage
pub(crate) fn get_storage() -> Option<LogStorage> {
    LOG_STORAGE.lock().unwrap().clone()
}
