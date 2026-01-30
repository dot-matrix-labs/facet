//! Workflow execution logic for different agent tasks

use super::config::AgentConfig;
use super::prompts::{PromptContext, PromptTemplate, PromptType};
use crate::claude::{ClaudeClient, ClaudeConfig, ClaudeInput, ClaudeResponse};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
/// A question that needs clarification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClarificationQuestion {
    pub question: String,
    pub options: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
}

/// Type of workflow to execute
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowType {
    /// Generate and execute CDP script
    CdpAutomation,
    /// Update agent configuration
    ConfigUpdate,
    /// Improve user feedback
    FeedbackImprovement,
}

/// Result of workflow execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowResult {
    pub success: bool,
    pub workflow_type: WorkflowType,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cdp_script: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution_report: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// Clarification questions if the agent needs more information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clarification: Option<Vec<ClarificationQuestion>>,
    /// Agent's understanding of the request so far
    #[serde(skip_serializing_if = "Option::is_none")]
    pub understanding: Option<String>,
    /// Refined feedback text (for feedback improvement workflow)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refined_feedback: Option<String>,
}

/// Workflow executor
pub struct WorkflowExecutor;

impl WorkflowExecutor {
    /// Create a new workflow executor
    pub fn new() -> Self {
        Self
    }

    /// Execute a workflow based on user input
    pub async fn execute(
        &self,
        workflow_type: WorkflowType,
        user_message: String,
        agent_config: &AgentConfig,
        _screenshot_path: Option<PathBuf>,
        _html_content: Option<String>,
        http_client: &reqwest::Client,
    ) -> Result<WorkflowResult> {
        match workflow_type {
            WorkflowType::CdpAutomation => {
                self.execute_cdp_workflow(user_message, http_client).await
            }
            WorkflowType::ConfigUpdate => {
                self.execute_config_update_workflow(user_message, agent_config)
                    .await
            }
            WorkflowType::FeedbackImprovement => {
                self.execute_feedback_improvement_workflow(user_message, agent_config)
                    .await
            }
        }
    }

    /// Execute CDP generation and automation workflow via Standalone Webdriver
    async fn execute_cdp_workflow(
        &self,
        user_message: String,
        http_client: &reqwest::Client,
    ) -> Result<WorkflowResult> {
        log::info!("╔═══════════════════════════════════════════════════════════╗");
        log::info!("║  🤖 CDP AUTOMATION WORKFLOW (DELEGATED TO SERVER)         ║");
        log::info!("╚═══════════════════════════════════════════════════════════╝");

        // Connect to facet-server on port 8443 (default)
        let url = "http://localhost:8443/inference";
        let payload = serde_json::json!({
            "prompt": user_message
        });

        log::info!("Sending inference request to {}", url);

        let response = match http_client.post(url).json(&payload).send().await {
            Ok(res) => res,
            Err(e) => {
                log::error!("Failed to connect to facet-server: {}", e);
                return Ok(WorkflowResult {
                    success: false,
                    workflow_type: WorkflowType::CdpAutomation,
                    message: "Failed to connect to facet-server".to_string(),
                    cdp_script: None,
                    execution_report: None,
                    error: Some(format!("Connection failed: {}", e)),
                    clarification: None,
                    understanding: None,
                    refined_feedback: None,
                });
            }
        };

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Ok(WorkflowResult {
                success: false,
                workflow_type: WorkflowType::CdpAutomation,
                message: format!("Server error: {}", error_text),
                cdp_script: None,
                execution_report: None,
                error: Some(error_text),
                clarification: None,
                understanding: None,
                refined_feedback: None,
            });
        }

        let json: serde_json::Value = match response.json().await {
            Ok(j) => j,
            Err(e) => {
                return Ok(WorkflowResult {
                    success: false,
                    workflow_type: WorkflowType::CdpAutomation,
                    message: "Invalid response from server".to_string(),
                    cdp_script: None,
                    execution_report: None,
                    error: Some(e.to_string()),
                    clarification: None,
                    understanding: None,
                    refined_feedback: None,
                });
            }
        };

        let status = json
            .get("status")
            .and_then(|s| s.as_str())
            .unwrap_or("error");
        let message = json
            .get("message")
            .and_then(|s| s.as_str())
            .unwrap_or("")
            .to_string();

        let success = status == "success";
        let execution_report = json.get("execution_report").cloned();

        Ok(WorkflowResult {
            success,
            workflow_type: WorkflowType::CdpAutomation,
            message,
            cdp_script: None, // Server doesn't return raw script currently, or maybe it does in report
            execution_report,
            error: if success {
                None
            } else {
                Some("Execution failed on server".to_string())
            },
            clarification: None,
            understanding: None,
            refined_feedback: None,
        })
    }

    /// Execute config update workflow
    async fn execute_config_update_workflow(
        &self,
        user_feedback: String,
        agent_config: &AgentConfig,
    ) -> Result<WorkflowResult> {
        // Serialize current config to TOML
        let current_config =
            toml::to_string_pretty(agent_config).context("Failed to serialize config")?;

        // Build prompt using template
        let template = PromptTemplate::new(PromptType::ConfigUpdate);
        let context = PromptContext {
            agent_name: agent_config.name.clone(),
            current_config: current_config.clone(),
            user_feedback: user_feedback.clone(),
            ..Default::default()
        };
        let prompt = template.build(context);

        // Call Claude to generate updated config
        let claude_response = self
            .call_claude(&prompt, None, None, &agent_config.settings.model)
            .await?;

        // Try to parse the response as TOML
        let updated_config_text = claude_response.text().trim();

        // Remove markdown code blocks if present
        let cleaned_toml = if updated_config_text.starts_with("```toml") {
            updated_config_text
                .trim_start_matches("```toml")
                .trim_end_matches("```")
                .trim()
        } else if updated_config_text.starts_with("```") {
            updated_config_text
                .trim_start_matches("```")
                .trim_end_matches("```")
                .trim()
        } else {
            updated_config_text
        };

        match toml::from_str::<AgentConfig>(cleaned_toml) {
            Ok(updated_config) => {
                // Save the updated config
                let config_path = AgentConfig::config_path(&agent_config.name)?;
                updated_config.save(&config_path).await?;

                Ok(WorkflowResult {
                    success: true,
                    workflow_type: WorkflowType::ConfigUpdate,
                    message: format!("Successfully updated {} configuration", agent_config.name),
                    cdp_script: None,
                    execution_report: Some(serde_json::json!({
                        "config_path": config_path.to_string_lossy(),
                        "updated_config": cleaned_toml,
                    })),
                    error: None,
                    clarification: None,
                    understanding: None,
                    refined_feedback: None,
                })
            }
            Err(e) => Ok(WorkflowResult {
                success: false,
                workflow_type: WorkflowType::ConfigUpdate,
                message: "Failed to parse updated configuration".to_string(),
                cdp_script: None,
                execution_report: None,
                error: Some(format!(
                    "Parse error: {}\n\nGenerated config:\n{}",
                    e, cleaned_toml
                )),
                clarification: None,
                understanding: None,
                refined_feedback: None,
            }),
        }
    }

    /// Execute feedback improvement workflow
    async fn execute_feedback_improvement_workflow(
        &self,
        user_feedback: String,
        agent_config: &AgentConfig,
    ) -> Result<WorkflowResult> {
        let template = PromptTemplate::new(PromptType::FeedbackImprovement);
        let context = PromptContext {
            user_feedback: user_feedback.clone(),
            ..Default::default()
        };
        let prompt = template.build(context);

        let claude_response = self
            .call_claude(&prompt, None, None, &agent_config.settings.model)
            .await?;

        let response_text = claude_response.text().trim();

        // Clean markdown
        let json_text = if response_text.starts_with("```json") {
            response_text
                .trim_start_matches("```json")
                .trim_end_matches("```")
                .trim()
        } else if response_text.starts_with("```") {
            response_text
                .trim_start_matches("```")
                .trim_end_matches("```")
                .trim()
        } else {
            response_text
        };

        // Parse JSON
        #[derive(Deserialize)]
        struct FeedbackResponse {
            #[serde(default)]
            message: String,
            refined_feedback: Option<String>,
        }

        match serde_json::from_str::<FeedbackResponse>(json_text) {
            Ok(parsed) => Ok(WorkflowResult {
                success: true,
                workflow_type: WorkflowType::FeedbackImprovement,
                message: parsed.message,
                cdp_script: None,
                execution_report: None,
                error: None,
                clarification: None,
                understanding: None,
                refined_feedback: parsed.refined_feedback,
            }),
            Err(e) => {
                // Fallback if JSON parsing fails - simple pass-through or error
                Ok(WorkflowResult {
                    success: false,
                    workflow_type: WorkflowType::FeedbackImprovement,
                    message: "I encountered an error processing your feedback.".to_string(),
                    cdp_script: None,
                    execution_report: None,
                    error: Some(format!("Failed to parse response: {}\n{}", e, json_text)),
                    clarification: None,
                    understanding: None,
                    refined_feedback: None,
                })
            }
        }
    }

    /// Call Claude API
    async fn call_claude(
        &self,
        prompt: &str,
        screenshot_path: Option<PathBuf>,
        html_content: Option<String>,
        model: &Option<String>,
    ) -> Result<ClaudeResponse> {
        let images = screenshot_path.map(|p| vec![p]).unwrap_or_default();

        let input = ClaudeInput {
            prompt: prompt.to_string(),
            images,
            html: html_content,
        };

        let config = ClaudeConfig {
            model: model.clone(),
            skip_permissions: true,
            ..Default::default()
        };

        let client = ClaudeClient::with_config(config);
        client
            .execute(input)
            .await
            .context("Claude API call failed")
    }
}
