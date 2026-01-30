use serde::{Deserialize, Serialize};
use tauri::AppHandle;

#[derive(Debug, Serialize, Deserialize)]
pub struct AppFeedback {
    pub title: String,
    pub description: String,
    pub email: Option<String>,
}

#[tauri::command]
pub async fn submit_application_feedback(
    _app: AppHandle,
    feedback: AppFeedback,
) -> Result<String, String> {
    log::info!("Submitting application feedback: {:?}", feedback.title);

    // Placeholder URL - user needs to replace this with their actual Cloudflare Worker URL
    // TODO: Move this to a configuration file or environment variable
    let worker_url = "https://feedback-api.internal/submit";

    // In a real implementation this would POST to the worker
    if worker_url.contains("internal") {
        // Mock success for development if URL not set
        log::warn!("Feedback worker URL not configured (using mock). Please deploy the Worker and update the URL in src-tauri/src/commands/feedback.rs");
        // Simulate network delay
        tokio::time::sleep(tokio::time::Duration::from_millis(1500)).await;
        return Ok("Feedback submitted successfully (Mock)".to_string());
    }

    let client = reqwest::Client::new();
    let res = client
        .post(worker_url)
        .json(&feedback)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if res.status().is_success() {
        Ok("Feedback submitted successfully".to_string())
    } else {
        let error_text = res.text().await.unwrap_or_default();
        Err(format!("Server returned error: {}", error_text))
    }
}
