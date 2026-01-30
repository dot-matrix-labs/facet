//! User profiles and multi-user support system
//!
//! This module now re-exports types and functionality from the `facet-types` crate.

pub use facet_types::profiles::auth;
pub use facet_types::profiles::command;
pub use facet_types::profiles::command_md;
pub use facet_types::profiles::crypto;
pub use facet_types::profiles::manager;
pub use facet_types::profiles::markdown;
pub use facet_types::profiles::storage;
pub use facet_types::profiles::types;

pub use facet_types::profiles::types::{
    CommandConfig, SimpleParameter, SimpleParameterType, UserConfig, UserPreferences, UserStats,
};
