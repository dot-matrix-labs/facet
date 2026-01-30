//! Health check endpoint
//!
//! Provides server status information including uptime and Claude CLI availability.

use crate::models::HealthResponse;
use std::sync::Arc;
use std::time::Instant;
use tokio::process::Command;
use warp::{reply, Reply};

/// Shared server state for health checks
///
/// Tracks server start time for uptime calculation.
#[derive(Clone)]
pub struct HealthState {
    /// Server start time
    start_time: Instant,

    /// Path to claude-cli binary
    claude_binary_path: String,
}

impl HealthState {
    /// Creates new health state
    ///
    /// # Arguments
    /// * `claude_binary_path` - Path to claude-cli binary to check
    ///
    /// # Returns
    /// New HealthState with current timestamp
    pub fn new(claude_binary_path: String) -> Self {
        Self {
            start_time: Instant::now(),
            claude_binary_path,
        }
    }

    /// Returns server uptime in seconds
    ///
    /// # Returns
    /// Seconds since server started
    pub fn uptime_seconds(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }

    /// Checks if claude-cli binary is available and executable
    ///
    /// Attempts to execute `claude --version` to verify availability.
    ///
    /// # Returns
    /// true if claude-cli responds to --version, false otherwise
    pub async fn check_claude_available(&self) -> bool {
        // Try to execute claude --version
        let result = Command::new(&self.claude_binary_path)
            .arg("--version")
            .output()
            .await;

        match result {
            Ok(output) => output.status.success(),
            Err(_) => false,
        }
    }
}

/// Health check endpoint handler
///
/// Returns server status, version, Claude CLI availability, and uptime.
/// This endpoint does not require authentication and is intended for
/// load balancers and monitoring systems.
///
/// # Arguments
/// * `state` - Shared health state containing server start time and config
///
/// # Returns
/// JSON response with health information
///
/// # Example Response
/// ```json
/// {
///   "status": "healthy",
///   "version": "1.0.0",
///   "claude_cli_available": true,
///   "uptime_seconds": 12345
/// }
/// ```
pub async fn health_handler(state: Arc<HealthState>) -> Result<impl Reply, warp::Rejection> {
    let claude_available = state.check_claude_available().await;

    let response = HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        claude_cli_available: claude_available,
        uptime_seconds: state.uptime_seconds(),
    };

    Ok(reply::json(&response))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_state_creation() {
        let state = HealthState::new("claude".to_string());
        assert_eq!(state.claude_binary_path, "claude");
    }

    #[test]
    fn test_uptime_increases() {
        let state = HealthState::new("claude".to_string());
        let uptime1 = state.uptime_seconds();

        std::thread::sleep(std::time::Duration::from_secs(1));

        let uptime2 = state.uptime_seconds();
        assert!(uptime2 >= uptime1);
    }

    #[tokio::test]
    async fn test_check_claude_available_with_echo() {
        // Use 'echo' as a mock binary that will succeed
        let state = HealthState::new("echo".to_string());
        let available = state.check_claude_available().await;
        assert!(available);
    }

    #[tokio::test]
    async fn test_check_claude_available_with_nonexistent() {
        // Use nonexistent binary
        let state = HealthState::new("/nonexistent/binary".to_string());
        let available = state.check_claude_available().await;
        assert!(!available);
    }

    #[tokio::test]
    async fn test_health_handler_returns_json() {
        let state = Arc::new(HealthState::new("echo".to_string()));
        let result = health_handler(state).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_health_response_format() {
        let state = HealthState::new("echo".to_string());
        let claude_available = state.check_claude_available().await;

        let response = HealthResponse {
            status: "healthy".to_string(),
            version: "1.0.0".to_string(),
            claude_cli_available: claude_available,
            uptime_seconds: state.uptime_seconds(),
        };

        // Serialize to ensure it's valid JSON
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("healthy"));
        assert!(json.contains("1.0.0"));
    }

    #[tokio::test]
    async fn test_health_handler_with_failing_claude() {
        let state = Arc::new(HealthState::new("/nonexistent".to_string()));
        let result = health_handler(state).await;

        // Handler should still succeed even if claude is unavailable
        assert!(result.is_ok());
    }
}
