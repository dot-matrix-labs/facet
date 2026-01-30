use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tokio::process::Command;

/// Health check result for Claude CLI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeHealthCheck {
    /// Whether Claude CLI is installed and accessible
    pub installed: bool,
    /// Path to Claude CLI executable
    pub path: Option<String>,
    /// Claude CLI version
    pub version: Option<String>,
    /// Whether user is authenticated
    pub authenticated: bool,
    /// List of issues found
    pub issues: Vec<String>,
    /// List of suggestions to fix issues
    pub suggestions: Vec<String>,
    /// Overall status
    pub status: HealthStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    Healthy,
    Warning,
    Error,
}

#[allow(dead_code)] // Health check functionality preserved for future use
impl ClaudeHealthCheck {
    /// Perform a comprehensive health check
    pub async fn check() -> Self {
        let mut check = Self {
            installed: false,
            path: None,
            version: None,
            authenticated: false,
            issues: Vec::new(),
            suggestions: Vec::new(),
            status: HealthStatus::Error,
        };

        // Check if Claude CLI is in PATH
        match Self::find_claude_path().await {
            Ok(path) => {
                check.installed = true;
                check.path = Some(path.clone());

                // Check version
                match Self::get_version(&path).await {
                    Ok(version) => {
                        check.version = Some(version);
                    }
                    Err(e) => {
                        check
                            .issues
                            .push(format!("Failed to get Claude CLI version: {}", e));
                    }
                }

                // Check authentication
                match Self::check_authentication(&path).await {
                    Ok(authenticated) => {
                        check.authenticated = authenticated;
                        if !authenticated {
                            check
                                .issues
                                .push("Claude CLI is not authenticated".to_string());
                            check
                                .suggestions
                                .push("Run 'claude setup-token' to authenticate".to_string());
                        }
                    }
                    Err(e) => {
                        check
                            .issues
                            .push(format!("Failed to check authentication: {}", e));
                    }
                }
            }
            Err(_) => {
                check
                    .issues
                    .push("Claude CLI is not installed or not in PATH".to_string());
                check.suggestions.push(
                    "Install Claude CLI: npm install -g @anthropic-ai/claude-code".to_string(),
                );
                check
                    .suggestions
                    .push("Or via Homebrew: brew install claude".to_string());
            }
        }

        // Determine overall status
        check.status = if check.installed && check.authenticated {
            HealthStatus::Healthy
        } else if check.installed {
            HealthStatus::Warning
        } else {
            HealthStatus::Error
        };

        check
    }

    /// Find Claude CLI executable path
    async fn find_claude_path() -> Result<String> {
        // Try 'which claude' on Unix or 'where claude' on Windows
        let cmd = if cfg!(windows) { "where" } else { "which" };

        let output = Command::new(cmd)
            .arg("claude")
            .output()
            .await
            .context("Failed to run which/where command")?;

        if output.status.success() {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            return Ok(path);
        }

        // If not found in PATH, try common installation locations
        if let Ok(home) = std::env::var("HOME") {
            let common_paths = vec![
                // User-specific installation (common for claude CLI)
                format!("{home}/.claude/local/claude"),
                // NPM global installation
                format!("{home}/.npm-global/bin/claude"),
                "/usr/local/bin/claude".to_string(),
                // Homebrew installations
                "/opt/homebrew/bin/claude".to_string(),
                "/usr/local/Homebrew/bin/claude".to_string(),
            ];

            for path in common_paths {
                if tokio::fs::metadata(&path).await.is_ok() {
                    return Ok(path);
                }
            }
        }

        anyhow::bail!("Claude CLI not found in PATH or common installation locations")
    }

    /// Get Claude CLI version
    async fn get_version(claude_path: &str) -> Result<String> {
        let output = Command::new(claude_path)
            .arg("--version")
            .output()
            .await
            .context("Failed to get version")?;

        if output.status.success() {
            let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
            Ok(version)
        } else {
            anyhow::bail!("Failed to get version")
        }
    }

    /// Check if Claude CLI is authenticated
    async fn check_authentication(claude_path: &str) -> Result<bool> {
        // Try a simple command that requires authentication
        let output = Command::new(claude_path)
            .arg("--print")
            .arg("test")
            .output()
            .await
            .context("Failed to check authentication")?;

        // If the command succeeds or returns a specific error about the prompt,
        // we're authenticated. If it fails with auth error, we're not.
        let stderr = String::from_utf8_lossy(&output.stderr);

        // Check for authentication errors
        if stderr.contains("not authenticated")
            || stderr.contains("login")
            || stderr.contains("setup-token")
        {
            Ok(false)
        } else {
            // If we get any other response, we're likely authenticated
            Ok(true)
        }
    }

    /// Get a human-readable status message
    #[allow(dead_code)]
    pub fn status_message(&self) -> String {
        match self.status {
            HealthStatus::Healthy => "Claude CLI is ready".to_string(),
            HealthStatus::Warning => {
                format!("Claude CLI has issues: {}", self.issues.join(", "))
            }
            HealthStatus::Error => {
                format!("Claude CLI is not available: {}", self.issues.join(", "))
            }
        }
    }

    /// Get setup instructions
    #[allow(dead_code)]
    pub fn setup_instructions(&self) -> Vec<String> {
        let mut instructions = Vec::new();

        if !self.installed {
            instructions.push("1. Install Claude CLI:".to_string());
            instructions.push("   npm install -g @anthropic-ai/claude-code".to_string());
            instructions.push("   OR".to_string());
            instructions.push("   brew install claude".to_string());
        }

        if self.installed && !self.authenticated {
            instructions.push("2. Authenticate Claude CLI:".to_string());
            instructions.push("   claude setup-token".to_string());
        }

        if self.installed && self.authenticated {
            instructions.push("Claude CLI is ready to use!".to_string());
        }

        instructions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_health_check() {
        let check = ClaudeHealthCheck::check().await;
        println!("Health check result: {:?}", check);
        println!("Status message: {}", check.status_message());
        println!("Setup instructions:");
        for instruction in check.setup_instructions() {
            println!("  {}", instruction);
        }
    }
}
