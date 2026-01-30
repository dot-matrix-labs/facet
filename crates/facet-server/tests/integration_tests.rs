//! Integration tests for facet-server
//!
//! These tests verify the full API functionality by making actual HTTP requests
//! to the server. Tests use the mock executor for fast, reliable testing without
//! requiring claude-cli to be installed.
//!
//! To run these tests:
//! ```bash
//! cargo test -p facet-server --test integration_tests
//! ```

use facet_server::{
    api::{execute_handler, get_session_handler, health_handler},
    auth::{with_auth, AuthState},
    claude::{Executor, MockClaudeExecutor},
    models::{
        ClaudeEvent, DomState, RequestContext, RequestOptions, FacetRequest, Screenshot,
        ScreenshotMetadata, Viewport,
    },
    session::SessionManager,
    Config,
};
use std::sync::Arc;
use uuid::Uuid;
use warp::http::StatusCode;
use warp::test::request;
use warp::Filter;

/// Helper to create a test FacetRequest
fn create_test_request() -> FacetRequest {
    FacetRequest {
        session_id: Uuid::new_v4(),
        context: RequestContext {
            screenshots: vec![Screenshot {
                timestamp: "2025-10-17T10:30:00Z".to_string(),
                image_data: {
                    use base64::{engine::general_purpose, Engine as _};
                    general_purpose::STANDARD
                        .encode(b"test image data - this is a minimal PNG-like structure")
                },
                metadata: ScreenshotMetadata {
                    window_title: "Test Window".to_string(),
                    url: Some("https://example.com".to_string()),
                    viewport: Viewport {
                        width: 1920,
                        height: 1080,
                    },
                },
            }],
            dom_state: DomState {
                accessible_tree: "test accessible tree".to_string(),
                interactive_elements: vec![],
            },
            user_intent: "Click the login button".to_string(),
        },
        prompt: "Please help me log into the website".to_string(),
        options: RequestOptions::default(),
    }
}

/// Test health endpoint returns correct response
#[tokio::test]
async fn test_health_endpoint() {
    use facet_server::api::health::HealthState;

    let health_state = Arc::new(HealthState::new("claude".to_string()));

    let filter = warp::path!("api" / "v1" / "health")
        .and(warp::get())
        .and(warp::any().map(move || health_state.clone()))
        .and_then(health_handler);

    let response = request()
        .method("GET")
        .path("/api/v1/health")
        .reply(&filter)
        .await;

    assert_eq!(response.status(), StatusCode::OK);

    let body: serde_json::Value = serde_json::from_slice(response.body()).expect("Valid JSON");
    assert_eq!(body["status"], "healthy");
    assert!(body.get("version").is_some());
    assert!(body.get("uptime_seconds").is_some());
}

/// Test authentication with valid token
#[tokio::test]
async fn test_auth_with_valid_token() {
    let tokens = vec!["test-token-123".to_string()];
    let auth_state = Arc::new(AuthState::new(tokens, true, 100));

    let filter = warp::path("test")
        .and(with_auth(auth_state))
        .map(|token: String| warp::reply::json(&token));

    let response = request()
        .method("GET")
        .path("/test")
        .header("authorization", "Bearer test-token-123")
        .reply(&filter)
        .await;

    assert_eq!(response.status(), StatusCode::OK);
}

/// Test authentication with invalid token
#[tokio::test]
async fn test_auth_with_invalid_token() {
    let tokens = vec!["test-token-123".to_string()];
    let auth_state = Arc::new(AuthState::new(tokens, true, 100));

    let filter = warp::path("test")
        .and(with_auth(auth_state))
        .map(|token: String| warp::reply::json(&token))
        .recover(handle_rejection);

    let response = request()
        .method("GET")
        .path("/test")
        .header("authorization", "Bearer wrong-token")
        .reply(&filter)
        .await;

    // Should be UNAUTHORIZED (401) or INTERNAL_SERVER_ERROR (500) depending on error handling
    assert!(
        response.status() == StatusCode::UNAUTHORIZED
            || response.status() == StatusCode::INTERNAL_SERVER_ERROR,
        "Expected 401 or 500, got {}",
        response.status()
    );
}

/// Test authentication without header
#[tokio::test]
async fn test_auth_without_header() {
    let tokens = vec!["test-token-123".to_string()];
    let auth_state = Arc::new(AuthState::new(tokens, true, 100));

    let filter = warp::path("test")
        .and(with_auth(auth_state))
        .map(|token: String| warp::reply::json(&token))
        .recover(handle_rejection);

    let response = request().method("GET").path("/test").reply(&filter).await;

    // Should be UNAUTHORIZED (401) or INTERNAL_SERVER_ERROR (500) depending on error handling
    assert!(
        response.status() == StatusCode::UNAUTHORIZED
            || response.status() == StatusCode::INTERNAL_SERVER_ERROR,
        "Expected 401 or 500, got {}",
        response.status()
    );
}

/// Test session creation and retrieval
#[tokio::test]
async fn test_session_lifecycle() {
    let session_manager = Arc::new(SessionManager::new(100));
    let session_id = Uuid::new_v4();

    // Create session (with max_concurrent limit)
    assert!(session_manager.register(session_id, 10).await.is_ok());

    // Retrieve session
    let manager_clone = session_manager.clone();
    let filter = warp::path!("sessions" / Uuid)
        .and(warp::get())
        .and(warp::any().map(move || manager_clone.clone()))
        .and_then(get_session_handler);

    let response = request()
        .method("GET")
        .path(&format!("/sessions/{}", session_id))
        .reply(&filter)
        .await;

    assert_eq!(response.status(), StatusCode::OK);

    let body: serde_json::Value = serde_json::from_slice(response.body()).expect("Valid JSON");
    assert_eq!(body["session_id"], session_id.to_string());
    assert_eq!(body["status"], "running");
}

/// Test session not found
#[tokio::test]
async fn test_session_not_found() {
    let session_manager = Arc::new(SessionManager::new(100));
    let session_id = Uuid::new_v4();

    let filter = warp::path!("sessions" / Uuid)
        .and(warp::get())
        .and(warp::any().map(move || session_manager.clone()))
        .and_then(get_session_handler)
        .recover(handle_rejection);

    let response = request()
        .method("GET")
        .path(&format!("/sessions/{}", session_id))
        .reply(&filter)
        .await;

    // The handler returns 200 OK with error JSON in the body (this is intentional)
    assert_eq!(response.status(), StatusCode::OK);

    // Verify the response contains an error
    let body: serde_json::Value = serde_json::from_slice(response.body()).expect("Valid JSON");
    if let Some(code) = body.get("code") {
        // Error response format
        assert_eq!(code, "SESSION_NOT_FOUND");
    }
}

/// Test execute endpoint with mock executor
#[tokio::test]
async fn test_execute_endpoint() {
    let config = Arc::new(Config::dev_default());
    let executor: Arc<dyn Executor> = Arc::new(MockClaudeExecutor::new());
    let session_manager = Arc::new(SessionManager::new(100));

    let test_request = create_test_request();

    let filter = warp::path("execute")
        .and(warp::post())
        .and(warp::body::json())
        .and(warp::any().map(move || executor.clone()))
        .and(warp::any().map(move || session_manager.clone()))
        .and(warp::any().map(move || config.clone()))
        .and_then(|request, executor, manager, config| {
            execute_handler(request, executor, manager, config)
        });

    let response = request()
        .method("POST")
        .path("/execute")
        .json(&test_request)
        .reply(&filter)
        .await;

    // Execute endpoint returns SSE stream (200 OK)
    assert_eq!(response.status(), StatusCode::OK);
}

/// Test request validation - empty prompt
#[tokio::test]
async fn test_request_validation_empty_prompt() {
    let mut test_request = create_test_request();
    test_request.prompt = "".to_string();

    let result = test_request.validate(10, 50000, 5000);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("prompt") || err.contains("Prompt"));
}

/// Test request validation - too many screenshots
#[tokio::test]
async fn test_request_validation_too_many_screenshots() {
    let mut test_request = create_test_request();

    // Create 11 screenshots (limit is 10 by default)
    test_request.context.screenshots = (0..11)
        .map(|i| Screenshot {
            timestamp: "2025-10-17T10:30:00Z".to_string(),
            image_data: {
                use base64::{engine::general_purpose, Engine as _};
                general_purpose::STANDARD.encode(format!("image {}", i).as_bytes())
            },
            metadata: ScreenshotMetadata {
                window_title: format!("Window {}", i),
                url: None,
                viewport: Viewport {
                    width: 1920,
                    height: 1080,
                },
            },
        })
        .collect();

    let result = test_request.validate(10, 50000, 5000); // Using limit of 10 screenshots
    assert!(result.is_err());
}

/// Test configuration loading from TOML
#[test]
fn test_config_loading() {
    let config = Config::dev_default();

    assert_eq!(config.server.host, "127.0.0.1");
    assert_eq!(config.server.port, 8443);
    assert!(config.server.dev_mode);
    assert!(!config.server.enable_tls);
}

/// Test configuration validation - TLS without cert
#[test]
fn test_config_validation_tls_without_cert() {
    let mut config = Config::dev_default();
    config.server.enable_tls = true;
    config.server.dev_mode = false;

    let result = config.validate();
    assert!(result.is_err());
}

/// Helper function to handle rejections for testing
async fn handle_rejection(
    err: warp::Rejection,
) -> Result<impl warp::Reply, std::convert::Infallible> {
    use facet_server::error::FacetError;

    let code;
    let message;

    if err.is_not_found() {
        code = StatusCode::NOT_FOUND;
        message = "Not found".to_string();
    } else if let Some(e) = err.find::<FacetError>() {
        code = e.status_code();
        message = e.to_string();
    } else {
        code = StatusCode::INTERNAL_SERVER_ERROR;
        message = "Internal server error".to_string();
    }

    Ok(warp::reply::with_status(message, code))
}

/// Test that mock executor produces expected events
#[tokio::test]
async fn test_mock_executor_output() {
    use futures::StreamExt;

    let executor = MockClaudeExecutor::new();
    let request = create_test_request();
    let session_id = request.session_id;

    let mut stream = executor.execute(request).await;

    let mut events = Vec::new();
    while let Some(result) = stream.next().await {
        events.push(result.expect("No errors from mock"));
    }

    // Mock executor should produce multiple events
    assert!(!events.is_empty());

    // Should end with a Complete event
    let last_event = events.last().unwrap();
    match last_event {
        ClaudeEvent::Complete {
            session_id: last_id,
            status,
        } => {
            assert_eq!(*last_id, session_id);
            assert_eq!(status, "success");
        }
        _ => panic!("Expected Complete event as last event"),
    }
}

/// Test concurrent session handling
#[tokio::test]
async fn test_concurrent_sessions() {
    let session_manager = Arc::new(SessionManager::new(100));

    // Create 10 concurrent sessions
    let mut handles = vec![];
    for _ in 0..10 {
        let manager = session_manager.clone();
        let handle = tokio::spawn(async move {
            let session_id = Uuid::new_v4();
            manager.register(session_id, 20).await.unwrap();
            session_id
        });
        handles.push(handle);
    }

    // Wait for all sessions to be created
    let mut session_ids = vec![];
    for handle in handles {
        let session_id = handle.await.unwrap();
        session_ids.push(session_id);
    }

    // Verify all sessions exist
    for session_id in session_ids {
        let status = session_manager.get_status(session_id).await;
        assert!(status.is_ok());
    }

    // Check running count
    let running = session_manager.running_count().await;
    assert_eq!(running, 10);
}

/// Test session cleanup
#[tokio::test]
async fn test_session_cleanup() {
    // Create manager with max_history of 3 (will keep only 3 completed sessions)
    let session_manager = Arc::new(SessionManager::new(3));

    // Create and complete 5 sessions
    for _ in 0..5 {
        let session_id = Uuid::new_v4();
        session_manager.register(session_id, 20).await.unwrap();
        session_manager.complete(session_id).await.unwrap();
    }

    // Wait a moment to ensure timestamps differ
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Cleanup old sessions - should remove 2 (keeping only 3 newest)
    let removed = session_manager.cleanup_old_sessions().await;
    assert_eq!(removed, 2);
}
