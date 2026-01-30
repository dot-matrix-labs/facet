use crate::profiles::crypto::{decrypt_file, derive_key, encrypt_file};
use crate::profiles::storage::get_user_dir;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::PathBuf;
use thiserror::Error;

/// Maximum log file size before rotation (10MB)
const MAX_LOG_SIZE: u64 = 10 * 1024 * 1024;

/// Number of rotated log files to keep
const MAX_LOG_FILES: usize = 3;

/// Log file name
const LOG_FILE_NAME: &str = "debug.log";

#[derive(Error, Debug)]
pub enum LogError {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("Crypto error: {0}")]
    Crypto(#[from] crate::profiles::crypto::CryptoError),

    #[error("Storage error: {0}")]
    Storage(#[from] crate::profiles::storage::StorageError),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, LogError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

impl From<log::Level> for LogLevel {
    fn from(level: log::Level) -> Self {
        match level {
            log::Level::Error => LogLevel::Error,
            log::Level::Warn => LogLevel::Warn,
            log::Level::Info => LogLevel::Info,
            log::Level::Debug | log::Level::Trace => LogLevel::Debug,
        }
    }
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::Debug => write!(f, "DEBUG"),
            LogLevel::Info => write!(f, "INFO"),
            LogLevel::Warn => write!(f, "WARN"),
            LogLevel::Error => write!(f, "ERROR"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: LogLevel,
    pub source: String, // "rust" or "frontend"
    pub message: String,
}

impl LogEntry {
    pub fn new(level: LogLevel, source: String, message: String) -> Self {
        Self {
            timestamp: chrono::Local::now().to_rfc3339(),
            level,
            source,
            message,
        }
    }
}

#[derive(Clone)]
pub struct LogStorage {
    username: String,
    encryption_key: crate::profiles::crypto::EncryptionKey,
    log_file_path: PathBuf,
}

impl LogStorage {
    /// Create a new log storage for a user
    pub fn new(username: &str, password: &str) -> Result<Self> {
        // Load salt for user
        let salt = crate::profiles::storage::load_salt(username, None)?;

        // Derive encryption key from password
        let (encryption_key, _) = derive_key(password, Some(&salt))?;

        // Get log file path
        let user_dir = get_user_dir(username, None)?;
        let log_file_path = user_dir.join(LOG_FILE_NAME);

        Ok(Self {
            username: username.to_string(),
            encryption_key,
            log_file_path,
        })
    }

    /// Append a log entry to the encrypted log file
    pub fn append_log(&self, entry: LogEntry) -> Result<()> {
        // Load existing logs
        let mut logs = self.read_logs()?;

        // Append new entry
        logs.push(entry);

        // Check if rotation is needed
        self.rotate_if_needed()?;

        // Write logs back
        self.write_logs(&logs)?;

        Ok(())
    }

    /// Read all log entries from the encrypted log file
    pub fn read_logs(&self) -> Result<Vec<LogEntry>> {
        if !self.log_file_path.exists() {
            return Ok(Vec::new());
        }

        // Read encrypted file
        let encrypted = fs::read(&self.log_file_path)?;

        if encrypted.is_empty() {
            return Ok(Vec::new());
        }

        // Decrypt
        let decrypted = decrypt_file(&encrypted, &self.encryption_key)?;

        // Parse JSON (each line is a JSON object)
        let content = String::from_utf8_lossy(&decrypted);
        let mut logs = Vec::new();

        for line in content.lines() {
            if line.trim().is_empty() {
                continue;
            }

            match serde_json::from_str::<LogEntry>(line) {
                Ok(entry) => logs.push(entry),
                Err(e) => {
                    // Log parsing error but continue
                    eprintln!("Failed to parse log entry: {}", e);
                }
            }
        }

        Ok(logs)
    }

    /// Write logs to encrypted file
    fn write_logs(&self, logs: &[LogEntry]) -> Result<()> {
        // Serialize logs (one JSON per line)
        let mut content = String::new();
        for log in logs {
            content.push_str(&serde_json::to_string(log)?);
            content.push('\n');
        }

        // Encrypt
        let encrypted = encrypt_file(content.as_bytes(), &self.encryption_key)?;

        // Write to file
        fs::write(&self.log_file_path, encrypted)?;

        Ok(())
    }

    /// Rotate log files if current file exceeds max size
    fn rotate_if_needed(&self) -> Result<()> {
        if !self.log_file_path.exists() {
            return Ok(());
        }

        let metadata = fs::metadata(&self.log_file_path)?;
        if metadata.len() < MAX_LOG_SIZE {
            return Ok(());
        }

        log::info!("Rotating log file for user: {}", self.username);

        // Shift existing rotated files
        for i in (1..MAX_LOG_FILES).rev() {
            let old_path = self.log_file_path.with_extension(format!("log.{}", i));
            let new_path = self.log_file_path.with_extension(format!("log.{}", i + 1));

            if old_path.exists() {
                if i + 1 >= MAX_LOG_FILES {
                    // Delete oldest file
                    fs::remove_file(&old_path)?;
                } else {
                    // Rename to next rotation
                    fs::rename(&old_path, &new_path)?;
                }
            }
        }

        // Move current log to .log.1
        let rotated_path = self.log_file_path.with_extension("log.1");
        fs::rename(&self.log_file_path, &rotated_path)?;

        Ok(())
    }

    /// Clear all logs for this user
    pub fn clear_logs(&self) -> Result<()> {
        // Remove main log file
        if self.log_file_path.exists() {
            fs::remove_file(&self.log_file_path)?;
        }

        // Remove rotated log files
        for i in 1..=MAX_LOG_FILES {
            let rotated_path = self.log_file_path.with_extension(format!("log.{}", i));
            if rotated_path.exists() {
                fs::remove_file(&rotated_path)?;
            }
        }

        log::info!("Cleared all logs for user: {}", self.username);
        Ok(())
    }

    /// Get total log file size
    pub fn get_log_size(&self) -> Result<u64> {
        let mut total_size = 0u64;

        if self.log_file_path.exists() {
            total_size += fs::metadata(&self.log_file_path)?.len();
        }

        for i in 1..=MAX_LOG_FILES {
            let rotated_path = self.log_file_path.with_extension(format!("log.{}", i));
            if rotated_path.exists() {
                total_size += fs::metadata(&rotated_path)?.len();
            }
        }

        Ok(total_size)
    }
}
