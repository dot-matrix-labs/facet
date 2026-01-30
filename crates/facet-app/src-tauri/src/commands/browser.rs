/// Tauri commands for interacting with the standalone webdriver
///
/// This module provides Tauri commands that check the status of the external
/// webdriver server and proxy requests to it.
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use tauri::State;

// ============================================================================
// Response Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebdriverStatus {
    pub is_available: bool,
    pub message: String,
}

// ============================================================================
// Tauri Commands
// ============================================================================

/// Check if the Webdriver server is running and available
#[tauri::command]
pub async fn get_browser_status(state: State<'_, AppState>) -> Result<WebdriverStatus, String> {
    // First check if we are in webdriver mode (detected at startup)
    let mode_enabled = *state.webdriver_mode.lock().await;
    if !mode_enabled {
        return Ok(WebdriverStatus {
            is_available: false,
            message: "Webdriver mode disabled (not detected at startup)".to_string(),
        });
    }

    let client = &state.http_client;

    // Attempt to ping the local webdriver server
    // TODO: Make port configurable or discoverable
    let url = "http://localhost:9669/health";

    match client.get(url).send().await {
        Ok(res) => {
            if res.status().is_success() {
                Ok(WebdriverStatus {
                    is_available: true,
                    message: "Webdriver server is active".to_string(),
                })
            } else {
                Ok(WebdriverStatus {
                    is_available: false,
                    message: format!("Webdriver server returned status: {}", res.status()),
                })
            }
        }
        Err(e) => Ok(WebdriverStatus {
            is_available: false,
            message: format!("Webdriver server unreachable: {}", e),
        }),
    }
}

/// Send an inference request to the webdriver
#[tauri::command]
pub async fn execute_webdriver_inference(
    prompt: String,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let client = &state.http_client;
    let url = "http://localhost:8443/inference";

    let payload = serde_json::json!({
        "prompt": prompt
    });

    let res = client
        .post(url)
        .json(&payload)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    let json: serde_json::Value = res
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    Ok(json)
}

// Deprecated / Stubbed commands to prevent frontend breakage temporarily
// These should be removed from frontend eventually

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchBrowserRequest {
    #[serde(default)]
    pub headless: bool,
    #[serde(default)]
    pub no_sandbox: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchBrowserResponse {
    pub session_id: String,
    // Stubbed SessionInfo
    pub session_info: serde_json::Value,
}

#[tauri::command]
pub async fn launch_browser_session(
    _request: LaunchBrowserRequest,
    _state: State<'_, AppState>,
) -> Result<LaunchBrowserResponse, String> {
    // Return a dummy session to satisfy frontend types if they haven't been updated yet
    // But logically we just rely on get_browser_status
    Ok(LaunchBrowserResponse {
        session_id: "external-webdriver".to_string(),
        session_info: serde_json::json!({
             "id": "external-webdriver",
             "profile_name": "External",
             "profile_dir": "",
             "created_at": chrono::Utc::now().to_rfc3339(),
             "browser_pid": 0,
             "debug_port": 0
        }),
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloseBrowserResponse {
    pub success: bool,
    pub message: String,
}

#[tauri::command]
pub async fn close_browser_session(
    _session_id: String,
    _state: State<'_, AppState>,
) -> Result<CloseBrowserResponse, String> {
    Ok(CloseBrowserResponse {
        success: true,
        message: "External webdriver managed independently".to_string(),
    })
}

#[tauri::command]
pub async fn close_all_browser_sessions(_state: State<'_, AppState>) -> Result<usize, String> {
    Ok(0)
}
