//! Agent configuration storage and management

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Agent configuration stored in a TOML file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Name of the agent
    pub name: String,

    /// Description of what the agent does
    pub description: String,

    /// Version of the configuration
    pub version: String,

    /// Agent-specific settings
    pub settings: AgentSettings,

    /// System instructions for the agent
    pub instructions: String,

    /// Examples to include in prompts
    #[serde(default)]
    pub examples: Vec<String>,

    /// Tags for organization
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Agent settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSettings {
    /// Claude model to use
    pub model: Option<String>,

    /// Whether to include screenshots in requests
    #[serde(default = "default_true")]
    pub include_screenshots: bool,

    /// Whether to include page HTML
    #[serde(default = "default_true")]
    pub include_html: bool,

    /// Maximum retries on failure
    #[serde(default = "default_retries")]
    pub max_retries: u32,

    /// Temperature for Claude API (0.0 - 1.0)
    #[serde(default = "default_temperature")]
    pub temperature: f32,
}

fn default_true() -> bool {
    true
}

fn default_retries() -> u32 {
    3
}

fn default_temperature() -> f32 {
    0.7
}

impl Default for AgentSettings {
    fn default() -> Self {
        Self {
            model: None,
            include_screenshots: true,
            include_html: true,
            max_retries: 3,
            temperature: 0.7,
        }
    }
}

impl AgentConfig {
    /// Load agent config from a TOML file
    pub async fn load(path: &Path) -> Result<Self> {
        let content = tokio::fs::read_to_string(path)
            .await
            .context("Failed to read agent config file")?;

        let config: AgentConfig =
            toml::from_str(&content).context("Failed to parse agent config TOML")?;

        Ok(config)
    }

    /// Save agent config to a TOML file
    pub async fn save(&self, path: &Path) -> Result<()> {
        let content = toml::to_string_pretty(self).context("Failed to serialize agent config")?;

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .context("Failed to create config directory")?;
        }

        tokio::fs::write(path, content)
            .await
            .context("Failed to write agent config file")?;

        Ok(())
    }

    /// Get the default config directory
    pub fn default_config_dir() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .context("Failed to get config directory")?
            .join("facet")
            .join("agents");

        Ok(config_dir)
    }

    /// Get path to a specific agent config
    pub fn config_path(agent_name: &str) -> Result<PathBuf> {
        Ok(Self::default_config_dir()?.join(format!("{}.toml", agent_name)))
    }

    /// Create default CDP generation agent config
    pub fn default_cdp_agent() -> Self {
        Self {
            name: "cdp-generator".to_string(),
            description: "Generates CDP automation scripts from natural language requests".to_string(),
            version: "1.0.0".to_string(),
            settings: AgentSettings::default(),
            instructions: r#"You are a browser automation expert that generates Chrome DevTools Protocol (CDP) scripts.

Your task is to:
1. Understand the user's natural language request
2. Analyze the current page (screenshot and HTML provided)
3. Generate a valid CDP JSON script that accomplishes the task
4. Use only the approved CDP commands listed in the template
5. Ensure the script is safe and follows best practices

Key principles:
- Always navigate to a page before interacting with it
- Use Runtime.evaluate for complex interactions (clicks, form fills, data extraction)
- Be defensive - check if elements exist before interacting
- Provide clear descriptions for each command
- Handle errors gracefully"#.to_string(),
            examples: vec![],
            tags: vec!["automation".to_string(), "cdp".to_string()],
        }
    }

    /// Create default meta/config agent config
    pub fn default_meta_agent() -> Self {
        Self {
            name: "meta-agent".to_string(),
            description: "Updates agent configurations and settings based on feedback".to_string(),
            version: "1.0.0".to_string(),
            settings: AgentSettings {
                model: Some("sonnet".to_string()),
                include_screenshots: false,
                include_html: false,
                max_retries: 2,
                temperature: 0.3, // Lower temperature for config changes
            },
            instructions: r#"You are a meta-agent responsible for updating agent configurations and instructions.

Your task is to:
1. Analyze user feedback about agent performance
2. Identify what went wrong or what could be improved
3. Update agent instructions to prevent similar issues
4. Make minimal, targeted changes to fix specific problems
5. Maintain the overall structure and purpose of the agent

Key principles:
- Be conservative - only change what's necessary
- Document why you're making changes
- Test your logic mentally before updating
- Don't remove important safety checks
- Keep instructions clear and concise"#.to_string(),
            examples: vec![],
            tags: vec!["meta".to_string(), "config".to_string()],
        }
    }

    /// Create default feedback assistant agent config
    pub fn default_feedback_agent() -> Self {
        Self {
            name: "feedback-assistant".to_string(),
            description: "Helps users improve their feedback and bug reports".to_string(),
            version: "1.0.0".to_string(),
            settings: AgentSettings {
                model: Some("claude-3-5-sonnet-20241022".to_string()),
                include_screenshots: false,
                include_html: false,
                max_retries: 2,
                temperature: 0.7,
            },
            instructions: r#"You are a helpful QA assistant.
Your goal is to improve bug reports and feedback from users.
You are professional, concise, and helpful."#
                .to_string(),
            examples: vec![],
            tags: vec!["feedback".to_string(), "qa".to_string()],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_and_load_config() {
        let temp_dir = std::env::temp_dir().join("facet-test-config");
        tokio::fs::create_dir_all(&temp_dir).await.unwrap();

        let config_path = temp_dir.join("test-agent.toml");

        let config = AgentConfig::default_cdp_agent();
        config.save(&config_path).await.unwrap();

        let loaded = AgentConfig::load(&config_path).await.unwrap();
        assert_eq!(loaded.name, config.name);
        assert_eq!(loaded.description, config.description);

        tokio::fs::remove_file(&config_path).await.ok();
    }
}
