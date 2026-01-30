//! Session management endpoints
//!
//! Provides endpoints for querying and cancelling active sessions.

use crate::error::{ErrorResponse, FacetError};
use crate::session::SessionManager;
use std::sync::Arc;
use uuid::Uuid;
use warp::{http::StatusCode, reply, Reply};

/// GET /api/v1/sessions/:id handler
///
/// Returns status information for a specific session.
///
/// # Arguments
/// * `session_id` - UUID of the session to query
/// * `manager` - Shared session manager
///
/// # Returns
/// JSON response with session status or 404 if not found
///
/// # Example Response
/// ```json
/// {
///   "session_id": "uuid",
///   "status": "running",
///   "started_at": "2025-10-17T10:30:00Z",
///   "completed_at": null,
///   "error": null
/// }
/// ```
pub async fn get_session_handler(
    session_id: Uuid,
    manager: Arc<SessionManager>,
) -> Result<impl Reply, warp::Rejection> {
    match manager.get_status(session_id).await {
        Ok(status) => Ok(reply::json(&status)),
        Err(e) => {
            let error_response = e.to_error_response(Some(session_id.to_string()));
            Ok(reply::json(&error_response))
        }
    }
}

/// DELETE /api/v1/sessions/:id handler
///
/// Cancels a running session. The session must be in Running state.
///
/// # Arguments
/// * `session_id` - UUID of the session to cancel
/// * `manager` - Shared session manager
///
/// # Returns
/// JSON response with updated session status or error
///
/// # Example Response
/// ```json
/// {
///   "session_id": "uuid",
///   "status": "cancelled"
/// }
/// ```
pub async fn delete_session_handler(
    session_id: Uuid,
    manager: Arc<SessionManager>,
) -> Result<impl Reply, warp::Rejection> {
    match manager.cancel(session_id).await {
        Ok(_) => {
            // Fetch updated status
            match manager.get_status(session_id).await {
                Ok(status) => Ok(reply::json(&status)),
                Err(e) => {
                    let error_response = e.to_error_response(Some(session_id.to_string()));
                    Ok(reply::json(&error_response))
                }
            }
        }
        Err(e) => {
            let error_response = e.to_error_response(Some(session_id.to_string()));
            Ok(reply::json(&error_response))
        }
    }
}

/// Converts FacetError to HTTP response
///
/// Helper function to create appropriate HTTP status code and error response
/// for different error types.
///
/// # Arguments
/// * `error` - FacetError to convert
/// * `session_id` - Optional session ID for error response
///
/// # Returns
/// HTTP status code and error response JSON
pub fn error_to_response(
    error: FacetError,
    session_id: Option<String>,
) -> (StatusCode, ErrorResponse) {
    let status = error.status_code();
    let response = error.to_error_response(session_id);
    (status, response)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::SessionState;

    #[tokio::test]
    async fn test_get_session_handler_found() {
        let manager = Arc::new(SessionManager::new(100));
        let session_id = Uuid::new_v4();

        // Register a session
        manager.register(session_id, 10).await.unwrap();

        // Query it
        let result = get_session_handler(session_id, manager).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_session_handler_not_found() {
        let manager = Arc::new(SessionManager::new(100));
        let session_id = Uuid::new_v4();

        // Query nonexistent session
        let result = get_session_handler(session_id, manager).await;
        assert!(result.is_ok()); // Handler doesn't reject, returns error JSON
    }

    #[tokio::test]
    async fn test_delete_session_handler_success() {
        let manager = Arc::new(SessionManager::new(100));
        let session_id = Uuid::new_v4();

        // Register a running session
        manager.register(session_id, 10).await.unwrap();

        // Cancel it
        let result = delete_session_handler(session_id, manager.clone()).await;
        assert!(result.is_ok());

        // Verify it's cancelled
        let status = manager.get_status(session_id).await.unwrap();
        assert_eq!(status.status, SessionState::Cancelled);
    }

    #[tokio::test]
    async fn test_delete_session_handler_not_running() {
        let manager = Arc::new(SessionManager::new(100));
        let session_id = Uuid::new_v4();

        // Register and complete a session
        manager.register(session_id, 10).await.unwrap();
        manager.complete(session_id).await.unwrap();

        // Try to cancel it
        let result = delete_session_handler(session_id, manager).await;
        assert!(result.is_ok()); // Returns error JSON, not rejection
    }

    #[tokio::test]
    async fn test_delete_session_handler_not_found() {
        let manager = Arc::new(SessionManager::new(100));
        let session_id = Uuid::new_v4();

        // Try to cancel nonexistent session
        let result = delete_session_handler(session_id, manager).await;
        assert!(result.is_ok()); // Returns error JSON
    }

    #[test]
    fn test_error_to_response_auth_failed() {
        let error = FacetError::AuthFailed("invalid token".to_string());
        let (status, response) = error_to_response(error, None);

        assert_eq!(status, StatusCode::UNAUTHORIZED);
        assert_eq!(response.code, "AUTH_FAILED");
    }

    #[test]
    fn test_error_to_response_session_not_found() {
        let error = FacetError::SessionNotFound("uuid-123".to_string());
        let session_id = Some("uuid-123".to_string());
        let (status, response) = error_to_response(error, session_id.clone());

        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(response.code, "SESSION_NOT_FOUND");
        assert_eq!(response.session_id, session_id);
    }

    #[test]
    fn test_error_to_response_rate_limited() {
        let error = FacetError::RateLimited("too many requests".to_string());
        let (status, response) = error_to_response(error, None);

        assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
        assert_eq!(response.retry_after_seconds, Some(60));
    }
}
