//! Data models for facet-server API
//!
//! This module defines all request and response types used in the API.
//! All types are designed for efficient serialization/deserialization
//! and include comprehensive validation logic.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Screenshot metadata containing window and viewport information
///
/// Captures the context of where a screenshot was taken, including
/// window title, current URL (for web content), and viewport dimensions.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScreenshotMetadata {
    /// Window title or application name
    pub window_title: String,

    /// Current URL if screenshot is from a web browser
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    /// Viewport dimensions
    pub viewport: Viewport,
}

/// Viewport dimensions in pixels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Viewport {
    /// Width in pixels
    pub width: u32,

    /// Height in pixels
    pub height: u32,
}

/// Screenshot data with metadata
///
/// Contains base64-encoded PNG image data along with metadata
/// about when and where the screenshot was captured.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Screenshot {
    /// ISO 8601 timestamp when screenshot was captured
    pub timestamp: String,

    /// Base64-encoded PNG image data
    pub image_data: String,

    /// Screenshot metadata
    pub metadata: ScreenshotMetadata,
}

impl Screenshot {
    /// Validates the screenshot data
    ///
    /// Checks that:
    /// - timestamp is valid RFC3339
    /// - image_data is valid base64
    /// - viewport dimensions are reasonable
    ///
    /// # Returns
    /// Ok(()) if valid, Err with description if invalid
    pub fn validate(&self) -> Result<(), String> {
        // Validate timestamp
        chrono::DateTime::parse_from_rfc3339(&self.timestamp)
            .map_err(|e| format!("Invalid timestamp: {}", e))?;

        // Validate base64 image data
        if self.image_data.is_empty() {
            return Err("Empty image data".to_string());
        }
        use base64::{engine::general_purpose, Engine as _};
        general_purpose::STANDARD
            .decode(&self.image_data)
            .map_err(|e| format!("Invalid base64 image data: {}", e))?;

        // Validate viewport dimensions (must be reasonable)
        if self.metadata.viewport.width == 0 || self.metadata.viewport.height == 0 {
            return Err("Invalid viewport dimensions: width and height must be > 0".to_string());
        }

        if self.metadata.viewport.width > 10000 || self.metadata.viewport.height > 10000 {
            return Err("Invalid viewport dimensions: exceeds maximum size".to_string());
        }

        Ok(())
    }

    /// Returns the approximate size of the screenshot in bytes
    ///
    /// Calculates the decoded size of the base64 image data.
    /// Useful for enforcing request size limits.
    ///
    /// # Returns
    /// Approximate size in bytes
    pub fn size_bytes(&self) -> usize {
        // Base64 encoding increases size by ~33%, so decoded size is ~75% of encoded
        (self.image_data.len() * 3) / 4
    }
}

/// DOM state information
///
/// Contains the accessibility tree and list of interactive elements
/// from the current page or application state.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DomState {
    /// Serialized accessibility tree
    pub accessible_tree: String,

    /// List of interactive elements with selectors
    pub interactive_elements: Vec<HashMap<String, serde_json::Value>>,
}

/// Context information for the request
///
/// Aggregates screenshots, DOM state, and user intent to provide
/// complete context for Claude to understand the automation task.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RequestContext {
    /// List of screenshots (ordered chronologically)
    pub screenshots: Vec<Screenshot>,

    /// Current DOM/accessibility state
    pub dom_state: DomState,

    /// User's stated intent or goal
    pub user_intent: String,
}

impl RequestContext {
    /// Validates the context data
    ///
    /// Ensures all screenshots are valid and constraints are met.
    ///
    /// # Arguments
    /// * `max_screenshots` - Maximum allowed screenshots
    /// * `max_intent_length` - Maximum user intent string length
    ///
    /// # Returns
    /// Ok(()) if valid, Err with description if invalid
    pub fn validate(&self, max_screenshots: usize, max_intent_length: usize) -> Result<(), String> {
        if self.screenshots.is_empty() {
            return Err("At least one screenshot is required".to_string());
        }

        if self.screenshots.len() > max_screenshots {
            return Err(format!(
                "Too many screenshots: {} (max: {})",
                self.screenshots.len(),
                max_screenshots
            ));
        }

        for (i, screenshot) in self.screenshots.iter().enumerate() {
            screenshot
                .validate()
                .map_err(|e| format!("Screenshot {}: {}", i, e))?;
        }

        if self.user_intent.is_empty() {
            return Err("User intent cannot be empty".to_string());
        }

        if self.user_intent.len() > max_intent_length {
            return Err(format!(
                "User intent too long: {} chars (max: {})",
                self.user_intent.len(),
                max_intent_length
            ));
        }

        Ok(())
    }

    /// Calculates total size of all screenshots
    ///
    /// # Returns
    /// Total size in bytes
    pub fn total_screenshot_size(&self) -> usize {
        self.screenshots.iter().map(|s| s.size_bytes()).sum()
    }
}

/// Request options for execution
///
/// Configures timeout, token limits, and streaming behavior.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RequestOptions {
    /// Timeout in seconds (overrides server default)
    #[serde(default = "default_timeout")]
    pub timeout_seconds: u64,

    /// Maximum tokens for Claude response
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,

    /// Enable streaming response
    #[serde(default = "default_stream")]
    pub stream: bool,
}

fn default_timeout() -> u64 {
    300
}

fn default_max_tokens() -> u32 {
    100000
}

fn default_stream() -> bool {
    true
}

impl Default for RequestOptions {
    fn default() -> Self {
        Self {
            timeout_seconds: default_timeout(),
            max_tokens: default_max_tokens(),
            stream: default_stream(),
        }
    }
}

/// Main request payload for /api/v1/execute endpoint
///
/// Contains all information needed to execute a Claude CLI session,
/// including context, prompt, and execution options.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FacetRequest {
    /// Unique session identifier (UUIDv4)
    pub session_id: Uuid,

    /// Request context (screenshots, DOM, intent)
    pub context: RequestContext,

    /// User's prompt/question for Claude
    pub prompt: String,

    /// Execution options
    #[serde(default)]
    pub options: RequestOptions,
}

impl FacetRequest {
    /// Validates the entire request
    ///
    /// Performs comprehensive validation of all request fields
    /// against configured limits.
    ///
    /// # Arguments
    /// * `max_screenshots` - Maximum allowed screenshots
    /// * `max_prompt_length` - Maximum prompt string length
    /// * `max_intent_length` - Maximum user intent string length
    ///
    /// # Returns
    /// Ok(()) if valid, Err with description if invalid
    pub fn validate(
        &self,
        max_screenshots: usize,
        max_prompt_length: usize,
        max_intent_length: usize,
    ) -> Result<(), String> {
        self.context.validate(max_screenshots, max_intent_length)?;

        if self.prompt.is_empty() {
            return Err("Prompt cannot be empty".to_string());
        }

        if self.prompt.len() > max_prompt_length {
            return Err(format!(
                "Prompt too long: {} chars (max: {})",
                self.prompt.len(),
                max_prompt_length
            ));
        }

        if self.options.timeout_seconds == 0 {
            return Err("Timeout must be greater than 0".to_string());
        }

        if self.options.timeout_seconds > 3600 {
            return Err("Timeout cannot exceed 1 hour".to_string());
        }

        Ok(())
    }

    /// Estimates total request size in bytes
    ///
    /// Useful for enforcing maximum request size limits.
    ///
    /// # Returns
    /// Approximate size in bytes
    pub fn estimate_size(&self) -> usize {
        self.context.total_screenshot_size()
            + self.prompt.len()
            + self.context.user_intent.len()
            + self.context.dom_state.accessible_tree.len()
    }
}

/// Event types streamed from Claude CLI
///
/// Represents different types of events that can be sent
/// via Server-Sent Events (SSE) during execution.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClaudeEvent {
    /// Text content from Claude
    Content { text: String },

    /// Tool use event
    ToolUse {
        tool: String,
        params: serde_json::Value,
    },

    /// Error during execution
    Error { code: String, message: String },

    /// Execution complete
    Complete { session_id: Uuid, status: String },

    /// Progress update
    Progress { message: String, percent: u8 },
}

impl ClaudeEvent {
    /// Converts event to SSE format
    ///
    /// Formats the event for Server-Sent Events protocol with
    /// appropriate event type and data fields.
    ///
    /// # Returns
    /// String in SSE format: "event: type\ndata: json\n\n"
    pub fn to_sse(&self) -> String {
        let event_type = match self {
            ClaudeEvent::Content { .. } => "content",
            ClaudeEvent::ToolUse { .. } => "tool_use",
            ClaudeEvent::Error { .. } => "error",
            ClaudeEvent::Complete { .. } => "complete",
            ClaudeEvent::Progress { .. } => "progress",
        };

        let data = serde_json::to_string(self).unwrap_or_else(|_| "{}".to_string());

        format!("event: {}\ndata: {}\n\n", event_type, data)
    }
}

/// Health check response
///
/// Provides server status information including Claude CLI availability.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HealthResponse {
    /// Overall health status
    pub status: String,

    /// Server version
    pub version: String,

    /// Whether claude-cli is available and executable
    pub claude_cli_available: bool,

    /// Server uptime in seconds
    pub uptime_seconds: u64,
}

/// Session status information
///
/// Tracks the state of an execution session.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SessionStatus {
    /// Session UUID
    pub session_id: Uuid,

    /// Current status
    pub status: SessionState,

    /// When session started
    pub started_at: String,

    /// When session completed (if finished)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<String>,

    /// Error message (if failed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Session execution state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SessionState {
    /// Session is currently executing
    Running,

    /// Session completed successfully
    Completed,

    /// Session failed with error
    Failed,

    /// Session was cancelled
    Cancelled,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_valid_screenshot() -> Screenshot {
        use base64::{engine::general_purpose, Engine as _};
        Screenshot {
            timestamp: "2025-10-17T10:30:00Z".to_string(),
            image_data: general_purpose::STANDARD.encode(b"fake png data"),
            metadata: ScreenshotMetadata {
                window_title: "Test Window".to_string(),
                url: Some("https://example.com".to_string()),
                viewport: Viewport {
                    width: 1920,
                    height: 1080,
                },
            },
        }
    }

    fn create_valid_context() -> RequestContext {
        RequestContext {
            screenshots: vec![create_valid_screenshot()],
            dom_state: DomState {
                accessible_tree: "tree data".to_string(),
                interactive_elements: vec![],
            },
            user_intent: "Click the login button".to_string(),
        }
    }

    #[test]
    fn test_screenshot_validation_success() {
        let screenshot = create_valid_screenshot();
        assert!(screenshot.validate().is_ok());
    }

    #[test]
    fn test_screenshot_invalid_timestamp() {
        let mut screenshot = create_valid_screenshot();
        screenshot.timestamp = "invalid".to_string();
        assert!(screenshot.validate().is_err());
    }

    #[test]
    fn test_screenshot_empty_image_data() {
        let mut screenshot = create_valid_screenshot();
        screenshot.image_data = String::new();
        let result = screenshot.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Empty image data"));
    }

    #[test]
    fn test_screenshot_invalid_base64() {
        let mut screenshot = create_valid_screenshot();
        screenshot.image_data = "not-valid-base64!!!".to_string();
        let result = screenshot.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid base64"));
    }

    #[test]
    fn test_screenshot_zero_viewport_dimensions() {
        let mut screenshot = create_valid_screenshot();
        screenshot.metadata.viewport.width = 0;
        let result = screenshot.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("viewport dimensions"));
    }

    #[test]
    fn test_screenshot_excessive_viewport_dimensions() {
        let mut screenshot = create_valid_screenshot();
        screenshot.metadata.viewport.width = 20000;
        let result = screenshot.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("exceeds maximum size"));
    }

    #[test]
    fn test_screenshot_size_calculation() {
        let screenshot = create_valid_screenshot();
        let size = screenshot.size_bytes();
        assert!(size > 0);
        // Size should be approximately 75% of base64 encoded length
        assert!(size < screenshot.image_data.len());
    }

    #[test]
    fn test_context_validation_success() {
        let context = create_valid_context();
        assert!(context.validate(10, 1000).is_ok());
    }

    #[test]
    fn test_context_no_screenshots() {
        let mut context = create_valid_context();
        context.screenshots.clear();
        let result = context.validate(10, 1000);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("At least one screenshot"));
    }

    #[test]
    fn test_context_too_many_screenshots() {
        let mut context = create_valid_context();
        context.screenshots = vec![create_valid_screenshot(); 15];
        let result = context.validate(10, 1000);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Too many screenshots"));
    }

    #[test]
    fn test_context_empty_user_intent() {
        let mut context = create_valid_context();
        context.user_intent = String::new();
        let result = context.validate(10, 1000);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("User intent cannot be empty"));
    }

    #[test]
    fn test_context_user_intent_too_long() {
        let mut context = create_valid_context();
        context.user_intent = "a".repeat(2000);
        let result = context.validate(10, 1000);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("User intent too long"));
    }

    #[test]
    fn test_request_validation_success() {
        let request = FacetRequest {
            session_id: Uuid::new_v4(),
            context: create_valid_context(),
            prompt: "Test prompt".to_string(),
            options: RequestOptions::default(),
        };
        assert!(request.validate(10, 50000, 1000).is_ok());
    }

    #[test]
    fn test_request_empty_prompt() {
        let request = FacetRequest {
            session_id: Uuid::new_v4(),
            context: create_valid_context(),
            prompt: String::new(),
            options: RequestOptions::default(),
        };
        let result = request.validate(10, 50000, 1000);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Prompt cannot be empty"));
    }

    #[test]
    fn test_request_prompt_too_long() {
        let request = FacetRequest {
            session_id: Uuid::new_v4(),
            context: create_valid_context(),
            prompt: "a".repeat(60000),
            options: RequestOptions::default(),
        };
        let result = request.validate(10, 50000, 1000);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Prompt too long"));
    }

    #[test]
    fn test_request_zero_timeout() {
        let request = FacetRequest {
            session_id: Uuid::new_v4(),
            context: create_valid_context(),
            prompt: "test".to_string(),
            options: RequestOptions {
                timeout_seconds: 0,
                ..Default::default()
            },
        };
        let result = request.validate(10, 50000, 1000);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Timeout must be greater than 0"));
    }

    #[test]
    fn test_request_excessive_timeout() {
        let request = FacetRequest {
            session_id: Uuid::new_v4(),
            context: create_valid_context(),
            prompt: "test".to_string(),
            options: RequestOptions {
                timeout_seconds: 7200,
                ..Default::default()
            },
        };
        let result = request.validate(10, 50000, 1000);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("cannot exceed 1 hour"));
    }

    #[test]
    fn test_request_options_defaults() {
        let options = RequestOptions::default();
        assert_eq!(options.timeout_seconds, 300);
        assert_eq!(options.max_tokens, 100000);
        assert!(options.stream);
    }

    #[test]
    fn test_claude_event_to_sse_content() {
        let event = ClaudeEvent::Content {
            text: "Hello".to_string(),
        };
        let sse = event.to_sse();
        assert!(sse.contains("event: content"));
        assert!(sse.contains("data: "));
        assert!(sse.contains("Hello"));
        assert!(sse.ends_with("\n\n"));
    }

    #[test]
    fn test_claude_event_to_sse_tool_use() {
        let event = ClaudeEvent::ToolUse {
            tool: "cdp_command".to_string(),
            params: serde_json::json!({"command": "click"}),
        };
        let sse = event.to_sse();
        assert!(sse.contains("event: tool_use"));
        assert!(sse.contains("cdp_command"));
    }

    #[test]
    fn test_claude_event_to_sse_error() {
        let event = ClaudeEvent::Error {
            code: "TEST_ERROR".to_string(),
            message: "Test message".to_string(),
        };
        let sse = event.to_sse();
        assert!(sse.contains("event: error"));
        assert!(sse.contains("TEST_ERROR"));
    }

    #[test]
    fn test_claude_event_to_sse_complete() {
        let event = ClaudeEvent::Complete {
            session_id: Uuid::new_v4(),
            status: "success".to_string(),
        };
        let sse = event.to_sse();
        assert!(sse.contains("event: complete"));
        assert!(sse.contains("success"));
    }

    #[test]
    fn test_session_status_serialization() {
        let status = SessionStatus {
            session_id: Uuid::new_v4(),
            status: SessionState::Running,
            started_at: "2025-10-17T10:30:00Z".to_string(),
            completed_at: None,
            error: None,
        };
        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("running"));
    }

    #[test]
    fn test_health_response_serialization() {
        let health = HealthResponse {
            status: "healthy".to_string(),
            version: "1.0.0".to_string(),
            claude_cli_available: true,
            uptime_seconds: 12345,
        };
        let json = serde_json::to_string(&health).unwrap();
        assert!(json.contains("healthy"));
        assert!(json.contains("1.0.0"));
    }

    #[test]
    fn test_request_deserialization_with_defaults() {
        let json = r#"{
            "session_id": "550e8400-e29b-41d4-a716-446655440000",
            "context": {
                "screenshots": [{
                    "timestamp": "2025-10-17T10:30:00Z",
                    "image_data": "dGVzdA==",
                    "metadata": {
                        "window_title": "Test",
                        "viewport": {"width": 1920, "height": 1080}
                    }
                }],
                "dom_state": {
                    "accessible_tree": "tree",
                    "interactive_elements": []
                },
                "user_intent": "test intent"
            },
            "prompt": "test prompt"
        }"#;

        let request: FacetRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.options.timeout_seconds, 300);
        assert_eq!(request.options.max_tokens, 100000);
        assert!(request.options.stream);
    }
}
