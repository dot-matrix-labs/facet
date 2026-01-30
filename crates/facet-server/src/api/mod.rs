//! API endpoints for facet-server
//!
//! This module contains all HTTP endpoint handlers and route definitions.

pub mod execute;
pub mod health;
pub mod inference;
pub mod sessions;

pub use execute::execute_handler;
pub use health::health_handler;
pub use inference::inference_handler;
pub use sessions::{delete_session_handler, get_session_handler};
