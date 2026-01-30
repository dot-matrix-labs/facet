use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeInput {
    pub prompt: String,
    pub images: Vec<PathBuf>,
    pub html: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeResponse {
    #[serde(rename = "type")]
    pub response_type: String,
    pub result: String,
    #[serde(default)]
    pub is_error: bool,
}

pub struct ClaudeClient {
    binary_path: String,
}

impl Default for ClaudeClient {
    fn default() -> Self {
        Self::new()
    }
}

impl ClaudeClient {
    pub fn new() -> Self {
        Self {
            binary_path: "claude".to_string(),
        }
    }

    pub fn with_path(path: String) -> Self {
        Self { binary_path: path }
    }

    pub async fn complete(&self, prompt: &str, system_prompt: Option<&str>) -> Result<String> {
        // Claude CLI doesn't support system prompt directly in the same way as API,
        // but we can prepend it to the prompt or use --system if available (it's not standard in CLI yet usually).
        // We'll prepend it.

        let full_prompt = if let Some(sys) = system_prompt {
            format!("{}\n\n{}", sys, prompt)
        } else {
            prompt.to_string()
        };

        let input = ClaudeInput {
            prompt: full_prompt,
            images: vec![],
            html: None,
        };

        let response = self.execute(input).await?;
        Ok(response.result)
    }

    pub async fn is_available() -> bool {
        let health = ClaudeHealthCheck::check().await;
        health.status == HealthStatus::Healthy
    }

    pub async fn execute(&self, input: ClaudeInput) -> Result<ClaudeResponse> {
        let mut cmd = Command::new(&self.binary_path);

        cmd.arg("--print").arg("--output-format").arg("json");

        // Construct prompt
        let mut prompt_text = input.prompt.clone();
        if let Some(html) = input.html {
            prompt_text.push_str("\n\nHTML Context:\n```html\n");
            prompt_text.push_str(&html);
            prompt_text.push_str("\n```");
        }

        cmd.arg(&prompt_text);

        let output = cmd.output().await.context("Failed to execute Claude CLI")?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        if !output.status.success() {
            anyhow::bail!(
                "Claude CLI failed with status {}: {}",
                output.status,
                stderr
            );
        }

        // Parse JSON
        let response: ClaudeResponse = serde_json::from_str(&stdout)
            .context(format!("Failed to parse Claude response: {}", stdout))?;

        if response.is_error {
            anyhow::bail!("Claude returned error: {}", response.result);
        }

        Ok(response)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeHealthCheck {
    pub installed: bool,
    pub path: Option<String>,
    pub version: Option<String>,
    pub authenticated: bool,
    pub issues: Vec<String>,
    pub suggestions: Vec<String>,
    pub status: HealthStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    Healthy,
    Warning,
    Error,
}

impl ClaudeHealthCheck {
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

        match Self::find_claude_path().await {
            Ok(path) => {
                check.installed = true;
                check.path = Some(path.clone());

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

        check.status = if check.installed && check.authenticated {
            HealthStatus::Healthy
        } else if check.installed {
            HealthStatus::Warning
        } else {
            HealthStatus::Error
        };

        check
    }

    async fn find_claude_path() -> Result<String> {
        let cmd = if cfg!(windows) { "where" } else { "which" };
        let output = Command::new(cmd)
            .arg("claude")
            .output()
            .await
            .context("Failed to run which/where command")?;

        if output.status.success() {
            return Ok(String::from_utf8_lossy(&output.stdout).trim().to_string());
        }

        if let Ok(home) = std::env::var("HOME") {
            let common_paths = vec![
                format!("{}/.claude/local/claude", home),
                format!("{}/.npm-global/bin/claude", home),
                "/usr/local/bin/claude".to_string(),
                "/opt/homebrew/bin/claude".to_string(),
            ];

            for path in common_paths {
                if tokio::fs::metadata(&path).await.is_ok() {
                    return Ok(path);
                }
            }
        }

        anyhow::bail!("Claude CLI not found")
    }

    async fn get_version(path: &str) -> Result<String> {
        let output = Command::new(path)
            .arg("--version")
            .output()
            .await
            .context("Failed to get version")?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            anyhow::bail!("Failed to get version")
        }
    }

    async fn check_authentication(path: &str) -> Result<bool> {
        let output = Command::new(path)
            .arg("--print")
            .arg("test")
            .output()
            .await
            .context("Failed to check authentication")?;

        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("not authenticated")
            || stderr.contains("login")
            || stderr.contains("setup-token")
        {
            Ok(false)
        } else {
            Ok(true)
        }
    }
}
