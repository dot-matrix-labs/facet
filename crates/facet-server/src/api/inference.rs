//! Inference endpoint for simple Claude prompt execution
//!
//! Handles POST /inference with simple JSON payload: {"prompt": "..."}
//! Returns JSON response.
//! This acts as a simplified adapter to the main execution engine.

use crate::claude::Executor;
use crate::config::Config;
use crate::models::{
    ClaudeEvent, DomState, RequestContext, RequestOptions, FacetRequest, Screenshot,
    ScreenshotMetadata, Viewport,
};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;
use warp::Reply;

#[derive(Debug, Deserialize)]
pub struct InferenceRequest {
    prompt: String,
}

#[derive(Debug, Serialize)]
pub struct InferenceResponse {
    status: String,
    message: String,
    execution_report: Option<serde_json::Value>,
}

/// Handler for the /inference endpoint
pub async fn inference_handler(
    request: InferenceRequest,
    executor: Arc<dyn Executor>,
    config: Arc<Config>,
) -> Result<impl Reply, warp::Rejection> {
    let session_id = Uuid::new_v4();

    // Create dummy context to satisfy validation constraints
    // The server expects visual context, so we provide a minimal placeholder
    let dummy_screenshot = Screenshot {
        // Use a static valid timestamp
        timestamp: "2024-01-01T00:00:00Z".to_string(),
        // 1x1 black pixel PNG
        image_data: "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAAAAAA6fptVAAAACklEQVR4nGNiAAAABgADNjd8qAAAAABJRU5ErkJggg==".to_string(),
        metadata: ScreenshotMetadata {
            window_title: "Inference Adapter".to_string(),
            url: None,
            viewport: Viewport { width: 100, height: 100 },
        },
    };

    let facet_request = FacetRequest {
        session_id,
        prompt: request.prompt,
        context: RequestContext {
            screenshots: vec![dummy_screenshot],
            dom_state: DomState {
                accessible_tree: "Inference Mode".to_string(),
                interactive_elements: vec![],
            },
            user_intent: "Execute simple inference".to_string(),
        },
        options: RequestOptions::default(),
    };

    // Validate (reuse config limits)
    if let Err(e) = facet_request.validate(
        config.limits.max_screenshot_count,
        config.limits.max_prompt_length,
        50000,
    ) {
        return Ok(warp::reply::json(&InferenceResponse {
            status: "error".to_string(),
            message: format!("Invalid request: {}", e),
            execution_report: None,
        }));
    }

    // Execute directly and collect results (non-streaming for this endpoint)
    let mut event_stream = executor.execute(facet_request).await;
    let mut final_message = String::new();
    let mut status = "success".to_string();

    while let Some(result) = event_stream.next().await {
        match result {
            Ok(event) => match event {
                ClaudeEvent::Complete { .. } => {
                    // Done
                }
                ClaudeEvent::Error { message, .. } => {
                    status = "error".to_string();
                    if final_message.is_empty() {
                        final_message = message;
                    } else {
                        final_message = format!("{}\nError: {}", final_message, message);
                    }
                }
                ClaudeEvent::Content { text } => {
                    // Accumulate text content
                    final_message.push_str(&text);
                }
                _ => {}
            },
            Err(e) => {
                status = "error".to_string();
                final_message = e.to_string();
            }
        }
    }

    if final_message.is_empty() {
        if status == "success" {
            final_message = "Command executed successfully.".to_string();
        } else {
            final_message = "Unknown error occurred.".to_string();
        }
    }

    // Return simple JSON response
    Ok(warp::reply::json(&InferenceResponse {
        status,
        message: final_message,
        execution_report: None, // TODO: Extract report from events if available
    }))
}
