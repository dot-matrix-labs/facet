use crate::developer_mode::DevTestServer;
use crate::profiles::auth::UserSession;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Application state that holds the developer mode resources and user session
pub struct AppState {
    pub dev_server: Arc<Mutex<Option<DevTestServer>>>,
    /// Unique session ID for organizing screenshots and other session data
    pub session_id: Arc<Mutex<String>>,
    /// Active user session (username, config, and encryption key)
    /// None if no user is logged in
    pub user_session: Arc<Mutex<Option<UserSession>>>,
    /// HTTP Client for communicating with standalone webdriver
    pub http_client: reqwest::Client,
    /// Webdriver mode enabled (detected at startup)
    pub webdriver_mode: Arc<Mutex<bool>>,
}

impl AppState {
    pub fn new() -> Self {
        // Generate a unique session ID using timestamp
        let session_id = chrono::Utc::now().format("%Y%m%d_%H%M%S").to_string();

        Self {
            dev_server: Arc::new(Mutex::new(None)),
            session_id: Arc::new(Mutex::new(session_id)),
            user_session: Arc::new(Mutex::new(None)),
            http_client: reqwest::Client::new(),
            webdriver_mode: Arc::new(Mutex::new(false)),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
