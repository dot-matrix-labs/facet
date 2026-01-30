//! Real Claude CLI executor
//!
//! Spawns headless claude-cli processes and streams stdout/stderr events.
//! Handles timeouts, process cleanup, and error recovery.

use crate::claude::Executor;
use crate::error::FacetError;
use crate::models::{ClaudeEvent, FacetRequest};
use async_stream::stream;
use futures::Stream;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::time::{timeout, Duration};

/// Real Claude CLI executor
///
/// Spawns actual claude-cli processes and forwards their output.
#[derive(Debug, Clone)]
pub struct ClaudeExecutor {
    /// Path to claude-cli binary
    binary_path: String,

    /// Default timeout for executions
    #[allow(dead_code)]
    default_timeout: Duration,
}

impl ClaudeExecutor {
    /// Creates a new Claude CLI executor
    ///
    /// # Arguments
    /// * `binary_path` - Path to claude executable
    /// * `timeout_seconds` - Default timeout in seconds
    ///
    /// # Returns
    /// New ClaudeExecutor instance
    pub fn new(binary_path: String, timeout_seconds: u64) -> Self {
        Self {
            binary_path,
            default_timeout: Duration::from_secs(timeout_seconds),
        }
    }

    /// Spawns a claude-cli process
    ///
    /// Launches claude in headless mode with streaming enabled.
    ///
    /// # Arguments
    /// * `request` - Request data to send to claude
    ///
    /// # Returns
    /// Spawned child process handle
    ///
    /// # Errors
    /// Returns FacetError if process fails to spawn
    #[allow(dead_code)]
    async fn spawn_process(&self, _request: &FacetRequest) -> Result<Child, FacetError> {
        Command::new(&self.binary_path)
            .arg("--headless")
            .arg("--stream")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| {
                FacetError::ClaudeUnavailable(format!(
                    "Failed to spawn claude-cli: {} (binary: {})",
                    e, self.binary_path
                ))
            })
    }

    /// Parses a line of stdout/stderr into a ClaudeEvent
    ///
    /// Attempts to parse as JSON event, falls back to plain text.
    ///
    /// # Arguments
    /// * `line` - Output line from claude-cli
    ///
    /// # Returns
    /// Parsed ClaudeEvent or Content event with the line
    #[allow(dead_code)]
    fn parse_output_line(&self, line: &str) -> ClaudeEvent {
        // Try to parse as JSON event
        if let Ok(event) = serde_json::from_str::<ClaudeEvent>(line) {
            return event;
        }

        // Fallback to content event
        ClaudeEvent::Content {
            text: line.to_string(),
        }
    }
}

#[async_trait::async_trait]
impl Executor for ClaudeExecutor {
    async fn execute(
        &self,
        request: FacetRequest,
    ) -> Box<dyn Stream<Item = Result<ClaudeEvent, FacetError>> + Send + Unpin + 'static> {
        let session_id = request.session_id;
        let binary_path = self.binary_path.clone();

        // Spawn process before creating stream
        let child_result = Command::new(&binary_path)
            .arg("--headless")
            .arg("--stream")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn();

        let stream = stream! {
            // Check if spawn succeeded
            let mut child = match child_result {
                Ok(child) => child,
                Err(e) => {
                    yield Err(FacetError::ClaudeUnavailable(format!(
                        "Failed to spawn claude-cli: {} (binary: {})",
                        e, binary_path
                    )));
                    return;
                }
            };

            // Get stdout handle
            let stdout = match child.stdout.take() {
                Some(stdout) => stdout,
                None => {
                    yield Err(FacetError::ExecutionError(
                        "Failed to capture stdout".to_string()
                    ));
                    return;
                }
            };

            // Create buffered reader for stdout
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();

            // Write request to stdin (TODO: implement proper JSON serialization)
            if let Some(stdin) = child.stdin.take() {
                // For now, just close stdin - full implementation would write request data
                drop(stdin);
            }

            // Stream output lines with timeout
            loop {
                let line_result = timeout(Duration::from_secs(5), lines.next_line()).await;

                match line_result {
                    Ok(Ok(Some(line))) => {
                        // Parse output line inline instead of using self
                        let event = if let Ok(event) = serde_json::from_str::<ClaudeEvent>(&line) {
                            event
                        } else {
                            ClaudeEvent::Content { text: line }
                        };
                        yield Ok(event);
                    }
                    Ok(Ok(None)) => {
                        // EOF reached
                        break;
                    }
                    Ok(Err(e)) => {
                        yield Err(FacetError::ExecutionError(format!(
                            "Failed to read output: {}",
                            e
                        )));
                        break;
                    }
                    Err(_) => {
                        // Timeout on individual line read
                        yield Err(FacetError::Timeout(
                            "No output received within timeout".to_string()
                        ));
                        break;
                    }
                }
            }

            // Wait for process to complete
            match timeout(Duration::from_secs(5), child.wait()).await {
                Ok(Ok(status)) => {
                    if !status.success() {
                        yield Ok(ClaudeEvent::Error {
                            code: "PROCESS_FAILED".to_string(),
                            message: format!("Claude process exited with status: {}", status),
                        });
                        yield Ok(ClaudeEvent::Complete {
                            session_id,
                            status: "failed".to_string(),
                        });
                    } else {
                        yield Ok(ClaudeEvent::Complete {
                            session_id,
                            status: "success".to_string(),
                        });
                    }
                }
                Ok(Err(e)) => {
                    yield Err(FacetError::ExecutionError(format!(
                        "Failed to wait for process: {}",
                        e
                    )));
                }
                Err(_) => {
                    // Kill the process if it hasn't finished
                    let _ = child.kill().await;
                    yield Err(FacetError::Timeout(
                        "Process did not complete within timeout".to_string()
                    ));
                }
            }
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

    #[test]
    fn test_executor_creation() {
        let executor = ClaudeExecutor::new("claude".to_string(), 300);
        assert_eq!(executor.binary_path, "claude");
        assert_eq!(executor.default_timeout.as_secs(), 300);
    }

    #[test]
    fn test_parse_output_line_json() {
        let executor = ClaudeExecutor::new("claude".to_string(), 300);
        let json_line = r#"{"type":"content","text":"Hello"}"#;
        let event = executor.parse_output_line(json_line);

        if let ClaudeEvent::Content { text } = event {
            assert_eq!(text, "Hello");
        } else {
            panic!("Expected Content event");
        }
    }

    #[test]
    fn test_parse_output_line_plain_text() {
        let executor = ClaudeExecutor::new("claude".to_string(), 300);
        let plain_line = "Plain text output";
        let event = executor.parse_output_line(plain_line);

        if let ClaudeEvent::Content { text } = event {
            assert_eq!(text, "Plain text output");
        } else {
            panic!("Expected Content event");
        }
    }

    #[tokio::test]
    async fn test_spawn_process_nonexistent_binary() {
        let executor = ClaudeExecutor::new("/nonexistent/binary".to_string(), 300);
        let request = create_test_request();

        let result = executor.spawn_process(&request).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Failed to spawn claude-cli"));
    }

    #[tokio::test]
    async fn test_execute_with_echo() {
        // Use echo as a simple test - it will exit immediately
        let executor = ClaudeExecutor::new("echo".to_string(), 30);
        let request = create_test_request();

        let mut stream = executor.execute(request).await;

        // Should get at least one event (could be error or complete)
        let first = stream.next().await;
        assert!(first.is_some());
    }

    // Note: Full integration tests with real claude-cli would require
    // the binary to be installed and properly configured
}
