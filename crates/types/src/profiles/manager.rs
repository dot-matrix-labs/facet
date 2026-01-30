/// User management operations (CRUD)
///
/// This module implements high-level user management operations including:
/// - User creation with encryption setup
/// - User configuration updates
/// - User deletion
/// - Default user initialization
///
/// All operations ensure data consistency and proper encryption.
// Allow dead code for Phase 1 - these will be used when Tauri commands are implemented
#[allow(dead_code)]
use crate::profiles::{
    crypto::{derive_key, EncryptionKey},
    storage::{
        create_user_directory, list_users as storage_list_users, load_salt, load_user_config,
        save_salt, save_user_config, save_user_profile, user_exists,
    },
    types::{UserConfig, UserPreferences},
};
use chrono::Utc;
use std::collections::HashMap;
use thiserror::Error;

// ============================================================================
// Error Types
// ============================================================================

#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum ManagerError {
    /// Storage error
    #[error("Storage error: {0}")]
    StorageError(#[from] crate::profiles::storage::StorageError),

    /// Cryptography error
    #[error("Crypto error: {0}")]
    CryptoError(#[from] crate::profiles::crypto::CryptoError),

    /// User already exists
    #[error("User already exists: {0}")]
    UserExists(String),

    /// User not found
    #[error("User not found: {0}")]
    UserNotFound(String),

    /// Invalid username format
    #[error("Invalid username: {0}")]
    InvalidUsername(String),

    /// Password validation failed
    #[error("Invalid password: {0}")]
    InvalidPassword(String),
}

pub type Result<T> = std::result::Result<T, ManagerError>;

// ============================================================================
// User Manager
// ============================================================================

/// User management operations
#[allow(dead_code)]
pub struct UserManager;

#[allow(dead_code)]
impl UserManager {
    /// Create a new user with password encryption
    ///
    /// # Parameters
    /// - `username`: Unique username (alphanumeric, underscore, dash)
    /// - `password`: User password (will be hashed with Argon2id)
    /// - `base_dir`: Optional base directory for testing. If None, uses the user's home directory.
    ///
    /// # Returns
    /// - `EncryptionKey`: Derived encryption key (store in app state)
    /// - `UserConfig`: Created user configuration
    ///
    /// # Errors
    /// - Returns error if user already exists
    /// - Returns error if username is invalid
    /// - Returns error if password is too weak
    pub fn create_user(
        username: &str,
        password: &str,
        base_dir: Option<&std::path::Path>,
    ) -> Result<(EncryptionKey, UserConfig)> {
        // Validate username
        Self::validate_username(username)?;

        // Validate password
        Self::validate_password(password)?;

        // Check if user already exists
        if user_exists(username, base_dir)? {
            return Err(ManagerError::UserExists(username.to_string()));
        }

        // Create user directory structure
        create_user_directory(username, base_dir)?;

        // Derive encryption key from password
        let (key, salt) = derive_key(password, None)?;

        // Save salt
        save_salt(username, &salt, base_dir)?;

        // Create default user config
        let config = UserConfig {
            username: username.to_string(),
            created_at: Utc::now(),
            last_login: Utc::now(),
            browser_profiles: HashMap::new(),
            default_browser_profile: None,
            preferences: UserPreferences::default(),
            stats: Default::default(),
        };

        // Save encrypted user config
        save_user_config(username, &config, &key, base_dir)?;

        // Create default user profile markdown
        let profile_content = format!(
            r#"# User Profile: {}

## Preferences
- (Add your automation preferences here)

## Goals
- (What do you want to accomplish with Facet?)

## Language Style
- (How do you prefer to communicate?)
"#,
            username
        );

        save_user_profile(username, &profile_content, &key, base_dir)?;

        log::info!("Created user: {}", username);

        Ok((key, config))
    }

    /// List all usernames
    pub fn list_users(base_dir: Option<&std::path::Path>) -> Result<Vec<String>> {
        Ok(storage_list_users(base_dir)?)
    }

    /// Load user configuration with password
    ///
    /// # Parameters
    /// - `username`: Username to load
    /// - `password`: User's password
    /// - `base_dir`: Optional base directory for testing. If None, uses the user's home directory.
    ///
    /// # Returns
    /// - `EncryptionKey`: Derived encryption key (store in app state)
    /// - `UserConfig`: Loaded user configuration
    ///
    /// # Errors
    /// - Returns error if user not found
    /// - Returns error if password is incorrect
    pub fn load_user(
        username: &str,
        password: &str,
        base_dir: Option<&std::path::Path>,
    ) -> Result<(EncryptionKey, UserConfig)> {
        // Load salt
        let salt = load_salt(username, base_dir)?;

        // Derive key from password
        let (key, _) = derive_key(password, Some(&salt))?;

        // Try to load and decrypt user config
        let config = load_user_config(username, &key, base_dir).map_err(|e| {
            log::warn!("Failed to decrypt user config for '{}': {}", username, e);
            ManagerError::CryptoError(crate::profiles::crypto::CryptoError::DecryptionFailed(
                "Incorrect password or corrupted data".to_string(),
            ))
        })?;

        log::info!("Loaded user: {}", username);

        Ok((key, config))
    }

    /// Update user's last login timestamp
    pub fn update_last_login(
        username: &str,
        config: &mut UserConfig,
        key: &EncryptionKey,
        base_dir: Option<&std::path::Path>,
    ) -> Result<()> {
        config.last_login = Utc::now();
        save_user_config(username, config, key, base_dir)?;
        Ok(())
    }

    /// Validate username format
    fn validate_username(username: &str) -> Result<()> {
        if username.is_empty() {
            return Err(ManagerError::InvalidUsername(
                "Username cannot be empty".into(),
            ));
        }

        if username.len() > 32 {
            return Err(ManagerError::InvalidUsername(
                "Username must be 32 characters or less".into(),
            ));
        }

        if !username
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
        {
            return Err(ManagerError::InvalidUsername(
                "Username can only contain alphanumeric characters, underscore, and dash".into(),
            ));
        }

        Ok(())
    }

    /// Validate password strength
    fn validate_password(password: &str) -> Result<()> {
        if password.len() < 12 {
            return Err(ManagerError::InvalidPassword(
                "Password must be at least 12 characters".into(),
            ));
        }

        Ok(())
    }
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_username_valid() {
        assert!(UserManager::validate_username("alice").is_ok());
        assert!(UserManager::validate_username("bob_123").is_ok());
        assert!(UserManager::validate_username("user-name").is_ok());
    }

    #[test]
    fn test_validate_username_invalid() {
        assert!(UserManager::validate_username("").is_err());
        assert!(UserManager::validate_username("user name").is_err());
        assert!(UserManager::validate_username("user@email").is_err());
        assert!(UserManager::validate_username("a".repeat(33).as_str()).is_err());
    }

    #[test]
    fn test_validate_password_valid() {
        assert!(UserManager::validate_password("correct_horse_battery_staple").is_ok());
        assert!(UserManager::validate_password("P@ssw0rd1234").is_ok());
    }

    #[test]
    fn test_validate_password_invalid() {
        assert!(UserManager::validate_password("").is_err());
        assert!(UserManager::validate_password("short").is_err());
        assert!(UserManager::validate_password("only11chars").is_err());
    }
}
