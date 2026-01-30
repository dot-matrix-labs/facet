//! Configuration management for facet-server
//!
//! Handles loading and validation of server configuration from TOML files.
//! Supports environment variable overrides and provides sensible defaults
//! for all optional settings.

use crate::error::FacetError;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Server configuration
///
/// Contains bind address, port, and TLS settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Host to bind to (e.g., "127.0.0.1" or "0.0.0.0")
    #[serde(default = "default_host")]
    pub host: String,

    /// Port to listen on
    #[serde(default = "default_port")]
    pub port: u16,

    /// Enable development mode (relaxed security, verbose logging)
    #[serde(default)]
    pub dev_mode: bool,

    /// Enable TLS (should be false for localhost development)
    #[serde(default = "default_enable_tls")]
    pub enable_tls: bool,

    /// Path to TLS certificate file (required if enable_tls = true)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tls_cert: Option<String>,

    /// Path to TLS private key file (required if enable_tls = true)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tls_key: Option<String>,
}

fn default_host() -> String {
    "127.0.0.1".to_string()
}

fn default_port() -> u16 {
    8443
}

fn default_enable_tls() -> bool {
    false
}

/// Authentication configuration
///
/// Controls authentication requirements and rate limiting.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    /// Development token for local testing
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dev_token: Option<String>,

    /// List of valid bearer tokens (production use)
    #[serde(default)]
    pub tokens: Vec<String>,

    /// Require authentication (can be disabled for local dev)
    #[serde(default = "default_require_auth")]
    pub require_auth: bool,

    /// Rate limit per token/IP (requests per minute)
    #[serde(default = "default_rate_limit")]
    pub rate_limit_per_minute: u32,
}

fn default_require_auth() -> bool {
    false
}

fn default_rate_limit() -> u32 {
    60
}

/// Claude CLI configuration
///
/// Specifies how to execute claude-cli and manage sessions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeConfig {
    /// Path to claude-cli binary
    #[serde(default = "default_binary_path")]
    pub binary_path: String,

    /// Use mock executor instead of real claude-cli
    #[serde(default)]
    pub mock_mode: bool,

    /// Default timeout for claude-cli execution (seconds)
    #[serde(default = "default_timeout_seconds")]
    pub default_timeout_seconds: u64,

    /// Maximum concurrent sessions
    #[serde(default = "default_max_concurrent")]
    pub max_concurrent_sessions: usize,
}

fn default_binary_path() -> String {
    "claude".to_string()
}

fn default_timeout_seconds() -> u64 {
    300
}

fn default_max_concurrent() -> usize {
    20
}

/// Request size and content limits
///
/// Enforces maximum sizes to prevent resource exhaustion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LimitsConfig {
    /// Maximum request size in megabytes
    #[serde(default = "default_max_request_mb")]
    pub max_request_size_mb: usize,

    /// Maximum number of screenshots per request
    #[serde(default = "default_max_screenshots")]
    pub max_screenshot_count: usize,

    /// Maximum prompt length in characters
    #[serde(default = "default_max_prompt_length")]
    pub max_prompt_length: usize,
}

fn default_max_request_mb() -> usize {
    50
}

fn default_max_screenshots() -> usize {
    10
}

fn default_max_prompt_length() -> usize {
    50000
}

/// Logging configuration
///
/// Controls log level, format, and data sanitization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level: trace, debug, info, warn, error
    #[serde(default = "default_log_level")]
    pub level: String,

    /// Use pretty-printed logs (vs. JSON for production)
    #[serde(default)]
    pub pretty_print: bool,

    /// Sanitize sensitive data from logs
    #[serde(default = "default_sanitize")]
    pub sanitize_sensitive_data: bool,
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_sanitize() -> bool {
    true
}

/// Root configuration structure
///
/// Aggregates all configuration sections and provides validation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub auth: AuthConfig,
    pub claude: ClaudeConfig,
    pub limits: LimitsConfig,
    pub logging: LoggingConfig,
}

impl Config {
    /// Loads configuration from a TOML file
    ///
    /// Reads the file at the specified path and deserializes it into
    /// a Config struct. Missing optional fields use default values.
    ///
    /// # Arguments
    /// * `path` - Path to TOML configuration file
    ///
    /// # Returns
    /// Loaded and validated configuration
    ///
    /// # Errors
    /// Returns FacetError::Config if file cannot be read or parsed
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, FacetError> {
        let contents = std::fs::read_to_string(path.as_ref())
            .map_err(|e| FacetError::Config(format!("Failed to read config file: {}", e)))?;

        let config: Config = toml::from_str(&contents)
            .map_err(|e| FacetError::Config(format!("Failed to parse config: {}", e)))?;

        config.validate()?;
        Ok(config)
    }

    /// Creates default development configuration
    ///
    /// Returns a Config with settings appropriate for local development:
    /// - Localhost binding
    /// - HTTP (no TLS)
    /// - Authentication disabled
    /// - Debug logging
    /// - Mock mode available
    ///
    /// # Returns
    /// Development-ready configuration
    pub fn dev_default() -> Self {
        Config {
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 8443,
                dev_mode: true,
                enable_tls: false,
                tls_cert: None,
                tls_key: None,
            },
            auth: AuthConfig {
                dev_token: Some("dev-token-12345".to_string()),
                tokens: vec![],
                require_auth: false,
                rate_limit_per_minute: 100,
            },
            claude: ClaudeConfig {
                binary_path: "claude".to_string(),
                mock_mode: false,
                default_timeout_seconds: 300,
                max_concurrent_sessions: 20,
            },
            limits: LimitsConfig {
                max_request_size_mb: 50,
                max_screenshot_count: 10,
                max_prompt_length: 50000,
            },
            logging: LoggingConfig {
                level: "debug".to_string(),
                pretty_print: true,
                sanitize_sensitive_data: true,
            },
        }
    }

    /// Validates the configuration
    ///
    /// Checks that all required fields are present and values are valid.
    /// Performs cross-field validation (e.g., TLS cert required if TLS enabled).
    ///
    /// # Returns
    /// Ok(()) if valid, Err with description if invalid
    ///
    /// # Errors
    /// Returns FacetError::Config if validation fails
    pub fn validate(&self) -> Result<(), FacetError> {
        // Validate server config
        if self.server.host.is_empty() {
            return Err(FacetError::Config("Host cannot be empty".to_string()));
        }

        if self.server.port == 0 {
            return Err(FacetError::Config(
                "Port must be greater than 0".to_string(),
            ));
        }

        // If TLS is enabled, cert and key must be provided
        if self.server.enable_tls {
            if self.server.tls_cert.is_none() || self.server.tls_key.is_none() {
                return Err(FacetError::Config(
                    "TLS cert and key required when enable_tls = true".to_string(),
                ));
            }

            // Check that TLS files exist
            if let Some(ref cert_path) = self.server.tls_cert {
                if !Path::new(cert_path).exists() {
                    return Err(FacetError::Config(format!(
                        "TLS cert file not found: {}",
                        cert_path
                    )));
                }
            }

            if let Some(ref key_path) = self.server.tls_key {
                if !Path::new(key_path).exists() {
                    return Err(FacetError::Config(format!(
                        "TLS key file not found: {}",
                        key_path
                    )));
                }
            }
        }

        // Validate auth config
        if self.auth.require_auth && self.auth.tokens.is_empty() && self.auth.dev_token.is_none() {
            return Err(FacetError::Config(
                "Authentication required but no tokens configured".to_string(),
            ));
        }

        if self.auth.rate_limit_per_minute == 0 {
            return Err(FacetError::Config(
                "Rate limit must be greater than 0".to_string(),
            ));
        }

        // Validate claude config
        if self.claude.binary_path.is_empty() {
            return Err(FacetError::Config(
                "Claude binary path cannot be empty".to_string(),
            ));
        }

        if self.claude.default_timeout_seconds == 0 {
            return Err(FacetError::Config(
                "Claude timeout must be greater than 0".to_string(),
            ));
        }

        if self.claude.max_concurrent_sessions == 0 {
            return Err(FacetError::Config(
                "Max concurrent sessions must be greater than 0".to_string(),
            ));
        }

        // Validate limits config
        if self.limits.max_request_size_mb == 0 {
            return Err(FacetError::Config(
                "Max request size must be greater than 0".to_string(),
            ));
        }

        if self.limits.max_screenshot_count == 0 {
            return Err(FacetError::Config(
                "Max screenshot count must be greater than 0".to_string(),
            ));
        }

        if self.limits.max_prompt_length == 0 {
            return Err(FacetError::Config(
                "Max prompt length must be greater than 0".to_string(),
            ));
        }

        // Validate logging config
        let valid_levels = ["trace", "debug", "info", "warn", "error"];
        if !valid_levels.contains(&self.logging.level.as_str()) {
            return Err(FacetError::Config(format!(
                "Invalid log level: {}. Must be one of: {}",
                self.logging.level,
                valid_levels.join(", ")
            )));
        }

        Ok(())
    }

    /// Returns the server bind address
    ///
    /// Formats host and port as "host:port" for binding.
    ///
    /// # Returns
    /// Bind address string
    pub fn bind_address(&self) -> String {
        format!("{}:{}", self.server.host, self.server.port)
    }

    /// Returns list of valid authentication tokens
    ///
    /// Combines dev_token and production tokens into a single list.
    ///
    /// # Returns
    /// Vector of valid token strings
    pub fn valid_tokens(&self) -> Vec<String> {
        let mut tokens = self.auth.tokens.clone();
        if let Some(ref dev_token) = self.auth.dev_token {
            tokens.push(dev_token.clone());
        }
        tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_dev_default_config() {
        let config = Config::dev_default();
        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(config.server.port, 8443);
        assert!(config.server.dev_mode);
        assert!(!config.server.enable_tls);
        assert!(!config.auth.require_auth);
        assert_eq!(config.logging.level, "debug");
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_validation_success() {
        let config = Config::dev_default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_validation_empty_host() {
        let mut config = Config::dev_default();
        config.server.host = String::new();
        let result = config.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Host cannot be empty"));
    }

    #[test]
    fn test_config_validation_zero_port() {
        let mut config = Config::dev_default();
        config.server.port = 0;
        let result = config.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Port must be greater"));
    }

    #[test]
    fn test_config_validation_tls_without_cert() {
        let mut config = Config::dev_default();
        config.server.enable_tls = true;
        config.server.tls_cert = None;
        let result = config.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("TLS cert and key required"));
    }

    #[test]
    fn test_config_validation_auth_no_tokens() {
        let mut config = Config::dev_default();
        config.auth.require_auth = true;
        config.auth.dev_token = None;
        config.auth.tokens.clear();
        let result = config.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("no tokens configured"));
    }

    #[test]
    fn test_config_validation_zero_rate_limit() {
        let mut config = Config::dev_default();
        config.auth.rate_limit_per_minute = 0;
        let result = config.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Rate limit must be greater"));
    }

    #[test]
    fn test_config_validation_empty_binary_path() {
        let mut config = Config::dev_default();
        config.claude.binary_path = String::new();
        let result = config.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("binary path cannot be empty"));
    }

    #[test]
    fn test_config_validation_invalid_log_level() {
        let mut config = Config::dev_default();
        config.logging.level = "invalid".to_string();
        let result = config.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid log level"));
    }

    #[test]
    fn test_bind_address() {
        let config = Config::dev_default();
        assert_eq!(config.bind_address(), "127.0.0.1:8443");
    }

    #[test]
    fn test_valid_tokens() {
        let mut config = Config::dev_default();
        config.auth.tokens = vec!["token1".to_string(), "token2".to_string()];
        config.auth.dev_token = Some("dev-token".to_string());

        let tokens = config.valid_tokens();
        assert_eq!(tokens.len(), 3);
        assert!(tokens.contains(&"token1".to_string()));
        assert!(tokens.contains(&"token2".to_string()));
        assert!(tokens.contains(&"dev-token".to_string()));
    }

    #[test]
    fn test_from_file_valid() {
        let toml_content = r#"
[server]
host = "127.0.0.1"
port = 9000
dev_mode = true
enable_tls = false

[auth]
dev_token = "test-token"
require_auth = false
rate_limit_per_minute = 50

[claude]
binary_path = "claude"
mock_mode = true
default_timeout_seconds = 600
max_concurrent_sessions = 10

[limits]
max_request_size_mb = 100
max_screenshot_count = 20
max_prompt_length = 100000

[logging]
level = "trace"
pretty_print = true
sanitize_sensitive_data = false
"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(toml_content.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let config = Config::from_file(temp_file.path()).unwrap();
        assert_eq!(config.server.port, 9000);
        assert_eq!(config.auth.rate_limit_per_minute, 50);
        assert!(config.claude.mock_mode);
        assert_eq!(config.claude.max_concurrent_sessions, 10);
        assert_eq!(config.limits.max_request_size_mb, 100);
        assert_eq!(config.logging.level, "trace");
    }

    #[test]
    fn test_from_file_invalid_path() {
        let result = Config::from_file("/nonexistent/path.toml");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Failed to read config file"));
    }

    #[test]
    fn test_from_file_invalid_toml() {
        let invalid_toml = "this is not valid toml { } [ ]";

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(invalid_toml.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let result = Config::from_file(temp_file.path());
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Failed to parse config"));
    }

    #[test]
    fn test_config_with_defaults() {
        // Test that missing fields use defaults
        let minimal_toml = r#"
[server]
[auth]
[claude]
[limits]
[logging]
"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(minimal_toml.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let config = Config::from_file(temp_file.path()).unwrap();
        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(config.server.port, 8443);
        assert_eq!(config.auth.rate_limit_per_minute, 60);
        assert_eq!(config.claude.binary_path, "claude");
        assert_eq!(config.limits.max_screenshot_count, 10);
        assert_eq!(config.logging.level, "info");
    }
}
