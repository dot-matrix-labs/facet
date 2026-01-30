//! Agent configuration and prompt templates
//!
//! This module handles agent configuration, prompt templates, and workflows

pub mod config;
pub mod prompts;
pub mod workflow;

pub use config::AgentConfig;
pub use workflow::{WorkflowExecutor, WorkflowResult, WorkflowType};
