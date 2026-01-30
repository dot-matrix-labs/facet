mod agent;
pub mod browser;
mod developer_mode;
mod feedback;
mod logging;
mod profiles;

pub use agent::*;
// Note: browser module is pub mod so we can selectively export commands to avoid conflicts
pub use developer_mode::*;
pub use feedback::*;
pub use logging::*;
pub use profiles::*;

use crate::claude::ClaudeHealthCheck;
use crate::events::*;
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State};

// Placeholder for remaining non-webdriver commands or cleaned up structure
// If there are other commands here that don't use webdriver, they should be preserved.
// Looking at the file, almost everything was webdriver related (navigate, get_content, screenshot, ask_claude).
// Checks for Claude Health are still valid as they don't depend on webdriver crate directly,
// BUT check_claude_health implementation in mod.rs might use common types?
// No, ClaudeHealthCheck is from crate::claude. So that's fine.

/// Check Claude CLI installation and configuration
#[tauri::command]
pub async fn check_claude_health(app: AppHandle) -> Result<ClaudeHealthCheck, String> {
    emit_claude_checking(&app, "Checking Claude CLI installation...").ok();

    let health = ClaudeHealthCheck::check().await;

    // Emit appropriate event based on health status
    match health.status {
        crate::claude::HealthStatus::Healthy => {
            emit_claude_ready(
                &app,
                health.version.as_deref().unwrap_or("unknown"),
                health.path.as_deref().unwrap_or("unknown"),
                health.authenticated,
            )
            .ok();
        }
        crate::claude::HealthStatus::Warning | crate::claude::HealthStatus::Error => {
            if let Some(issue) = health.issues.first() {
                let suggestion = health
                    .suggestions
                    .first()
                    .map(|s| s.as_str())
                    .unwrap_or("See documentation for setup instructions");
                emit_claude_not_ready(&app, issue, suggestion).ok();
            }
        }
    }

    Ok(health)
}

/// System diagnostics - check all dependencies
#[derive(Debug, Serialize, Deserialize)]
pub struct SystemDiagnostics {
    pub chrome_status: String,
    pub chrome_installed: bool,
    pub claude_health: ClaudeHealthCheck,
    pub browser_running: bool,
    pub current_url: Option<String>,
}

#[tauri::command]
pub async fn run_diagnostics(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<SystemDiagnostics, String> {
    log::info!("Running system diagnostics...");
    emit_info(&app, "Running system diagnostics...").ok();

    // Check Claude
    let claude_health = ClaudeHealthCheck::check().await;

    // Check if Chrome/Chromium is installed (try to detect)
    let chrome_installed = check_chrome_installed().await;

    // Check browser status via HTTP client now
    let client = &state.http_client;
    // We can't easily check if "browser is running" inside the remote server without an endpoint.
    // We can check if the SERVER is running.
    let url = "http://localhost:9669/health";
    let browser_running = client.get(url).send().await.is_ok(); // Simplified check

    let current_url = None; // Can't get this easily without new endpoint

    let chrome_status = if browser_running {
        "Webdriver Server Available".to_string()
    } else if chrome_installed {
        "Chrome Installed (Server Not Reachable)".to_string()
    } else {
        "Not installed".to_string()
    };

    log::info!(
        "Diagnostics complete - Chrome: {}, Claude: {:?}",
        chrome_status,
        claude_health.status
    );

    let diagnostics = SystemDiagnostics {
        chrome_status,
        chrome_installed,
        claude_health,
        browser_running,
        current_url,
    };

    emit_success(&app, "Diagnostics complete").ok();

    Ok(diagnostics)
}

/// Check if Chrome/Chromium is installed on the system
async fn check_chrome_installed() -> bool {
    use std::process::Command;

    // Try common Chrome/Chromium locations and commands
    let commands = vec![
        "google-chrome",
        "google-chrome-stable",
        "chromium",
        "chromium-browser",
        "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
        "/usr/bin/google-chrome",
        "/usr/bin/chromium",
        "/usr/bin/chromium-browser",
    ];

    for cmd in commands {
        if let Ok(output) = Command::new(cmd).arg("--version").output() {
            if output.status.success() {
                log::debug!("Found Chrome at: {}", cmd);
                return true;
            }
        }
    }

    // Also check if chromiumoxide can auto-download
    log::debug!("Chrome not found in system");
    false
}
