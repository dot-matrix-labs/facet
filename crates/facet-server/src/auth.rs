//! Authentication and rate limiting middleware
//!
//! Provides bearer token authentication and per-token rate limiting
//! for API endpoints. Supports development mode with relaxed requirements.

use crate::error::FacetError;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use warp::{reject, Filter, Rejection};

/// Authentication state
///
/// Tracks valid tokens and per-token request counts for rate limiting.
#[derive(Clone)]
pub struct AuthState {
    /// List of valid bearer tokens
    valid_tokens: Vec<String>,

    /// Whether authentication is required
    require_auth: bool,

    /// Rate limit (requests per minute)
    rate_limit: u32,

    /// Map of token to request timestamps (for rate limiting)
    request_history: Arc<Mutex<HashMap<String, Vec<std::time::Instant>>>>,
}

impl AuthState {
    /// Creates new authentication state
    ///
    /// # Arguments
    /// * `valid_tokens` - List of accepted bearer tokens
    /// * `require_auth` - Whether to enforce authentication (false for dev mode)
    /// * `rate_limit` - Maximum requests per minute per token
    ///
    /// # Returns
    /// New AuthState instance
    pub fn new(valid_tokens: Vec<String>, require_auth: bool, rate_limit: u32) -> Self {
        Self {
            valid_tokens,
            require_auth,
            rate_limit,
            request_history: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Validates bearer token
    ///
    /// Checks if the provided token is in the list of valid tokens.
    /// If require_auth is false, always returns Ok.
    ///
    /// # Arguments
    /// * `token` - Bearer token from Authorization header
    ///
    /// # Returns
    /// Ok(token) if valid, Err if invalid or missing
    pub fn validate_token(&self, token: &str) -> Result<String, FacetError> {
        if !self.require_auth {
            return Ok(token.to_string());
        }

        if self.valid_tokens.contains(&token.to_string()) {
            Ok(token.to_string())
        } else {
            Err(FacetError::AuthFailed("Invalid token".to_string()))
        }
    }

    /// Checks rate limit for a token
    ///
    /// Examines request history for the token and determines if the
    /// rate limit has been exceeded. Cleans up old entries (>1 minute).
    ///
    /// # Arguments
    /// * `token` - Bearer token to check
    ///
    /// # Returns
    /// Ok(()) if within limit, Err if rate limit exceeded
    pub async fn check_rate_limit(&self, token: &str) -> Result<(), FacetError> {
        let mut history = self.request_history.lock().await;
        let now = std::time::Instant::now();
        let one_minute_ago = now - std::time::Duration::from_secs(60);

        // Get or create history for this token
        let requests = history.entry(token.to_string()).or_insert_with(Vec::new);

        // Remove requests older than 1 minute
        requests.retain(|&timestamp| timestamp > one_minute_ago);

        // Check if limit exceeded
        if requests.len() >= self.rate_limit as usize {
            return Err(FacetError::RateLimited(format!(
                "Rate limit of {} requests per minute exceeded",
                self.rate_limit
            )));
        }

        // Record this request
        requests.push(now);

        Ok(())
    }

    /// Clears rate limit history for a token
    ///
    /// Useful for testing or administrative operations.
    ///
    /// # Arguments
    /// * `token` - Token to clear history for
    pub async fn clear_rate_limit(&self, token: &str) {
        let mut history = self.request_history.lock().await;
        history.remove(token);
    }

    /// Returns current request count for a token in the last minute
    ///
    /// # Arguments
    /// * `token` - Token to check
    ///
    /// # Returns
    /// Number of requests in the last minute
    pub async fn get_request_count(&self, token: &str) -> usize {
        let history = self.request_history.lock().await;
        let now = std::time::Instant::now();
        let one_minute_ago = now - std::time::Duration::from_secs(60);

        history
            .get(token)
            .map(|requests| {
                requests
                    .iter()
                    .filter(|&&timestamp| timestamp > one_minute_ago)
                    .count()
            })
            .unwrap_or(0)
    }
}

/// Custom rejection for authentication failures
#[derive(Debug)]
pub struct AuthRejection(pub FacetError);

impl reject::Reject for AuthRejection {}

/// Extracts bearer token from Authorization header
///
/// Parses "Bearer <token>" format and returns the token portion.
///
/// # Arguments
/// * `auth_header` - Authorization header value
///
/// # Returns
/// Extracted token or rejection if malformed
fn extract_bearer_token(auth_header: String) -> Result<String, Rejection> {
    if let Some(token) = auth_header.strip_prefix("Bearer ") {
        Ok(token.to_string())
    } else {
        Err(reject::custom(AuthRejection(FacetError::AuthFailed(
            "Invalid Authorization header format".to_string(),
        ))))
    }
}

/// Creates authentication filter
///
/// Warp filter that validates bearer tokens and enforces rate limits.
/// Returns the validated token for downstream handlers.
///
/// # Arguments
/// * `auth_state` - Shared authentication state
///
/// # Returns
/// Warp filter that extracts and validates bearer tokens
pub fn with_auth(
    auth_state: Arc<AuthState>,
) -> impl Filter<Extract = (String,), Error = Rejection> + Clone {
    warp::header::optional::<String>("authorization").and_then(
        move |auth_header: Option<String>| {
            let auth_state = auth_state.clone();
            async move {
                // If auth not required and no header, use empty token
                let token = if let Some(header) = auth_header {
                    extract_bearer_token(header)?
                } else if !auth_state.require_auth {
                    String::new()
                } else {
                    return Err(reject::custom(AuthRejection(FacetError::AuthFailed(
                        "Missing Authorization header".to_string(),
                    ))));
                };

                // Validate token
                auth_state
                    .validate_token(&token)
                    .map_err(|e| reject::custom(AuthRejection(e)))?;

                // Check rate limit
                auth_state
                    .check_rate_limit(&token)
                    .await
                    .map_err(|e| reject::custom(AuthRejection(e)))?;

                Ok::<String, Rejection>(token)
            }
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_auth_state() -> AuthState {
        AuthState::new(
            vec!["valid-token-1".to_string(), "valid-token-2".to_string()],
            true,
            5, // 5 requests per minute
        )
    }

    #[test]
    fn test_validate_token_success() {
        let auth_state = create_test_auth_state();
        let result = auth_state.validate_token("valid-token-1");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "valid-token-1");
    }

    #[test]
    fn test_validate_token_failure() {
        let auth_state = create_test_auth_state();
        let result = auth_state.validate_token("invalid-token");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_token_without_auth_required() {
        let auth_state = AuthState::new(vec![], false, 10);
        let result = auth_state.validate_token("any-token");
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_rate_limit_within_limit() {
        let auth_state = create_test_auth_state();
        let token = "valid-token-1";

        // Make 5 requests (at limit)
        for _ in 0..5 {
            let result = auth_state.check_rate_limit(token).await;
            assert!(result.is_ok());
        }
    }

    #[tokio::test]
    async fn test_rate_limit_exceeds_limit() {
        let auth_state = create_test_auth_state();
        let token = "valid-token-1";

        // Make 5 requests (at limit)
        for _ in 0..5 {
            auth_state.check_rate_limit(token).await.unwrap();
        }

        // 6th request should fail
        let result = auth_state.check_rate_limit(token).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Rate limit"));
    }

    #[tokio::test]
    async fn test_rate_limit_per_token() {
        let auth_state = create_test_auth_state();

        // Token 1 at limit
        for _ in 0..5 {
            auth_state.check_rate_limit("valid-token-1").await.unwrap();
        }

        // Token 2 should still work
        let result = auth_state.check_rate_limit("valid-token-2").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_clear_rate_limit() {
        let auth_state = create_test_auth_state();
        let token = "valid-token-1";

        // Hit rate limit
        for _ in 0..5 {
            auth_state.check_rate_limit(token).await.unwrap();
        }

        // Clear history
        auth_state.clear_rate_limit(token).await;

        // Should work again
        let result = auth_state.check_rate_limit(token).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_request_count() {
        let auth_state = create_test_auth_state();
        let token = "valid-token-1";

        assert_eq!(auth_state.get_request_count(token).await, 0);

        // Make 3 requests
        for _ in 0..3 {
            auth_state.check_rate_limit(token).await.unwrap();
        }

        assert_eq!(auth_state.get_request_count(token).await, 3);
    }

    #[test]
    fn test_extract_bearer_token_success() {
        let header = "Bearer my-secret-token".to_string();
        let result = extract_bearer_token(header);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "my-secret-token");
    }

    #[test]
    fn test_extract_bearer_token_invalid_format() {
        let header = "InvalidFormat token".to_string();
        let result = extract_bearer_token(header);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_with_auth_filter_valid_token() {
        let auth_state = Arc::new(create_test_auth_state());
        let _filter = with_auth(auth_state);

        // This would need warp test utilities to properly test
        // For now, we're testing the underlying functions
    }

    #[tokio::test]
    async fn test_rate_limit_cleanup_old_requests() {
        let auth_state = create_test_auth_state();
        let token = "valid-token-1";

        // Make requests
        for _ in 0..3 {
            auth_state.check_rate_limit(token).await.unwrap();
        }

        assert_eq!(auth_state.get_request_count(token).await, 3);

        // Note: We can't easily test time-based cleanup without mocking time
        // In a real scenario, requests older than 1 minute would be removed
    }
}
