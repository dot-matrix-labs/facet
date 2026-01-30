//! Execute endpoint for running Claude CLI requests
//!
//! Handles POST /api/v1/execute with streaming SSE responses.

use crate::claude::Executor;
use crate::config::Config;
use crate::error::FacetError;
use crate::models::{ClaudeEvent, FacetRequest};
use crate::session::SessionManager;
use futures::StreamExt;
use std::convert::Infallible;
use std::sync::Arc;
use warp::Reply;

/// Execute endpoint handler
///
/// Processes a Facet request and returns a stream of Claude events via SSE.
///
/// # Arguments
/// * `request` - Validated Facet request
/// * `executor` - Claude executor (real or mock)
/// * `session_manager` - Session tracking
/// * `config` - Server configuration for validation limits
///
/// # Returns
/// Server-Sent Events stream of Claude events
pub async fn execute_handler(
    request: FacetRequest,
    executor: Arc<dyn Executor>,
    session_manager: Arc<SessionManager>,
    config: Arc<Config>,
) -> Result<impl Reply, warp::Rejection> {
    let session_id = request.session_id;

    // Validate request against configured limits
    if let Err(e) = request.validate(
        config.limits.max_screenshot_count,
        config.limits.max_prompt_length,
        50000, // max intent length - could be configurable
    ) {
        return Err(warp::reject::custom(crate::auth::AuthRejection(
            FacetError::InvalidRequest(e),
        )));
    }

    // Register session
    if let Err(e) = session_manager
        .register(session_id, config.claude.max_concurrent_sessions)
        .await
    {
        return Err(warp::reject::custom(crate::auth::AuthRejection(e)));
    }

    // Execute request and get event stream
    let mut event_stream = executor.execute(request).await;

    // Convert to SSE stream
    let session_manager_clone = session_manager.clone();
    let sse_stream = async_stream::stream! {
        while let Some(result) = event_stream.next().await {
            match result {
                Ok(event) => {
                    // Check if this is a terminal event
                    let is_complete = matches!(event, ClaudeEvent::Complete { .. });
                    let is_error = matches!(event, ClaudeEvent::Error { .. });

                    // Update session status
                    if is_complete {
                        let _ = session_manager_clone.complete(session_id).await;
                    } else if is_error {
                        if let ClaudeEvent::Error { message, .. } = &event {
                            let _ = session_manager_clone.fail(session_id, message.clone()).await;
                        }
                    }

                    // Convert event to SSE format
                    let sse_data = event.to_sse();
                    yield Ok::<_, Infallible>(warp::sse::Event::default().data(sse_data));
                }
                Err(e) => {
                    // Convert error to SSE error event
                    let error_event = ClaudeEvent::Error {
                        code: e.error_code(),
                        message: e.to_string(),
                    };
                    let sse_data = error_event.to_sse();
                    yield Ok::<_, Infallible>(warp::sse::Event::default().data(sse_data));

                    // Mark session as failed
                    let _ = session_manager_clone.fail(session_id, e.to_string()).await;
                    break;
                }
            }
        }
    };

    Ok(warp::sse::reply(warp::sse::keep_alive().stream(sse_stream)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::claude::MockClaudeExecutor;
    use crate::models::{
        DomState, RequestContext, RequestOptions, Screenshot, ScreenshotMetadata, Viewport,
    };
    use uuid::Uuid;

    fn create_test_request() -> FacetRequest {
        FacetRequest {
            session_id: Uuid::new_v4(),
            context: RequestContext {
                screenshots: vec![Screenshot {
                    timestamp: "2025-10-17T10:30:00Z".to_string(),
                    image_data: {
                        use base64::{engine::general_purpose, Engine as _};
                        general_purpose::STANDARD.encode(b"test image")
                    },
                    metadata: ScreenshotMetadata {
                        window_title: "Test".to_string(),
                        url: Some("https://test.com".to_string()),
                        viewport: Viewport {
                            width: 1920,
                            height: 1080,
                        },
                    },
                }],
                dom_state: DomState {
                    accessible_tree: "test tree".to_string(),
                    interactive_elements: vec![],
                },
                user_intent: "test intent".to_string(),
            },
            prompt: "test prompt".to_string(),
            options: RequestOptions::default(),
        }
    }

    #[tokio::test]
    async fn test_execute_handler_with_mock() {
        let config = Arc::new(Config::dev_default());
        let executor: Arc<dyn Executor> = Arc::new(MockClaudeExecutor::with_delay(10));
        let session_manager = Arc::new(SessionManager::new(100));
        let request = create_test_request();
        let session_id = request.session_id;

        let result = execute_handler(request, executor, session_manager.clone(), config).await;
        assert!(result.is_ok());

        // Give time for async processing
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

        // Check session was registered and completed
        let status = session_manager.get_status(session_id).await;
        assert!(status.is_ok());
    }

    #[tokio::test]
    async fn test_execute_handler_validation_failure() {
        let config = Arc::new(Config::dev_default());
        let executor: Arc<dyn Executor> = Arc::new(MockClaudeExecutor::with_delay(10));
        let session_manager = Arc::new(SessionManager::new(100));

        let mut request = create_test_request();
        // Make prompt too long
        request.prompt = "a".repeat(100000);

        let result = execute_handler(request, executor, session_manager, config).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_execute_handler_concurrent_limit() {
        let config = Arc::new(Config::dev_default());
        let executor: Arc<dyn Executor> = Arc::new(MockClaudeExecutor::with_delay(10));
        let session_manager = Arc::new(SessionManager::new(100));

        // Register max concurrent sessions
        for _ in 0..config.claude.max_concurrent_sessions {
            let session_id = Uuid::new_v4();
            session_manager
                .register(session_id, config.claude.max_concurrent_sessions)
                .await
                .unwrap();
        }

        // Next request should fail
        let request = create_test_request();
        let result = execute_handler(request, executor, session_manager, config).await;
        assert!(result.is_err());
    }
}
