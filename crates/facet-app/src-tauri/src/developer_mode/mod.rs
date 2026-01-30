//! Developer mode module
//!
//! Provides developer tools including:
//! - System path information
//! - Mock test server for manual e2e testing
//! - Debug utilities

mod test_server;

#[cfg(test)]
mod tests;

pub use test_server::DevTestServer;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

/// System paths information for developer mode
#[derive(Debug, Serialize, Deserialize)]
pub struct SystemPaths {
    /// Installation directory
    pub installation_dir: String,
    /// Application config directory
    pub config_dir: String,
    /// Application data directory
    pub data_dir: String,
    /// Application cache directory
    pub cache_dir: String,
    /// Temporary directory
    pub temp_dir: String,
    /// Current working directory
    pub current_dir: String,
    /// Chrome executable path (if available)
    pub chrome_path: Option<String>,
}

impl SystemPaths {
    /// Get system paths for the current application
    pub fn get(app: &AppHandle) -> anyhow::Result<Self> {
        let path_resolver = app.path();

        let installation_dir = std::env::current_exe()?
            .parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| "Unknown".to_string());

        let config_dir = path_resolver
            .app_config_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| "Unknown".to_string());

        let data_dir = path_resolver
            .app_data_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| "Unknown".to_string());

        let cache_dir = path_resolver
            .app_cache_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| "Unknown".to_string());

        let temp_dir = std::env::temp_dir().to_string_lossy().to_string();

        let current_dir = std::env::current_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| "Unknown".to_string());

        // Try to find Chrome path (this is platform-dependent)
        let chrome_path = Self::find_chrome_path();

        Ok(Self {
            installation_dir,
            config_dir,
            data_dir,
            cache_dir,
            temp_dir,
            current_dir,
            chrome_path,
        })
    }

    /// Attempt to find Chrome executable path
    fn find_chrome_path() -> Option<String> {
        // Check common Chrome locations
        let potential_paths = if cfg!(target_os = "linux") {
            vec![
                "/usr/bin/google-chrome",
                "/usr/bin/chromium-browser",
                "/usr/bin/chromium",
                "/snap/bin/chromium",
            ]
        } else if cfg!(target_os = "macos") {
            vec![
                "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
                "/Applications/Chromium.app/Contents/MacOS/Chromium",
            ]
        } else if cfg!(target_os = "windows") {
            vec![
                "C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe",
                "C:\\Program Files (x86)\\Google\\Chrome\\Application\\chrome.exe",
            ]
        } else {
            vec![]
        };

        for path in potential_paths {
            if PathBuf::from(path).exists() {
                return Some(path.to_string());
            }
        }

        None
    }
}
