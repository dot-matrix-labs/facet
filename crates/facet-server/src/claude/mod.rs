//! Claude CLI execution module
//!
//! Provides interfaces for executing claude-cli processes and streaming results.

pub mod executor;
pub mod mock;

pub use executor::ClaudeExecutor;
pub use mock::MockClaudeExecutor;

use crate::error::FacetError;
use crate::models::{ClaudeEvent, FacetRequest};
use futures::Stream;

/// Trait for Claude CLI executors
///
/// Defines the interface for executing Claude CLI requests with streaming responses.
/// Implementations include real executor (spawns claude-cli) and mock executor (for testing).
#[async_trait::async_trait]
pub trait Executor: Send + Sync {
    /// Executes a Facet request and returns a stream of Claude events
    ///
    /// The implementation should:
    /// 1. Validate the request
    /// 2. Spawn/simulate claude-cli process
    /// 3. Stream stdout/stderr events
    /// 4. Handle timeout and cleanup
    /// 5. Emit Complete or Error event at end
    ///
    /// # Arguments
    /// * `request` - The validated Facet request to execute
    ///
    /// # Returns
    /// Async stream of ClaudeEvent instances
    ///
    /// # Errors
    /// Stream may include ClaudeEvent::Error for execution failures
    async fn execute(
        &self,
        request: FacetRequest,
    ) -> Box<dyn Stream<Item = Result<ClaudeEvent, FacetError>> + Send + Unpin + 'static>;
}
