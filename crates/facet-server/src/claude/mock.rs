//! Mock Claude CLI executor for testing
//!
//! Provides a simulated executor that returns predefined responses
//! without requiring claude-cli to be installed. Useful for rapid
//! development and automated testing.

use crate::claude::Executor;
use crate::error::FacetError;
use crate::models::{ClaudeEvent, FacetRequest};
use async_stream::stream;
use futures::Stream;

/// Mock executor that returns predefined responses
///
/// Simulates Claude CLI execution by generating synthetic events.
/// Includes configurable delay to simulate processing time.
#[derive(Debug, Clone)]
pub struct MockClaudeExecutor {
    /// Delay between events in milliseconds
    event_delay_ms: u64,

    /// Whether to simulate an error
    should_fail: bool,
}

impl MockClaudeExecutor {
    /// Creates a new mock executor with default settings
    ///
    /// # Returns
    /// MockClaudeExecutor with 100ms delay between events
    pub fn new() -> Self {
        Self {
            event_delay_ms: 100,
            should_fail: false,
        }
    }

    /// Creates a mock executor with custom delay
    ///
    /// # Arguments
    /// * `delay_ms` - Milliseconds to wait between events
    ///
    /// # Returns
    /// MockClaudeExecutor with specified delay
    pub fn with_delay(delay_ms: u64) -> Self {
        Self {
            event_delay_ms: delay_ms,
            should_fail: false,
        }
    }

    /// Creates a mock executor that simulates failure
    ///
    /// # Returns
    /// MockClaudeExecutor that will emit an error event
    pub fn with_failure() -> Self {
        Self {
            event_delay_ms: 100,
            should_fail: true,
        }
    }
}

impl Default for MockClaudeExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl Executor for MockClaudeExecutor {
    async fn execute(
        &self,
        request: FacetRequest,
    ) -> Box<dyn Stream<Item = Result<ClaudeEvent, FacetError>> + Send + Unpin + 'static> {
        let delay_ms = self.event_delay_ms;
        let should_fail = self.should_fail;
        let session_id = request.session_id;

        let stream = stream! {
            // Simulate processing delay
            tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;

            if should_fail {
                // Emit error event
                yield Ok(ClaudeEvent::Error {
                    code: "MOCK_ERROR".to_string(),
                    message: "Simulated failure for testing".to_string(),
                });
                yield Ok(ClaudeEvent::Complete {
                    session_id,
                    status: "failed".to_string(),
                });
                return;
            }

            // Emit content event
            yield Ok(ClaudeEvent::Content {
                text: "Mock: Analyzing screenshot...".to_string(),
            });

            tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;

            // Emit progress event
            yield Ok(ClaudeEvent::Progress {
                message: "Processing request".to_string(),
                percent: 50,
            });

            tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;

            // Emit tool use event
            yield Ok(ClaudeEvent::ToolUse {
                tool: "cdp_command".to_string(),
                params: serde_json::json!({
                    "command": "click",
                    "selector": "#submit-button"
                }),
            });

            tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;

            // Emit another content event
            yield Ok(ClaudeEvent::Content {
                text: "Mock: Task completed successfully".to_string(),
            });

            tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;

            // Emit complete event
            yield Ok(ClaudeEvent::Complete {
                session_id,
                status: "success".to_string(),
            });
        };

        Box::new(Box::pin(stream))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{
        DomState, RequestContext, RequestOptions, Screenshot, ScreenshotMetadata, Viewport,
    };
    use futures::StreamExt;
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
    async fn test_mock_executor_success() {
        let executor = MockClaudeExecutor::with_delay(10); // Fast for testing
        let request = create_test_request();
        let session_id = request.session_id;

        let mut stream = executor.execute(request).await;

        // Collect all events
        let mut events = Vec::new();
        while let Some(result) = stream.next().await {
            assert!(result.is_ok());
            events.push(result.unwrap());
        }

        // Should have multiple events
        assert!(events.len() >= 4);

        // First event should be content
        assert!(matches!(events[0], ClaudeEvent::Content { .. }));

        // Last event should be complete
        if let ClaudeEvent::Complete {
            session_id: sid,
            status,
        } = &events[events.len() - 1]
        {
            assert_eq!(*sid, session_id);
            assert_eq!(status, "success");
        } else {
            panic!("Last event should be Complete");
        }
    }

    #[tokio::test]
    async fn test_mock_executor_emits_tool_use() {
        let executor = MockClaudeExecutor::with_delay(10);
        let request = create_test_request();

        let mut stream = executor.execute(request).await;

        // Collect all events
        let mut has_tool_use = false;
        while let Some(result) = stream.next().await {
            if let Ok(ClaudeEvent::ToolUse { .. }) = result {
                has_tool_use = true;
            }
        }

        assert!(has_tool_use, "Should emit at least one ToolUse event");
    }

    #[tokio::test]
    async fn test_mock_executor_emits_progress() {
        let executor = MockClaudeExecutor::with_delay(10);
        let request = create_test_request();

        let mut stream = executor.execute(request).await;

        // Collect all events
        let mut has_progress = false;
        while let Some(result) = stream.next().await {
            if let Ok(ClaudeEvent::Progress { .. }) = result {
                has_progress = true;
            }
        }

        assert!(has_progress, "Should emit at least one Progress event");
    }

    #[tokio::test]
    async fn test_mock_executor_with_failure() {
        let executor = MockClaudeExecutor::with_failure();
        let request = create_test_request();

        let mut stream = executor.execute(request).await;

        // Collect all events
        let mut events = Vec::new();
        while let Some(result) = stream.next().await {
            assert!(result.is_ok());
            events.push(result.unwrap());
        }

        // Should have error event
        let has_error = events
            .iter()
            .any(|e| matches!(e, ClaudeEvent::Error { .. }));
        assert!(has_error, "Should emit Error event when configured to fail");

        // Last event should be complete with failed status
        if let ClaudeEvent::Complete { status, .. } = &events[events.len() - 1] {
            assert_eq!(status, "failed");
        } else {
            panic!("Last event should be Complete");
        }
    }

    #[tokio::test]
    async fn test_mock_executor_respects_delay() {
        let delay_ms = 50;
        let executor = MockClaudeExecutor::with_delay(delay_ms);
        let request = create_test_request();

        let start = std::time::Instant::now();
        let mut stream = executor.execute(request).await;

        // Consume first event
        let _ = stream.next().await;

        let elapsed = start.elapsed().as_millis();
        // Should take at least the delay time
        assert!(
            elapsed >= delay_ms as u128,
            "Expected delay of at least {}ms, got {}ms",
            delay_ms,
            elapsed
        );
    }

    #[test]
    fn test_mock_executor_new() {
        let executor = MockClaudeExecutor::new();
        assert_eq!(executor.event_delay_ms, 100);
        assert!(!executor.should_fail);
    }

    #[test]
    fn test_mock_executor_default() {
        let executor = MockClaudeExecutor::default();
        assert_eq!(executor.event_delay_ms, 100);
        assert!(!executor.should_fail);
    }

    #[tokio::test]
    async fn test_mock_executor_session_id_preservation() {
        let executor = MockClaudeExecutor::with_delay(10);
        let request = create_test_request();
        let session_id = request.session_id;

        let mut stream = executor.execute(request).await;

        // Find complete event
        while let Some(result) = stream.next().await {
            if let Ok(ClaudeEvent::Complete {
                session_id: sid, ..
            }) = result
            {
                assert_eq!(sid, session_id, "Session ID should be preserved");
                return;
            }
        }

        panic!("Should have received Complete event");
    }
}
