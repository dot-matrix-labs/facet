//! Session management for tracking active Claude CLI executions
//!
//! This module provides thread-safe tracking of active sessions, including
//! status updates, cancellation, and automatic cleanup. Uses Arc<Mutex<>>
//! for shared state management across async tasks.

use crate::error::FacetError;
use crate::models::{SessionState, SessionStatus};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

/// Session metadata tracked for each execution
///
/// Contains timing information, current state, and optional error details.
#[derive(Debug, Clone)]
struct SessionInfo {
    /// Session UUID
    id: Uuid,

    /// Current execution state
    state: SessionState,

    /// ISO 8601 timestamp when session started
    started_at: String,

    /// ISO 8601 timestamp when session completed/failed/cancelled
    completed_at: Option<String>,

    /// Error message if session failed
    error: Option<String>,
}

impl SessionInfo {
    /// Creates a new session in Running state
    ///
    /// # Arguments
    /// * `id` - Session UUID
    ///
    /// # Returns
    /// New SessionInfo with current timestamp
    fn new(id: Uuid) -> Self {
        Self {
            id,
            state: SessionState::Running,
            started_at: chrono::Utc::now().to_rfc3339(),
            completed_at: None,
            error: None,
        }
    }

    /// Converts to public SessionStatus
    ///
    /// # Returns
    /// SessionStatus suitable for API responses
    fn to_status(&self) -> SessionStatus {
        SessionStatus {
            session_id: self.id,
            status: self.state.clone(),
            started_at: self.started_at.clone(),
            completed_at: self.completed_at.clone(),
            error: self.error.clone(),
        }
    }
}

/// Thread-safe session manager
///
/// Tracks all active and recent sessions with atomic operations.
/// Supports concurrent access from multiple API handlers and executors.
#[derive(Debug, Clone)]
pub struct SessionManager {
    /// Map of session ID to session metadata
    sessions: Arc<Mutex<HashMap<Uuid, SessionInfo>>>,

    /// Maximum number of sessions to keep in history
    max_history: usize,
}

impl SessionManager {
    /// Creates a new SessionManager
    ///
    /// # Arguments
    /// * `max_history` - Maximum completed sessions to retain for status queries
    ///
    /// # Returns
    /// New SessionManager instance
    pub fn new(max_history: usize) -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            max_history,
        }
    }

    /// Registers a new session
    ///
    /// Creates a new session entry in Running state. If max concurrent
    /// sessions would be exceeded, returns an error.
    ///
    /// # Arguments
    /// * `session_id` - UUID for the new session
    /// * `max_concurrent` - Maximum allowed concurrent running sessions
    ///
    /// # Returns
    /// Ok(()) if session registered, Err if concurrent limit exceeded
    ///
    /// # Errors
    /// Returns FacetError::Internal if max concurrent sessions exceeded
    pub async fn register(
        &self,
        session_id: Uuid,
        max_concurrent: usize,
    ) -> Result<(), FacetError> {
        let mut sessions = self.sessions.lock().await;

        // Count running sessions
        let running_count = sessions
            .values()
            .filter(|s| matches!(s.state, SessionState::Running))
            .count();

        if running_count >= max_concurrent {
            return Err(FacetError::Internal(format!(
                "Maximum concurrent sessions ({}) exceeded",
                max_concurrent
            )));
        }

        sessions.insert(session_id, SessionInfo::new(session_id));
        Ok(())
    }

    /// Marks a session as completed successfully
    ///
    /// Updates session state to Completed with current timestamp.
    ///
    /// # Arguments
    /// * `session_id` - Session UUID to mark complete
    ///
    /// # Returns
    /// Ok(()) if session found and updated, Err if session not found
    pub async fn complete(&self, session_id: Uuid) -> Result<(), FacetError> {
        let mut sessions = self.sessions.lock().await;

        let session = sessions
            .get_mut(&session_id)
            .ok_or_else(|| FacetError::SessionNotFound(session_id.to_string()))?;

        session.state = SessionState::Completed;
        session.completed_at = Some(chrono::Utc::now().to_rfc3339());

        Ok(())
    }

    /// Marks a session as failed with error message
    ///
    /// Updates session state to Failed with error details and timestamp.
    ///
    /// # Arguments
    /// * `session_id` - Session UUID to mark failed
    /// * `error` - Error message describing failure
    ///
    /// # Returns
    /// Ok(()) if session found and updated, Err if session not found
    pub async fn fail(&self, session_id: Uuid, error: String) -> Result<(), FacetError> {
        let mut sessions = self.sessions.lock().await;

        let session = sessions
            .get_mut(&session_id)
            .ok_or_else(|| FacetError::SessionNotFound(session_id.to_string()))?;

        session.state = SessionState::Failed;
        session.error = Some(error);
        session.completed_at = Some(chrono::Utc::now().to_rfc3339());

        Ok(())
    }

    /// Cancels a running session
    ///
    /// Updates session state to Cancelled. The caller is responsible
    /// for actually terminating the underlying process.
    ///
    /// # Arguments
    /// * `session_id` - Session UUID to cancel
    ///
    /// # Returns
    /// Ok(()) if session found and cancelled, Err if not found or not running
    pub async fn cancel(&self, session_id: Uuid) -> Result<(), FacetError> {
        let mut sessions = self.sessions.lock().await;

        let session = sessions
            .get_mut(&session_id)
            .ok_or_else(|| FacetError::SessionNotFound(session_id.to_string()))?;

        // Can only cancel running sessions
        if !matches!(session.state, SessionState::Running) {
            return Err(FacetError::InvalidRequest(format!(
                "Session {} is not running (current state: {:?})",
                session_id, session.state
            )));
        }

        session.state = SessionState::Cancelled;
        session.completed_at = Some(chrono::Utc::now().to_rfc3339());

        Ok(())
    }

    /// Retrieves session status
    ///
    /// Returns current status information for the specified session.
    ///
    /// # Arguments
    /// * `session_id` - Session UUID to query
    ///
    /// # Returns
    /// SessionStatus if found, Err if session not found
    pub async fn get_status(&self, session_id: Uuid) -> Result<SessionStatus, FacetError> {
        let sessions = self.sessions.lock().await;

        let session = sessions
            .get(&session_id)
            .ok_or_else(|| FacetError::SessionNotFound(session_id.to_string()))?;

        Ok(session.to_status())
    }

    /// Cleans up old completed sessions
    ///
    /// Removes oldest completed/failed/cancelled sessions to maintain
    /// max_history limit. Running sessions are never removed.
    ///
    /// # Returns
    /// Number of sessions removed
    pub async fn cleanup_old_sessions(&self) -> usize {
        let mut sessions = self.sessions.lock().await;

        // Collect completed sessions sorted by completion time
        let mut completed: Vec<_> = sessions
            .iter()
            .filter(|(_, info)| !matches!(info.state, SessionState::Running))
            .map(|(id, info)| (*id, info.completed_at.clone()))
            .collect();

        completed.sort_by(|a, b| a.1.cmp(&b.1));

        // Remove oldest sessions beyond max_history
        let to_remove_count = completed.len().saturating_sub(self.max_history);
        let mut removed_count = 0;

        for (id, _) in completed.iter().take(to_remove_count) {
            sessions.remove(id);
            removed_count += 1;
        }

        removed_count
    }

    /// Returns count of running sessions
    ///
    /// # Returns
    /// Number of sessions in Running state
    pub async fn running_count(&self) -> usize {
        let sessions = self.sessions.lock().await;
        sessions
            .values()
            .filter(|s| matches!(s.state, SessionState::Running))
            .count()
    }

    /// Returns total session count (all states)
    ///
    /// # Returns
    /// Total number of tracked sessions
    pub async fn total_count(&self) -> usize {
        let sessions = self.sessions.lock().await;
        sessions.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_register_session() {
        let manager = SessionManager::new(100);
        let session_id = Uuid::new_v4();

        let result = manager.register(session_id, 10).await;
        assert!(result.is_ok());

        let status = manager.get_status(session_id).await.unwrap();
        assert_eq!(status.session_id, session_id);
        assert_eq!(status.status, SessionState::Running);
        assert!(status.completed_at.is_none());
        assert!(status.error.is_none());
    }

    #[tokio::test]
    async fn test_register_exceeds_max_concurrent() {
        let manager = SessionManager::new(100);

        // Register max_concurrent sessions
        for _ in 0..5 {
            let session_id = Uuid::new_v4();
            manager.register(session_id, 5).await.unwrap();
        }

        // Next registration should fail
        let session_id = Uuid::new_v4();
        let result = manager.register(session_id, 5).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Maximum concurrent"));
    }

    #[tokio::test]
    async fn test_complete_session() {
        let manager = SessionManager::new(100);
        let session_id = Uuid::new_v4();

        manager.register(session_id, 10).await.unwrap();
        manager.complete(session_id).await.unwrap();

        let status = manager.get_status(session_id).await.unwrap();
        assert_eq!(status.status, SessionState::Completed);
        assert!(status.completed_at.is_some());
        assert!(status.error.is_none());
    }

    #[tokio::test]
    async fn test_fail_session() {
        let manager = SessionManager::new(100);
        let session_id = Uuid::new_v4();
        let error_msg = "Test error message".to_string();

        manager.register(session_id, 10).await.unwrap();
        manager.fail(session_id, error_msg.clone()).await.unwrap();

        let status = manager.get_status(session_id).await.unwrap();
        assert_eq!(status.status, SessionState::Failed);
        assert!(status.completed_at.is_some());
        assert_eq!(status.error, Some(error_msg));
    }

    #[tokio::test]
    async fn test_cancel_running_session() {
        let manager = SessionManager::new(100);
        let session_id = Uuid::new_v4();

        manager.register(session_id, 10).await.unwrap();
        let result = manager.cancel(session_id).await;
        assert!(result.is_ok());

        let status = manager.get_status(session_id).await.unwrap();
        assert_eq!(status.status, SessionState::Cancelled);
        assert!(status.completed_at.is_some());
    }

    #[tokio::test]
    async fn test_cancel_completed_session_fails() {
        let manager = SessionManager::new(100);
        let session_id = Uuid::new_v4();

        manager.register(session_id, 10).await.unwrap();
        manager.complete(session_id).await.unwrap();

        let result = manager.cancel(session_id).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not running"));
    }

    #[tokio::test]
    async fn test_get_status_nonexistent_session() {
        let manager = SessionManager::new(100);
        let session_id = Uuid::new_v4();

        let result = manager.get_status(session_id).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[tokio::test]
    async fn test_running_count() {
        let manager = SessionManager::new(100);

        let session1 = Uuid::new_v4();
        let session2 = Uuid::new_v4();
        let session3 = Uuid::new_v4();

        manager.register(session1, 10).await.unwrap();
        manager.register(session2, 10).await.unwrap();
        manager.register(session3, 10).await.unwrap();

        assert_eq!(manager.running_count().await, 3);

        manager.complete(session1).await.unwrap();
        assert_eq!(manager.running_count().await, 2);

        manager.fail(session2, "error".to_string()).await.unwrap();
        assert_eq!(manager.running_count().await, 1);
    }

    #[tokio::test]
    async fn test_total_count() {
        let manager = SessionManager::new(100);

        let session1 = Uuid::new_v4();
        let session2 = Uuid::new_v4();

        manager.register(session1, 10).await.unwrap();
        manager.register(session2, 10).await.unwrap();

        assert_eq!(manager.total_count().await, 2);

        manager.complete(session1).await.unwrap();
        assert_eq!(manager.total_count().await, 2); // Still tracked
    }

    #[tokio::test]
    async fn test_cleanup_old_sessions() {
        let manager = SessionManager::new(2); // Keep max 2 completed

        // Create and complete 5 sessions
        for _ in 0..5 {
            let session_id = Uuid::new_v4();
            manager.register(session_id, 10).await.unwrap();
            manager.complete(session_id).await.unwrap();
            // Small delay to ensure different timestamps
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }

        assert_eq!(manager.total_count().await, 5);

        let removed = manager.cleanup_old_sessions().await;
        assert_eq!(removed, 3); // Should remove 3 oldest

        assert_eq!(manager.total_count().await, 2);
    }

    #[tokio::test]
    async fn test_cleanup_preserves_running_sessions() {
        let manager = SessionManager::new(1); // Keep max 1 completed

        let running = Uuid::new_v4();
        let completed1 = Uuid::new_v4();
        let completed2 = Uuid::new_v4();

        // Register one running and two completed
        manager.register(running, 10).await.unwrap();
        manager.register(completed1, 10).await.unwrap();
        manager.register(completed2, 10).await.unwrap();

        manager.complete(completed1).await.unwrap();
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        manager.complete(completed2).await.unwrap();

        manager.cleanup_old_sessions().await;

        // Running session should still exist
        assert!(manager.get_status(running).await.is_ok());

        // At least one completed should exist
        assert_eq!(manager.total_count().await, 2);
    }
}
