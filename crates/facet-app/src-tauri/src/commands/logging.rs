use crate::logging::{LogEntry, LogLevel};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntryDto {
    pub timestamp: String,
    pub level: String,
    pub source: String,
    pub message: String,
}

impl From<LogEntry> for LogEntryDto {
    fn from(entry: LogEntry) -> Self {
        Self {
            timestamp: entry.timestamp,
            level: entry.level.to_string(),
            source: entry.source,
            message: entry.message,
        }
    }
}

/// Log a message from the frontend
#[tauri::command]
pub async fn log_frontend_message(level: String, message: String) -> Result<(), String> {
    let log_level = match level.to_lowercase().as_str() {
        "debug" => LogLevel::Debug,
        "info" => LogLevel::Info,
        "warn" => LogLevel::Warn,
        "error" => LogLevel::Error,
        _ => LogLevel::Info,
    };

    crate::logging::log_frontend_message(log_level, message)
}

/// Get all log entries
#[tauri::command]
pub async fn get_logs() -> Result<Vec<LogEntryDto>, String> {
    let storage = crate::logging::get_storage()
        .ok_or_else(|| "Logging not initialized. Please log in.".to_string())?;

    let logs = storage
        .read_logs()
        .map_err(|e| format!("Failed to read logs: {}", e))?;

    Ok(logs.into_iter().map(LogEntryDto::from).collect())
}

/// Clear all logs
#[tauri::command]
pub async fn clear_logs() -> Result<(), String> {
    let storage = crate::logging::get_storage()
        .ok_or_else(|| "Logging not initialized. Please log in.".to_string())?;

    storage
        .clear_logs()
        .map_err(|e| format!("Failed to clear logs: {}", e))?;

    Ok(())
}

/// Get log file size in bytes
#[tauri::command]
pub async fn get_log_size() -> Result<u64, String> {
    let storage = crate::logging::get_storage()
        .ok_or_else(|| "Logging not initialized. Please log in.".to_string())?;

    storage
        .get_log_size()
        .map_err(|e| format!("Failed to get log size: {}", e))
}
