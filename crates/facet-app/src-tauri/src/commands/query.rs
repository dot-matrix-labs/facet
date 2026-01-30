/// Tauri commands for querying the robert-server GraphRAG endpoints
///
/// This module provides Tauri commands for:
/// - Querying the knowledge graph
/// - Managing contexts (Personal/Work/Business)
/// - Document ingestion
/// - Memory management (mark as outdated, attribution)
///
/// NOTE: Currently contains legacy webdriver commands that should be migrated/removed
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

// ============================================================================
// GraphRAG Query Commands (TO BE IMPLEMENTED)
// ============================================================================

/// Request to query the knowledge graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphQueryRequest {
    pub query: String,
    pub context_id: Option<String>,
    pub limit: Option<usize>,
}

/// Response from a graph query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphQueryResponse {
    pub answer: String,
    pub sources: Vec<SourceAttribution>,
    pub used_local_model: bool,
    pub used_cloud_model: bool,
}

/// Source attribution for transparency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceAttribution {
    pub node_id: String,
    pub content: String,
    pub metadata: serde_json::Value,
}

/// Query the knowledge graph using robert-server
#[tauri::command]
pub async fn query_knowledge_graph(
    _request: GraphQueryRequest,
    _state: State<'_, AppState>,
) -> Result<GraphQueryResponse, String> {
    // TODO: Implement GraphRAG query via robert-server
    // Should call POST /api/query endpoint
    Err("Not yet implemented - see TODO.md".to_string())
}

/// Ingest a document into the knowledge graph
#[tauri::command]
pub async fn ingest_document(
    _path: String,
    _context_id: String,
    _state: State<'_, AppState>,
) -> Result<(), String> {
    // TODO: Implement document ingestion via robert-server
    // Should call POST /api/ingest endpoint
    Err("Not yet implemented - see TODO.md".to_string())
}

/// Get available contexts (Personal, Work, etc.)
#[tauri::command]
pub async fn get_contexts(_state: State<'_, AppState>) -> Result<Vec<ContextInfo>, String> {
    // TODO: Implement context listing via robert-server
    // Should call GET /api/contexts endpoint
    Err("Not yet implemented - see TODO.md".to_string())
}

/// Context information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub is_active: bool,
}

/// Switch active context
#[tauri::command]
pub async fn set_active_context(
    _context_id: String,
    _state: State<'_, AppState>,
) -> Result<(), String> {
    // TODO: Implement context switching via robert-server
    // Should call POST /api/contexts/:id/activate endpoint
    Err("Not yet implemented - see TODO.md".to_string())
}

/// Mark a node as outdated (reactive pruning)
#[tauri::command]
pub async fn mark_as_outdated(
    _node_id: String,
    _state: State<'_, AppState>,
) -> Result<(), String> {
    // TODO: Implement reactive pruning via robert-server
    // Should call POST /api/nodes/:id/prune endpoint
    Err("Not yet implemented - see TODO.md".to_string())
}

/// Get attribution for a query result
#[tauri::command]
pub async fn get_attribution(
    _node_id: String,
    _state: State<'_, AppState>,
) -> Result<Vec<SourceAttribution>, String> {
    // TODO: Implement attribution retrieval via robert-server
    // Should call GET /api/nodes/:id/attribution endpoint
    Err("Not yet implemented - see TODO.md".to_string())
}
