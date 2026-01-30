pub mod health;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Stdio;
use thiserror::Error;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

/// Errors that can occur when calling Claude CLI
#[derive(Debug, Error)]
#[allow(dead_code)] // Some variants are defined for future use
pub enum ClaudeError {
    #[error("Claude CLI not found or not executable")]
    NotInstalled,

    #[error("Claude CLI not authenticated. Run 'claude' to authenticate or 'claude setup-token' for headless mode")]
    NotAuthenticated,

    #[error("Claude CLI permission denied: {0}")]
    PermissionDenied(String),

    #[error("Claude CLI rate limit exceeded: {0}")]
    RateLimitExceeded(String),

    #[error("Claude CLI model not available: {0}")]
    ModelNotAvailable(String),

    #[error("Claude CLI invalid input: {0}")]
    InvalidInput(String),

    #[error("Claude CLI invalid output format: {0}")]
    InvalidOutputFormat(String),

    #[error("Claude CLI timeout: operation took too long")]
    Timeout,

    #[error("Claude CLI process error: {0}")]
    ProcessError(String),

    #[error("Failed to parse Claude CLI output: {0}")]
    ParseError(String),

    #[error("Claude CLI returned non-zero exit code {code}: {stderr}")]
    CommandFailed { code: i32, stderr: String },

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Other error: {0}")]
    Other(String),
}

/// Configuration for Claude CLI invocation
#[derive(Debug, Clone, Default)]
pub struct ClaudeConfig {
    /// Path to the claude CLI executable (defaults to "claude" in PATH)
    pub claude_path: Option<PathBuf>,
    /// Whether to skip permissions (use only in trusted sandboxes)
    #[allow(dead_code)]
    pub skip_permissions: bool,
    /// Model to use (e.g., "sonnet", "opus", or full model name)
    pub model: Option<String>,
    /// Additional directories to allow tool access to
    #[allow(dead_code)]
    pub allowed_dirs: Vec<PathBuf>,
    /// Tools to allow (e.g., ["Bash(git:*)", "Edit"])
    #[allow(dead_code)]
    pub allowed_tools: Vec<String>,
    /// Tools to disallow
    #[allow(dead_code)]
    pub disallowed_tools: Vec<String>,
}

/// Input message for Claude CLI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeInput {
    /// Text prompt
    pub prompt: String,
    /// Optional image paths (screenshots)
    pub images: Vec<PathBuf>,
    /// Optional HTML content
    pub html: Option<String>,
}

/// Response from Claude CLI (JSON format)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ClaudeResponse {
    /// Response type
    #[serde(rename = "type")]
    pub response_type: String,

    /// Response subtype
    #[serde(default)]
    pub subtype: Option<String>,

    /// Whether this is an error response
    #[serde(default)]
    pub is_error: bool,

    /// The result/response text from Claude
    pub result: String,

    /// Session ID
    #[serde(default)]
    pub session_id: Option<String>,

    /// Duration in milliseconds
    #[serde(default)]
    pub duration_ms: Option<u64>,

    /// Usage information
    #[serde(default)]
    pub usage: Option<serde_json::Value>,

    /// Permission denials
    #[serde(default)]
    pub permission_denials: Option<Vec<serde_json::Value>>,
}

impl ClaudeResponse {
    /// Get the text content (for backwards compatibility)
    pub fn text(&self) -> &str {
        &self.result
    }
}

/// Streaming response chunk from Claude CLI
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct ClaudeStreamChunk {
    /// Type of chunk (e.g., "content", "metadata")
    #[serde(rename = "type")]
    pub chunk_type: String,
    /// Content of the chunk
    pub content: Option<String>,
    /// Metadata for the chunk
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
}

/// Analyze result message from JSON response and classify the error
fn classify_claude_error_from_result(result: &str, stderr: &str, exit_code: i32) -> ClaudeError {
    let result_lower = result.to_lowercase();

    // Check for rate limiting (e.g., "5-hour limit reached")
    if result_lower.contains("limit reached")
        || result_lower.contains("rate limit")
        || result_lower.contains("hour limit")
        || result_lower.contains("usage limit")
    {
        return ClaudeError::RateLimitExceeded(result.to_string());
    }

    // Check for authentication issues
    if result_lower.contains("not authenticated")
        || result_lower.contains("authentication")
        || result_lower.contains("login")
        || result_lower.contains("please sign in")
    {
        return ClaudeError::NotAuthenticated;
    }

    // Check for permission issues
    if result_lower.contains("permission denied")
        || result_lower.contains("access denied")
        || result_lower.contains("unauthorized")
    {
        return ClaudeError::PermissionDenied(result.to_string());
    }

    // Check for model availability
    if result_lower.contains("model")
        && (result_lower.contains("not available") || result_lower.contains("not found"))
    {
        return ClaudeError::ModelNotAvailable(result.to_string());
    }

    // Check for timeout
    if result_lower.contains("timeout") || result_lower.contains("timed out") {
        return ClaudeError::Timeout;
    }

    // Fall back to stderr-based classification
    classify_claude_error(stderr, exit_code)
}

/// Analyze stderr output and classify the error
fn classify_claude_error(stderr: &str, exit_code: i32) -> ClaudeError {
    let stderr_lower = stderr.to_lowercase();

    // Check for authentication issues
    if stderr_lower.contains("not authenticated")
        || stderr_lower.contains("authentication")
        || stderr_lower.contains("login")
        || stderr_lower.contains("please sign in")
    {
        return ClaudeError::NotAuthenticated;
    }

    // Check for permission issues
    if stderr_lower.contains("permission denied")
        || stderr_lower.contains("access denied")
        || stderr_lower.contains("unauthorized")
    {
        return ClaudeError::PermissionDenied(stderr.to_string());
    }

    // Check for rate limiting
    if stderr_lower.contains("rate limit")
        || stderr_lower.contains("too many requests")
        || stderr_lower.contains("429")
    {
        return ClaudeError::RateLimitExceeded(stderr.to_string());
    }

    // Check for model availability
    if stderr_lower.contains("model")
        && (stderr_lower.contains("not available") || stderr_lower.contains("not found"))
    {
        return ClaudeError::ModelNotAvailable(stderr.to_string());
    }

    // Check for invalid input
    if stderr_lower.contains("invalid")
        && (stderr_lower.contains("input") || stderr_lower.contains("argument"))
    {
        return ClaudeError::InvalidInput(stderr.to_string());
    }

    // Check for timeout
    if stderr_lower.contains("timeout") || stderr_lower.contains("timed out") {
        return ClaudeError::Timeout;
    }

    // Default to command failed with code
    ClaudeError::CommandFailed {
        code: exit_code,
        stderr: stderr.to_string(),
    }
}

/// Claude CLI client for headless automation
pub struct ClaudeClient {
    config: ClaudeConfig,
}

impl ClaudeClient {
    /// Create a new Claude client with default configuration
    pub fn new() -> Self {
        Self {
            config: ClaudeConfig::default(),
        }
    }

    /// Create a new Claude client with custom configuration
    pub fn with_config(config: ClaudeConfig) -> Self {
        Self { config }
    }

    /// Execute Claude CLI with the given input and return the response
    pub async fn execute(&self, input: ClaudeInput) -> Result<ClaudeResponse> {
        log::debug!("🔮 Executing Claude CLI...");
        log::debug!("Images count: {}", input.images.len());
        log::debug!("HTML provided: {}", input.html.is_some());

        let output = self.run_command(&input, false).await?;

        log::debug!("Claude CLI output length: {} bytes", output.len());
        log::debug!(
            "Claude CLI raw output (first 500 chars): {}",
            &output.chars().take(500).collect::<String>()
        );

        // Parse JSON response
        let response: ClaudeResponse =
            serde_json::from_str(&output).context("Failed to parse Claude CLI response as JSON")?;

        log::info!("✅ Claude CLI executed successfully");
        log::debug!("Response text length: {} chars", response.text().len());

        Ok(response)
    }

    /// Execute Claude CLI with streaming output
    #[allow(dead_code)]
    pub async fn execute_streaming<F>(&self, input: ClaudeInput, mut callback: F) -> Result<()>
    where
        F: FnMut(ClaudeStreamChunk) -> Result<()>,
    {
        let mut child = self.spawn_streaming_command(&input).await?;

        // Get stdout
        let stdout = child.stdout.take().context("Failed to capture stdout")?;

        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();

        // Process each line as it arrives
        while let Some(line) = lines.next_line().await? {
            if line.trim().is_empty() {
                continue;
            }

            // Parse each line as a JSON chunk
            match serde_json::from_str::<ClaudeStreamChunk>(&line) {
                Ok(chunk) => {
                    callback(chunk)?;
                }
                Err(e) => {
                    eprintln!(
                        "Warning: Failed to parse stream chunk: {} - Line: {}",
                        e, line
                    );
                }
            }
        }

        // Wait for the process to complete
        let status = child.wait().await?;
        if !status.success() {
            anyhow::bail!("Claude CLI exited with status: {}", status);
        }

        Ok(())
    }

    /// Build the prompt with HTML context
    /// Note: Images are NOT included in the prompt text for Claude CLI.
    /// Claude CLI in headless mode doesn't support passing images directly.
    /// For now, we'll just include HTML and the prompt text.
    fn build_prompt(&self, input: &ClaudeInput) -> String {
        let mut prompt = String::new();

        // Note about images if provided (but can't actually use them yet)
        if !input.images.is_empty() {
            log::warn!(
                "⚠️  Images provided but Claude CLI headless mode doesn't support image input yet"
            );
            log::warn!("   Image paths: {:?}", input.images);
            log::warn!("   Continuing without images...");
        }

        // Add HTML content if provided
        if let Some(html) = &input.html {
            prompt.push_str("Here is the HTML content of the web page:\n\n");
            prompt.push_str("```html\n");
            // Truncate HTML if it's too large (Claude has token limits)
            if html.len() > 100_000 {
                log::warn!(
                    "⚠️  HTML content is large ({} bytes), truncating to 100KB",
                    html.len()
                );
                prompt.push_str(&html[..100_000]);
                prompt.push_str("\n... [truncated]");
            } else {
                prompt.push_str(html);
            }
            prompt.push_str("\n```\n\n");
        }

        // Add the main prompt
        prompt.push_str(&input.prompt);

        prompt
    }

    /// Run the Claude CLI command and return the output
    async fn run_command(&self, input: &ClaudeInput, streaming: bool) -> Result<String> {
        let claude_cmd = self
            .config
            .claude_path
            .as_ref()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| "claude".to_string());

        log::debug!("📟 Building Claude CLI command...");
        log::debug!("Command: {}", claude_cmd);

        let mut cmd = Command::new(&claude_cmd);

        // Add print mode for non-interactive use
        cmd.arg("--print");

        // Set output format
        if streaming {
            cmd.arg("--output-format").arg("stream-json");
        } else {
            cmd.arg("--output-format").arg("json");
        }

        // Add model if specified
        if let Some(model) = &self.config.model {
            log::debug!("Model: {}", model);
            cmd.arg("--model").arg(model);
        }

        // Add permission settings
        if self.config.skip_permissions {
            log::debug!("Skip permissions: enabled");
            cmd.arg("--dangerously-skip-permissions");
        }

        // Add allowed directories
        if !self.config.allowed_dirs.is_empty() {
            log::debug!("Allowed dirs: {:?}", self.config.allowed_dirs);
            cmd.arg("--add-dir");
            for dir in &self.config.allowed_dirs {
                cmd.arg(dir);
            }
        }

        // Add allowed tools
        if !self.config.allowed_tools.is_empty() {
            log::debug!("Allowed tools: {:?}", self.config.allowed_tools);
            cmd.arg("--allowed-tools");
            for tool in &self.config.allowed_tools {
                cmd.arg(tool);
            }
        }

        // Add disallowed tools
        if !self.config.disallowed_tools.is_empty() {
            log::debug!("Disallowed tools: {:?}", self.config.disallowed_tools);
            cmd.arg("--disallowed-tools");
            for tool in &self.config.disallowed_tools {
                cmd.arg(tool);
            }
        }

        // Build the full prompt
        log::debug!("Building prompt...");
        let prompt = self.build_prompt(input);
        log::debug!("Prompt length: {} chars", prompt.len());
        log::debug!(
            "Prompt preview (first 200 chars): {}",
            &prompt.chars().take(200).collect::<String>()
        );

        // Add prompt as the final argument
        cmd.arg(&prompt);

        // Execute the command
        log::info!("🚀 Spawning Claude CLI process...");
        let output = cmd.output().await.context("Failed to execute Claude CLI")?;

        log::debug!("Claude CLI process exited with status: {}", output.status);
        log::debug!("Stdout length: {} bytes", output.stdout.len());
        log::debug!("Stderr length: {} bytes", output.stderr.len());

        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let exit_code = output.status.code().unwrap_or(-1);

        // Try to parse JSON response even if exit code is non-zero
        // Claude CLI may return JSON with error information
        if let Ok(json_response) = serde_json::from_str::<ClaudeResponse>(&stdout) {
            log::debug!("Parsed JSON response from Claude CLI");

            // Check if the response indicates an error
            if json_response.is_error || !output.status.success() {
                let error_msg = &json_response.result;
                log::error!("❌ Claude CLI returned error!");
                log::error!("Exit code: {}", exit_code);
                log::error!("Error message: {}", error_msg);
                log::error!("Stderr: {}", stderr);

                // Classify based on the error message in the result
                let error = classify_claude_error_from_result(error_msg, &stderr, exit_code);
                log::error!("Error classification: {:?}", error);

                return Err(anyhow::Error::from(error));
            }

            // Success - return the JSON output as string
            log::debug!("✅ Claude CLI completed successfully");
            return Ok(stdout.to_string());
        }

        // If we can't parse JSON and exit code is non-zero, it's an error
        if !output.status.success() {
            log::error!("❌ Claude CLI failed!");
            log::error!("Exit code: {}", exit_code);
            log::error!("Stderr: {}", stderr);
            log::error!("Stdout: {}", stdout);

            // Classify the error using stderr
            let error = classify_claude_error(&stderr, exit_code);
            log::error!("Error classification: {:?}", error);

            return Err(anyhow::Error::from(error));
        }

        // Success case with non-JSON output (shouldn't happen with --output-format json)
        log::debug!("✅ Claude CLI completed successfully");
        Ok(stdout.to_string())
    }

    /// Spawn a streaming command
    #[allow(dead_code)]
    async fn spawn_streaming_command(&self, input: &ClaudeInput) -> Result<tokio::process::Child> {
        let claude_cmd = self
            .config
            .claude_path
            .as_ref()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| "claude".to_string());

        let mut cmd = Command::new(&claude_cmd);

        // Configure for streaming
        cmd.arg("--print");
        cmd.arg("--output-format").arg("stream-json");

        // Add model if specified
        if let Some(model) = &self.config.model {
            cmd.arg("--model").arg(model);
        }

        // Add permission settings
        if self.config.skip_permissions {
            cmd.arg("--dangerously-skip-permissions");
        }

        // Add allowed directories
        if !self.config.allowed_dirs.is_empty() {
            cmd.arg("--add-dir");
            for dir in &self.config.allowed_dirs {
                cmd.arg(dir);
            }
        }

        // Add allowed tools
        if !self.config.allowed_tools.is_empty() {
            cmd.arg("--allowed-tools");
            for tool in &self.config.allowed_tools {
                cmd.arg(tool);
            }
        }

        // Add disallowed tools
        if !self.config.disallowed_tools.is_empty() {
            cmd.arg("--disallowed-tools");
            for tool in &self.config.disallowed_tools {
                cmd.arg(tool);
            }
        }

        // Build the full prompt
        let prompt = self.build_prompt(input);
        cmd.arg(&prompt);

        // Spawn with stdout capture
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let child = cmd.spawn().context("Failed to spawn Claude CLI")?;

        Ok(child)
    }
}

impl Default for ClaudeClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_build_prompt() {
        let client = ClaudeClient::new();
        let input = ClaudeInput {
            prompt: "What do you see in the screenshot?".to_string(),
            images: vec![PathBuf::from("/tmp/screenshot.png")],
            html: Some("<html><body>Hello</body></html>".to_string()),
        };

        let prompt = client.build_prompt(&input);
        assert!(prompt.contains("screenshot"));
        assert!(prompt.contains("HTML"));
        assert!(prompt.contains("What do you see"));
    }
}
