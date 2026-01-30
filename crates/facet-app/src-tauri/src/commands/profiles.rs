//! Tauri commands for user profile management
//!
//! This module exposes profile functionality to the frontend including:
//! - User creation and login
//! - User listing
//! - Profile management
//! - Session management
//! - Command management (Phase 3)

use crate::profiles::{
    auth::{AuthError, AuthService},
    command_md::{CommandExecutor, CommandManager},
    manager::UserManager,
    storage::{load_user_profile, save_user_profile},
    types::{Command, CommandInfo, UserConfig},
};
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri::State;

// ============================================================================
// Response Types
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct ProfileResult<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> ProfileResult<T> {
    fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message),
        }
    }
}

// ============================================================================
// Tauri Commands
// ============================================================================

/// Create a new user account
///
/// # Parameters
/// - `username`: Unique username (alphanumeric, underscore, dash)
/// - `password`: User password (minimum 12 characters)
///
/// # Returns
/// UserConfig if successful, error message if failed
#[tauri::command]
pub async fn create_user(
    state: State<'_, AppState>,
    username: String,
    password: String,
) -> Result<ProfileResult<UserConfig>, String> {
    log::info!("Creating user: {}", username);

    match AuthService::create_and_login(&username, &password, None) {
        Ok(session) => {
            // Initialize encrypted logging for this user
            if let Err(e) = crate::logging::init_for_user(&username, &password) {
                log::warn!("⚠️  Failed to initialize logging: {}", e);
            }

            // Store session in app state
            let mut user_session = state.user_session.lock().await;
            *user_session = Some(session.clone());

            log::info!("✅ User '{}' created and logged in", username);
            Ok(ProfileResult::success(session.config))
        }
        Err(e) => {
            log::error!("❌ Failed to create user: {}", e);
            Ok(ProfileResult::error(e.to_string()))
        }
    }
}

/// Login an existing user
///
/// # Parameters
/// - `username`: The username to login
/// - `password`: The user's password
///
/// # Returns
/// UserConfig if successful, error message if failed
#[tauri::command]
pub async fn login_user(
    state: State<'_, AppState>,
    username: String,
    password: String,
) -> Result<ProfileResult<UserConfig>, String> {
    log::info!("Login attempt for user: {}", username);

    match AuthService::login(&username, &password, None) {
        Ok(session) => {
            // Initialize encrypted logging for this user
            if let Err(e) = crate::logging::init_for_user(&username, &password) {
                log::warn!("⚠️  Failed to initialize logging: {}", e);
            }

            // Store session in app state
            let mut user_session = state.user_session.lock().await;
            *user_session = Some(session.clone());

            log::info!("✅ User '{}' logged in successfully", username);
            Ok(ProfileResult::success(session.config))
        }
        Err(AuthError::InvalidPassword) => {
            log::warn!("❌ Invalid password for user: {}", username);
            Ok(ProfileResult::error("Invalid password".to_string()))
        }
        Err(AuthError::UserNotFound(_)) => {
            log::warn!("❌ User not found: {}", username);
            Ok(ProfileResult::error("User not found".to_string()))
        }
        Err(e) => {
            log::error!("❌ Login failed: {}", e);
            Ok(ProfileResult::error(e.to_string()))
        }
    }
}

/// Logout the current user
///
/// Clears the active user session and encryption key from memory
#[tauri::command]
pub async fn logout_user(state: State<'_, AppState>) -> Result<ProfileResult<()>, String> {
    let mut user_session = state.user_session.lock().await;

    if let Some(session) = user_session.take() {
        log::info!("🔓 User '{}' logged out", session.username);

        // Cleanup logging
        crate::logging::cleanup();

        Ok(ProfileResult::success(()))
    } else {
        log::warn!("⚠️  No active session to logout");
        Ok(ProfileResult::error("No active session".to_string()))
    }
}

/// Get the current logged-in user's configuration
///
/// Returns None if no user is logged in
#[tauri::command]
pub async fn get_current_user(
    state: State<'_, AppState>,
) -> Result<ProfileResult<UserConfig>, String> {
    let user_session = state.user_session.lock().await;

    if let Some(session) = user_session.as_ref() {
        Ok(ProfileResult::success(session.config.clone()))
    } else {
        Ok(ProfileResult::error("No active session".to_string()))
    }
}

/// List all available user accounts
///
/// Returns list of usernames
#[tauri::command]
pub async fn list_users() -> Result<ProfileResult<Vec<String>>, String> {
    match UserManager::list_users(None) {
        Ok(users) => {
            log::info!("📋 Listed {} users", users.len());
            Ok(ProfileResult::success(users))
        }
        Err(e) => {
            log::error!("❌ Failed to list users: {}", e);
            Ok(ProfileResult::error(e.to_string()))
        }
    }
}

/// Get the current user's profile markdown content
///
/// # Returns
/// Profile markdown content or error if not logged in
#[tauri::command]
pub async fn get_user_profile(state: State<'_, AppState>) -> Result<ProfileResult<String>, String> {
    let user_session = state.user_session.lock().await;

    if let Some(session) = user_session.as_ref() {
        let encryption_key = session.get_encryption_key();

        match load_user_profile(&session.username, &encryption_key, None) {
            Ok(content) => {
                log::info!("✅ Profile loaded for user: {}", session.username);
                Ok(ProfileResult::success(content))
            }
            Err(e) => {
                log::warn!("⚠️  Profile not found or failed to load: {}", e);
                // Return empty content if profile doesn't exist yet
                Ok(ProfileResult::success(String::new()))
            }
        }
    } else {
        Ok(ProfileResult::error("No active session".to_string()))
    }
}

/// Update the current user's profile markdown content
///
/// # Parameters
/// - `content`: New markdown content for user-profile.md
///
/// # Returns
/// Success if saved, error message if failed
#[tauri::command]
pub async fn update_user_profile(
    state: State<'_, AppState>,
    content: String,
) -> Result<ProfileResult<()>, String> {
    let user_session = state.user_session.lock().await;

    if let Some(session) = user_session.as_ref() {
        let encryption_key = session.get_encryption_key();

        match save_user_profile(&session.username, &content, &encryption_key, None) {
            Ok(_) => {
                log::info!("✅ Profile updated for user: {}", session.username);
                Ok(ProfileResult::success(()))
            }
            Err(e) => {
                log::error!("❌ Failed to update profile: {}", e);
                Ok(ProfileResult::error(e.to_string()))
            }
        }
    } else {
        Ok(ProfileResult::error("No active session".to_string()))
    }
}

/// Check if any users exist in the system
///
/// Useful for determining if this is first launch
#[tauri::command]
pub async fn has_users() -> Result<ProfileResult<bool>, String> {
    match UserManager::list_users(None) {
        Ok(users) => Ok(ProfileResult::success(!users.is_empty())),
        Err(e) => {
            log::error!("❌ Failed to check users: {}", e);
            Ok(ProfileResult::error(e.to_string()))
        }
    }
}

// ============================================================================
// Command System Commands (Phase 3 - Markdown-based)
// ============================================================================

/// Save a command (markdown-based)
///
/// # Parameters
/// - `command`: The Command structure to save
///
/// # Returns
/// Success if saved, error message if failed
#[tauri::command]
pub async fn save_command(
    state: State<'_, AppState>,
    command: Command,
) -> Result<ProfileResult<()>, String> {
    let user_session = state.user_session.lock().await;

    if let Some(session) = user_session.as_ref() {
        let encryption_key = session.get_encryption_key();
        let manager = CommandManager::new(session.username.clone(), encryption_key);

        match manager.save_command(&command) {
            Ok(_) => {
                log::info!("Saved command '{}'", command.frontmatter.command_name);
                Ok(ProfileResult::success(()))
            }
            Err(e) => {
                log::error!("Failed to save command: {}", e);
                Ok(ProfileResult::error(e.to_string()))
            }
        }
    } else {
        Ok(ProfileResult::error("No active session".to_string()))
    }
}

/// Get a command by name (markdown-based)
///
/// # Parameters
/// - `name`: The command name to retrieve
///
/// # Returns
/// Command structure if found, error message if not found
#[tauri::command]
pub async fn get_command(
    state: State<'_, AppState>,
    name: String,
) -> Result<ProfileResult<Command>, String> {
    let user_session = state.user_session.lock().await;

    if let Some(session) = user_session.as_ref() {
        let encryption_key = session.get_encryption_key();
        let manager = CommandManager::new(session.username.clone(), encryption_key);

        match manager.load_command(&name) {
            Ok(command) => {
                log::info!("Command '{}' loaded", name);
                Ok(ProfileResult::success(command))
            }
            Err(e) => {
                log::error!("Failed to load command '{}': {}", name, e);
                Ok(ProfileResult::error(e.to_string()))
            }
        }
    } else {
        Ok(ProfileResult::error("No active session".to_string()))
    }
}

/// List all saved commands
///
/// # Returns
/// List of CommandInfo for all saved commands
#[tauri::command]
pub async fn list_commands(
    state: State<'_, AppState>,
) -> Result<ProfileResult<Vec<CommandInfo>>, String> {
    let user_session = state.user_session.lock().await;

    if let Some(session) = user_session.as_ref() {
        let encryption_key = session.get_encryption_key();
        let manager = CommandManager::new(session.username.clone(), encryption_key);

        match manager.list_commands() {
            Ok(commands) => {
                log::info!("📋 Listed {} commands", commands.len());
                Ok(ProfileResult::success(commands))
            }
            Err(e) => {
                log::error!("❌ Failed to list commands: {}", e);
                Ok(ProfileResult::error(e.to_string()))
            }
        }
    } else {
        Ok(ProfileResult::error("No active session".to_string()))
    }
}

/// Delete a command by name
///
/// # Parameters
/// - `name`: The command name to delete
///
/// # Returns
/// Success if deleted, error message if failed
#[tauri::command]
pub async fn delete_command(
    state: State<'_, AppState>,
    name: String,
) -> Result<ProfileResult<()>, String> {
    let user_session = state.user_session.lock().await;

    if let Some(session) = user_session.as_ref() {
        let encryption_key = session.get_encryption_key();
        let manager = CommandManager::new(session.username.clone(), encryption_key);

        match manager.delete_command(&name) {
            Ok(_) => {
                log::info!("✅ Command '{}' deleted", name);
                Ok(ProfileResult::success(()))
            }
            Err(e) => {
                log::error!("❌ Failed to delete command '{}': {}", name, e);
                Ok(ProfileResult::error(e.to_string()))
            }
        }
    } else {
        Ok(ProfileResult::error("No active session".to_string()))
    }
}

/// Build execution prompt for a command (AI-driven)
///
/// # Parameters
/// - `name`: The command name to execute
/// - `params`: Map of parameter names to values
///
/// # Returns
/// AI prompt string to send to LLM for CDP generation
#[tauri::command]
pub async fn build_command_prompt(
    state: State<'_, AppState>,
    name: String,
    params: HashMap<String, String>,
) -> Result<ProfileResult<String>, String> {
    let user_session = state.user_session.lock().await;

    if let Some(session) = user_session.as_ref() {
        let encryption_key = session.get_encryption_key();
        let executor = CommandExecutor::new(session.username.clone(), encryption_key.clone());

        // Load user profile if exists
        let user_profile = load_user_profile(&session.username, &encryption_key, None).ok();

        match executor.build_execution_prompt(&name, params, user_profile) {
            Ok(prompt) => {
                log::info!("Built execution prompt for command '{}'", name);
                Ok(ProfileResult::success(prompt))
            }
            Err(e) => {
                log::error!("Failed to build prompt for command '{}': {}", name, e);
                Ok(ProfileResult::error(e.to_string()))
            }
        }
    } else {
        Ok(ProfileResult::error("No active session".to_string()))
    }
}

/// Get static CDP script with parameter substitution (fallback)
///
/// # Parameters
/// - `name`: The command name to execute
/// - `params`: Map of parameter names to values
///
/// # Returns
/// CDP script JSON with substituted parameters, ready for execution
#[tauri::command]
pub async fn get_static_cdp(
    state: State<'_, AppState>,
    name: String,
    params: HashMap<String, String>,
) -> Result<ProfileResult<String>, String> {
    let user_session = state.user_session.lock().await;

    if let Some(session) = user_session.as_ref() {
        let encryption_key = session.get_encryption_key();
        let executor = CommandExecutor::new(session.username.clone(), encryption_key);

        match executor.get_static_cdp_script(&name, params) {
            Ok(script) => {
                log::info!("Retrieved static CDP for command '{}'", name);
                Ok(ProfileResult::success(script))
            }
            Err(e) => {
                log::error!("Failed to get static CDP for command '{}': {}", name, e);
                Ok(ProfileResult::error(e.to_string()))
            }
        }
    } else {
        Ok(ProfileResult::error("No active session".to_string()))
    }
}
