//! Error types for facet-server
//!
//! This module defines all error types used throughout the application.
//! Each error type maps to specific HTTP status codes and provides
//! structured error responses for clients.

use serde::{Deserialize, Serialize};
use warp::http::StatusCode;
use warp::reject::Reject;

/// Custom error types for facet-server operations
///
/// Each variant represents a specific error condition that can occur
/// during request processing. These errors are converted to appropriate
/// HTTP responses with structured error data.
#[derive(Debug, thiserror::Error)]
pub enum FacetError {
    /// Authentication failed - invalid or missing token
    #[error("Authentication failed: {0}")]
    AuthFailed(String),

    /// Rate limit exceeded for this token/IP
    #[error("Rate limit exceeded: {0}")]
    RateLimited(String),

    /// Request validation failed
    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    /// Claude CLI binary not found or not executable
    #[error("Claude CLI unavailable: {0}")]
    ClaudeUnavailable(String),

    /// Claude CLI process execution failed
    #[error("Execution error: {0}")]
    ExecutionError(String),

    /// Request timeout exceeded
    #[error("Request timeout: {0}")]
    Timeout(String),

    /// Session not found
    #[error("Session not found: {0}")]
    SessionNotFound(String),

    /// Internal server error
    #[error("Internal error: {0}")]
    Internal(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),
}

impl Reject for FacetError {}

/// Structured error response format for API clients
///
/// This structure is serialized to JSON and sent to clients
/// when an error occurs. It provides consistent error formatting
/// across all endpoints.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ErrorResponse {
    /// Error code (matches FacetError variant name)
    pub code: String,

    /// Human-readable error message
    pub message: String,

    /// Optional session ID if error occurred during session processing
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,

    /// ISO 8601 timestamp when error occurred
    pub timestamp: String,

    /// Optional retry-after hint in seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_after_seconds: Option<u64>,
}

impl FacetError {
    /// Converts error to HTTP status code
    ///
    /// Maps each error variant to the appropriate HTTP status code
    /// for REST API responses.
    ///
    /// # Returns
    /// HTTP status code appropriate for this error type
    pub fn status_code(&self) -> StatusCode {
        match self {
            FacetError::AuthFailed(_) => StatusCode::UNAUTHORIZED,
            FacetError::RateLimited(_) => StatusCode::TOO_MANY_REQUESTS,
            FacetError::InvalidRequest(_) => StatusCode::BAD_REQUEST,
            FacetError::ClaudeUnavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
            FacetError::ExecutionError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            FacetError::Timeout(_) => StatusCode::GATEWAY_TIMEOUT,
            FacetError::SessionNotFound(_) => StatusCode::NOT_FOUND,
            FacetError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            FacetError::Config(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// Converts error to error code string
    ///
    /// Returns a string representation of the error type that
    /// clients can use for programmatic error handling.
    ///
    /// # Returns
    /// Error code string (e.g., "AUTH_FAILED", "RATE_LIMITED")
    pub fn error_code(&self) -> String {
        match self {
            FacetError::AuthFailed(_) => "AUTH_FAILED",
            FacetError::RateLimited(_) => "RATE_LIMITED",
            FacetError::InvalidRequest(_) => "INVALID_REQUEST",
            FacetError::ClaudeUnavailable(_) => "CLAUDE_UNAVAILABLE",
            FacetError::ExecutionError(_) => "EXECUTION_ERROR",
            FacetError::Timeout(_) => "TIMEOUT",
            FacetError::SessionNotFound(_) => "SESSION_NOT_FOUND",
            FacetError::Internal(_) => "INTERNAL_ERROR",
            FacetError::Config(_) => "CONFIG_ERROR",
        }
        .to_string()
    }

    /// Converts error to structured error response
    ///
    /// Creates an ErrorResponse with appropriate retry-after hints
    /// for rate-limited requests.
    ///
    /// # Arguments
    /// * `session_id` - Optional session ID to include in response
    ///
    /// # Returns
    /// Structured error response ready for JSON serialization
    pub fn to_error_response(&self, session_id: Option<String>) -> ErrorResponse {
        let retry_after = match self {
            FacetError::RateLimited(_) => Some(60),
            _ => None,
        };

        ErrorResponse {
            code: self.error_code(),
            message: self.to_string(),
            session_id,
            timestamp: chrono::Utc::now().to_rfc3339(),
            retry_after_seconds: retry_after,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_failed_status_code() {
        let err = FacetError::AuthFailed("invalid token".to_string());
        assert_eq!(err.status_code(), StatusCode::UNAUTHORIZED);
        assert_eq!(err.error_code(), "AUTH_FAILED");
    }

    #[test]
    fn test_rate_limited_status_code() {
        let err = FacetError::RateLimited("too many requests".to_string());
        assert_eq!(err.status_code(), StatusCode::TOO_MANY_REQUESTS);
        assert_eq!(err.error_code(), "RATE_LIMITED");
    }

    #[test]
    fn test_invalid_request_status_code() {
        let err = FacetError::InvalidRequest("missing field".to_string());
        assert_eq!(err.status_code(), StatusCode::BAD_REQUEST);
        assert_eq!(err.error_code(), "INVALID_REQUEST");
    }

    #[test]
    fn test_claude_unavailable_status_code() {
        let err = FacetError::ClaudeUnavailable("binary not found".to_string());
        assert_eq!(err.status_code(), StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(err.error_code(), "CLAUDE_UNAVAILABLE");
    }

    #[test]
    fn test_execution_error_status_code() {
        let err = FacetError::ExecutionError("process crashed".to_string());
        assert_eq!(err.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(err.error_code(), "EXECUTION_ERROR");
    }

    #[test]
    fn test_timeout_status_code() {
        let err = FacetError::Timeout("exceeded 300s".to_string());
        assert_eq!(err.status_code(), StatusCode::GATEWAY_TIMEOUT);
        assert_eq!(err.error_code(), "TIMEOUT");
    }

    #[test]
    fn test_session_not_found_status_code() {
        let err = FacetError::SessionNotFound("uuid-123".to_string());
        assert_eq!(err.status_code(), StatusCode::NOT_FOUND);
        assert_eq!(err.error_code(), "SESSION_NOT_FOUND");
    }

    #[test]
    fn test_error_response_without_session_id() {
        let err = FacetError::InvalidRequest("test error".to_string());
        let response = err.to_error_response(None);

        assert_eq!(response.code, "INVALID_REQUEST");
        assert_eq!(response.message, "Invalid request: test error");
        assert!(response.session_id.is_none());
        assert!(response.retry_after_seconds.is_none());
        // Timestamp should be valid RFC3339
        assert!(chrono::DateTime::parse_from_rfc3339(&response.timestamp).is_ok());
    }

    #[test]
    fn test_error_response_with_session_id() {
        let err = FacetError::ExecutionError("failed".to_string());
        let session_id = "test-session-123".to_string();
        let response = err.to_error_response(Some(session_id.clone()));

        assert_eq!(response.code, "EXECUTION_ERROR");
        assert_eq!(response.session_id, Some(session_id));
    }

    #[test]
    fn test_rate_limited_has_retry_after() {
        let err = FacetError::RateLimited("limit exceeded".to_string());
        let response = err.to_error_response(None);

        assert_eq!(response.retry_after_seconds, Some(60));
    }

    #[test]
    fn test_error_response_json_serialization() {
        let err = FacetError::AuthFailed("invalid token".to_string());
        let response = err.to_error_response(Some("session-xyz".to_string()));

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("AUTH_FAILED"));
        assert!(json.contains("session-xyz"));
    }

    #[test]
    fn test_error_display_format() {
        let err = FacetError::AuthFailed("invalid token".to_string());
        let display = format!("{}", err);
        assert_eq!(display, "Authentication failed: invalid token");
    }
}
