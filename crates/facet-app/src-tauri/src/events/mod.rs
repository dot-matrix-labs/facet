use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};

/// Debug event types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum DebugEvent {
    /// Chrome is being downloaded
    ChromeDownloading { message: String },
    /// Chrome download progress
    ChromeDownloadProgress {
        downloaded: u64,
        total: u64,
        percent: u8,
    },
    /// Chrome download completed
    ChromeDownloaded { path: String, version: String },
    /// Chrome is launching
    ChromeLaunching { message: String },
    /// Chrome successfully launched
    ChromeLaunched { message: String },
    /// Page navigation started
    PageNavigating { url: String },
    /// Page is loading
    PageLoading { message: String },
    /// Page loaded successfully
    PageLoaded { url: String, title: String },
    /// Content is being parsed
    PageParsing { message: String },
    /// General info message
    Info { message: String },
    /// Success message
    Success { message: String },
    /// Error occurred
    Error {
        message: String,
        details: Option<String>,
    },
    /// Claude CLI check started
    ClaudeChecking { message: String },
    /// Claude CLI found and validated
    ClaudeReady {
        version: String,
        path: String,
        authenticated: bool,
    },
    /// Claude CLI not found or misconfigured
    ClaudeNotReady { issue: String, suggestion: String },
    /// Claude is processing request
    ClaudeProcessing { message: String },
    /// Claude screenshot capture
    ClaudeScreenshot { path: String },
    /// Claude HTML extraction
    ClaudeHtmlExtracted { size_kb: usize },
    /// Claude API call started
    ClaudeApiCall {
        model: String,
        prompt_preview: String,
    },
    /// Claude response received
    ClaudeResponse { preview: String, full_length: usize },
}

impl DebugEvent {
    /// Get the event name for Tauri emission
    pub fn event_name(&self) -> &'static str {
        "debug-event"
    }

    /// Emit this event to the frontend
    pub fn emit(&self, app: &AppHandle) -> Result<(), String> {
        app.emit(self.event_name(), self)
            .map_err(|e| format!("Failed to emit event: {}", e))
    }
}

/// Helper functions to emit specific events
pub fn emit_info(app: &AppHandle, message: impl Into<String>) -> Result<(), String> {
    DebugEvent::Info {
        message: message.into(),
    }
    .emit(app)
}

pub fn emit_success(app: &AppHandle, message: impl Into<String>) -> Result<(), String> {
    DebugEvent::Success {
        message: message.into(),
    }
    .emit(app)
}

pub fn emit_error(
    app: &AppHandle,
    message: impl Into<String>,
    details: Option<String>,
) -> Result<(), String> {
    DebugEvent::Error {
        message: message.into(),
        details,
    }
    .emit(app)
}

#[allow(dead_code)]
pub fn emit_chrome_downloading(app: &AppHandle, message: impl Into<String>) -> Result<(), String> {
    DebugEvent::ChromeDownloading {
        message: message.into(),
    }
    .emit(app)
}

#[allow(dead_code)]
pub fn emit_chrome_launching(app: &AppHandle, message: impl Into<String>) -> Result<(), String> {
    DebugEvent::ChromeLaunching {
        message: message.into(),
    }
    .emit(app)
}

#[allow(dead_code)]
pub fn emit_chrome_launched(app: &AppHandle, message: impl Into<String>) -> Result<(), String> {
    DebugEvent::ChromeLaunched {
        message: message.into(),
    }
    .emit(app)
}

#[allow(dead_code)]
pub fn emit_page_navigating(app: &AppHandle, url: impl Into<String>) -> Result<(), String> {
    DebugEvent::PageNavigating { url: url.into() }.emit(app)
}

#[allow(dead_code)]
pub fn emit_page_loaded(
    app: &AppHandle,
    url: impl Into<String>,
    title: impl Into<String>,
) -> Result<(), String> {
    DebugEvent::PageLoaded {
        url: url.into(),
        title: title.into(),
    }
    .emit(app)
}

pub fn emit_claude_checking(app: &AppHandle, message: impl Into<String>) -> Result<(), String> {
    DebugEvent::ClaudeChecking {
        message: message.into(),
    }
    .emit(app)
}

pub fn emit_claude_ready(
    app: &AppHandle,
    version: impl Into<String>,
    path: impl Into<String>,
    authenticated: bool,
) -> Result<(), String> {
    DebugEvent::ClaudeReady {
        version: version.into(),
        path: path.into(),
        authenticated,
    }
    .emit(app)
}

pub fn emit_claude_not_ready(
    app: &AppHandle,
    issue: impl Into<String>,
    suggestion: impl Into<String>,
) -> Result<(), String> {
    DebugEvent::ClaudeNotReady {
        issue: issue.into(),
        suggestion: suggestion.into(),
    }
    .emit(app)
}

pub fn emit_claude_processing(app: &AppHandle, message: impl Into<String>) -> Result<(), String> {
    DebugEvent::ClaudeProcessing {
        message: message.into(),
    }
    .emit(app)
}

#[allow(dead_code)]
pub fn emit_claude_screenshot(app: &AppHandle, path: impl Into<String>) -> Result<(), String> {
    DebugEvent::ClaudeScreenshot { path: path.into() }.emit(app)
}

#[allow(dead_code)]
pub fn emit_claude_html_extracted(app: &AppHandle, size_kb: usize) -> Result<(), String> {
    DebugEvent::ClaudeHtmlExtracted { size_kb }.emit(app)
}

#[allow(dead_code)]
pub fn emit_claude_api_call(
    app: &AppHandle,
    model: impl Into<String>,
    prompt_preview: impl Into<String>,
) -> Result<(), String> {
    DebugEvent::ClaudeApiCall {
        model: model.into(),
        prompt_preview: prompt_preview.into(),
    }
    .emit(app)
}

#[allow(dead_code)]
pub fn emit_claude_response(
    app: &AppHandle,
    preview: impl Into<String>,
    full_length: usize,
) -> Result<(), String> {
    DebugEvent::ClaudeResponse {
        preview: preview.into(),
        full_length,
    }
    .emit(app)
}
