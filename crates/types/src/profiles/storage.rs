// Allow dead code for Phase 1 - many functions will be used in later phases
#![allow(dead_code)]

/// Filesystem operations for user profiles
///
/// This module handles all file I/O operations for the user profiles system,
/// including:
/// - Directory structure creation and management
/// - Encrypted file read/write operations
/// - Path resolution and validation
/// - Migration from older versions
///
/// File system structure:
/// ```text
/// ~/.facet/
/// ├── app-config.json          # Global application config
/// ├── users/
/// │   ├── alice/
/// │   │   ├── .salt            # Argon2id salt (16 bytes)
/// │   │   ├── user.json        # User configuration (encrypted)
/// │   │   ├── user-profile.md  # AI context document (encrypted)
/// │   │   ├── browser-profiles/
/// │   │   │   ├── default/     # Default browser profile
/// │   │   │   └── named/       # Named browser profiles
/// │   │   └── commands/
/// │   │       └── *.md         # Command files (encrypted)
/// │   └── bob/
/// │       └── ...
/// └── .tmp/
///     └── ephemeral-{uuid}/    # Temporary browser profiles
/// ```
use crate::profiles::{
    crypto::{decrypt_file, encrypt_file, EncryptionKey},
    types::UserConfig,
};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use thiserror::Error;

// ============================================================================
// Constants
// ============================================================================

/// Root directory name for Facet data
const ROBERT_DIR: &str = ".facet";

/// Directory name for user profiles
const USERS_DIR: &str = "users";

/// Directory name for temporary ephemeral profiles
const TMP_DIR: &str = ".tmp";

/// Filename for user configuration
const USER_CONFIG_FILE: &str = "user.json";

/// Filename for user profile markdown
const USER_PROFILE_FILE: &str = "user-profile.md";

/// Filename for salt storage
const SALT_FILE: &str = ".salt";

/// Directory name for browser profiles
const BROWSER_PROFILES_DIR: &str = "browser-profiles";

/// Directory name for commands
const COMMANDS_DIR: &str = "commands";

/// Default browser profile name
const DEFAULT_BROWSER_PROFILE: &str = "default";

// ============================================================================
// Error Types
// ============================================================================

#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum StorageError {
    /// I/O error during filesystem operations
    #[error("I/O error: {0}")]
    IoError(#[from] io::Error),

    /// JSON serialization/deserialization error
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// Cryptography error
    #[error("Crypto error: {0}")]
    CryptoError(#[from] crate::profiles::crypto::CryptoError),

    /// User directory not found
    #[error("User not found: {0}")]
    UserNotFound(String),

    /// Invalid path or filename
    #[error("Invalid path: {0}")]
    InvalidPath(String),

    /// Directory already exists
    #[error("Directory already exists: {0}")]
    AlreadyExists(String),

    /// Migration error
    #[error("Migration failed: {0}")]
    MigrationError(String),
}

pub type Result<T> = std::result::Result<T, StorageError>;

// ============================================================================
// Path Resolution
// ============================================================================

/// Get the root Facet directory path
///
/// Returns `~/.facet/` or platform-specific equivalent.
/// If `base_dir` is provided, uses `base_dir/.facet` instead.
/// This enables dependency injection for testing without modifying global environment variables.
///
/// # Parameters
/// - `base_dir`: Optional base directory for testing. If None, uses the user's home directory.
///
/// # Returns
/// - `PathBuf`: Path to the Facet directory
///
/// # Errors
/// - Returns error if home directory cannot be determined and base_dir is None
pub fn get_facet_dir(base_dir: Option<&Path>) -> Result<PathBuf> {
    let root = match base_dir {
        Some(dir) => dir.to_path_buf(),
        None => dirs::home_dir().ok_or_else(|| {
            StorageError::InvalidPath("Could not determine home directory".into())
        })?,
    };

    Ok(root.join(ROBERT_DIR))
}

/// Get the users directory path
///
/// Returns `~/.facet/users/` or `base_dir/.facet/users/` if base_dir is provided
pub fn get_users_dir(base_dir: Option<&Path>) -> Result<PathBuf> {
    Ok(get_facet_dir(base_dir)?.join(USERS_DIR))
}

/// Get the temporary directory path
///
/// Returns `~/.facet/.tmp/` or `base_dir/.facet/.tmp/` if base_dir is provided
pub fn get_tmp_dir(base_dir: Option<&Path>) -> Result<PathBuf> {
    Ok(get_facet_dir(base_dir)?.join(TMP_DIR))
}

/// Get a specific user's directory path
///
/// Returns `~/.facet/users/{username}/` or `base_dir/.facet/users/{username}/` if base_dir is provided
pub fn get_user_dir(username: &str, base_dir: Option<&Path>) -> Result<PathBuf> {
    validate_username(username)?;
    Ok(get_users_dir(base_dir)?.join(username))
}

/// Get a user's browser profiles directory
///
/// Returns `~/.facet/users/{username}/browser-profiles/`
pub fn get_browser_profiles_dir(username: &str, base_dir: Option<&Path>) -> Result<PathBuf> {
    Ok(get_user_dir(username, base_dir)?.join(BROWSER_PROFILES_DIR))
}

/// Get a specific browser profile directory
///
/// Returns `~/.facet/users/{username}/browser-profiles/{profile_name}/`
pub fn get_browser_profile_dir(
    username: &str,
    profile_name: &str,
    base_dir: Option<&Path>,
) -> Result<PathBuf> {
    validate_profile_name(profile_name)?;
    Ok(get_browser_profiles_dir(username, base_dir)?.join(profile_name))
}

/// Get a user's commands directory
///
/// Returns `~/.facet/users/{username}/commands/`
pub fn get_commands_dir(username: &str, base_dir: Option<&Path>) -> Result<PathBuf> {
    Ok(get_user_dir(username, base_dir)?.join(COMMANDS_DIR))
}

/// Get a specific command file path
///
/// Returns `~/.facet/users/{username}/commands/{command_name}.md`
pub fn get_command_path(
    username: &str,
    command_name: &str,
    base_dir: Option<&Path>,
) -> Result<PathBuf> {
    validate_command_name(command_name)?;
    Ok(get_commands_dir(username, base_dir)?.join(format!("{}.md", command_name)))
}

/// Get the salt file path for a user
///
/// Returns `~/.facet/users/{username}/.salt`
pub fn get_salt_path(username: &str, base_dir: Option<&Path>) -> Result<PathBuf> {
    Ok(get_user_dir(username, base_dir)?.join(SALT_FILE))
}

/// Get the user config file path
///
/// Returns `~/.facet/users/{username}/user.json`
pub fn get_user_config_path(username: &str, base_dir: Option<&Path>) -> Result<PathBuf> {
    Ok(get_user_dir(username, base_dir)?.join(USER_CONFIG_FILE))
}

/// Get the user profile markdown file path
///
/// Returns `~/.facet/users/{username}/user-profile.md`
pub fn get_user_profile_path(username: &str, base_dir: Option<&Path>) -> Result<PathBuf> {
    Ok(get_user_dir(username, base_dir)?.join(USER_PROFILE_FILE))
}

// ============================================================================
// Validation
// ============================================================================

/// Validate username format (alphanumeric, underscore, dash only)
fn validate_username(username: &str) -> Result<()> {
    if username.is_empty() {
        return Err(StorageError::InvalidPath("Username cannot be empty".into()));
    }

    if !username
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
    {
        return Err(StorageError::InvalidPath(format!(
            "Username '{}' contains invalid characters. Use only alphanumeric, underscore, and dash.",
            username
        )));
    }

    Ok(())
}

/// Validate profile name format
fn validate_profile_name(profile_name: &str) -> Result<()> {
    if profile_name.is_empty() {
        return Err(StorageError::InvalidPath(
            "Profile name cannot be empty".into(),
        ));
    }

    if !profile_name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
    {
        return Err(StorageError::InvalidPath(format!(
            "Profile name '{}' contains invalid characters",
            profile_name
        )));
    }

    Ok(())
}

/// Validate command name format (kebab-case)
fn validate_command_name(command_name: &str) -> Result<()> {
    if command_name.is_empty() {
        return Err(StorageError::InvalidPath(
            "Command name cannot be empty".into(),
        ));
    }

    if !command_name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-')
    {
        return Err(StorageError::InvalidPath(format!(
            "Command name '{}' contains invalid characters. Use kebab-case (alphanumeric and dash only)",
            command_name
        )));
    }

    Ok(())
}

// ============================================================================
// Directory Management
// ============================================================================

/// Initialize the Facet directory structure
///
/// Creates the root directory, users directory, and temporary directory
/// if they don't already exist.
///
/// # Parameters
/// - `base_dir`: Optional base directory for testing. If None, uses the user's home directory.
pub fn initialize_facet_dir(base_dir: Option<&Path>) -> Result<()> {
    let facet_dir = get_facet_dir(base_dir)?;
    let users_dir = get_users_dir(base_dir)?;
    let tmp_dir = get_tmp_dir(base_dir)?;

    fs::create_dir_all(&facet_dir)?;
    fs::create_dir_all(&users_dir)?;
    fs::create_dir_all(&tmp_dir)?;

    log::info!("Initialized Facet directory: {}", facet_dir.display());

    Ok(())
}

/// Create a new user directory structure
///
/// Creates:
/// - `~/.facet/users/{username}/`
/// - `~/.facet/users/{username}/browser-profiles/`
/// - `~/.facet/users/{username}/browser-profiles/default/`
/// - `~/.facet/users/{username}/commands/`
///
/// # Parameters
/// - `username`: The username for the new directory
/// - `base_dir`: Optional base directory for testing. If None, uses the user's home directory.
///
/// # Errors
/// Returns error if user directory already exists
pub fn create_user_directory(username: &str, base_dir: Option<&Path>) -> Result<PathBuf> {
    validate_username(username)?;

    let user_dir = get_user_dir(username, base_dir)?;

    // Check if user already exists
    if user_dir.exists() {
        return Err(StorageError::AlreadyExists(format!(
            "User '{}' already exists",
            username
        )));
    }

    // Create user directory
    fs::create_dir_all(&user_dir)?;

    // Create browser profiles directory
    let browser_profiles_dir = get_browser_profiles_dir(username, base_dir)?;
    fs::create_dir_all(&browser_profiles_dir)?;

    // Create default browser profile directory
    let default_profile_dir = get_browser_profile_dir(username, DEFAULT_BROWSER_PROFILE, base_dir)?;
    fs::create_dir_all(&default_profile_dir)?;

    // Create commands directory
    let commands_dir = get_commands_dir(username, base_dir)?;
    fs::create_dir_all(&commands_dir)?;

    log::info!("Created user directory: {}", user_dir.display());

    Ok(user_dir)
}

/// Create a named browser profile directory
pub fn create_browser_profile(
    username: &str,
    profile_name: &str,
    base_dir: Option<&Path>,
) -> Result<PathBuf> {
    validate_username(username)?;
    validate_profile_name(profile_name)?;

    let profile_dir = get_browser_profile_dir(username, profile_name, base_dir)?;

    if profile_dir.exists() {
        return Err(StorageError::AlreadyExists(format!(
            "Browser profile '{}' already exists",
            profile_name
        )));
    }

    fs::create_dir_all(&profile_dir)?;

    log::info!(
        "Created browser profile '{}' for user '{}'",
        profile_name,
        username
    );

    Ok(profile_dir)
}

/// Delete a browser profile directory
pub fn delete_browser_profile(
    username: &str,
    profile_name: &str,
    base_dir: Option<&Path>,
) -> Result<()> {
    validate_username(username)?;
    validate_profile_name(profile_name)?;

    // Prevent deletion of default profile
    if profile_name == DEFAULT_BROWSER_PROFILE {
        return Err(StorageError::InvalidPath(
            "Cannot delete default browser profile".into(),
        ));
    }

    let profile_dir = get_browser_profile_dir(username, profile_name, base_dir)?;

    if !profile_dir.exists() {
        return Err(StorageError::InvalidPath(format!(
            "Browser profile '{}' does not exist",
            profile_name
        )));
    }

    fs::remove_dir_all(&profile_dir)?;

    log::info!(
        "Deleted browser profile '{}' for user '{}'",
        profile_name,
        username
    );

    Ok(())
}

/// List all usernames
pub fn list_users(base_dir: Option<&Path>) -> Result<Vec<String>> {
    let users_dir = get_users_dir(base_dir)?;

    if !users_dir.exists() {
        return Ok(Vec::new());
    }

    let mut usernames = Vec::new();

    for entry in fs::read_dir(users_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                usernames.push(name.to_string());
            }
        }
    }

    usernames.sort();
    Ok(usernames)
}

/// Check if a user exists
pub fn user_exists(username: &str, base_dir: Option<&Path>) -> Result<bool> {
    let user_dir = get_user_dir(username, base_dir)?;
    Ok(user_dir.exists())
}

// ============================================================================
// Encrypted File Operations
// ============================================================================

/// Save salt to file
pub fn save_salt(username: &str, salt: &[u8], base_dir: Option<&Path>) -> Result<()> {
    let salt_path = get_salt_path(username, base_dir)?;
    fs::write(salt_path, salt)?;
    Ok(())
}

/// Load salt from file
pub fn load_salt(username: &str, base_dir: Option<&Path>) -> Result<Vec<u8>> {
    let salt_path = get_salt_path(username, base_dir)?;

    if !salt_path.exists() {
        return Err(StorageError::UserNotFound(format!(
            "Salt file not found for user '{}'",
            username
        )));
    }

    Ok(fs::read(salt_path)?)
}

/// Save user configuration (encrypted)
pub fn save_user_config(
    username: &str,
    config: &UserConfig,
    key: &EncryptionKey,
    base_dir: Option<&Path>,
) -> Result<()> {
    let config_path = get_user_config_path(username, base_dir)?;

    // Serialize to JSON
    let json = serde_json::to_string_pretty(config)?;

    // Encrypt
    let encrypted = encrypt_file(json.as_bytes(), key)?;

    // Write to file
    fs::write(config_path, encrypted)?;

    log::debug!("Saved user config for '{}'", username);

    Ok(())
}

/// Load user configuration (encrypted)
pub fn load_user_config(
    username: &str,
    key: &EncryptionKey,
    base_dir: Option<&Path>,
) -> Result<UserConfig> {
    let config_path = get_user_config_path(username, base_dir)?;

    if !config_path.exists() {
        return Err(StorageError::UserNotFound(format!(
            "User config not found for '{}'",
            username
        )));
    }

    // Read encrypted file
    let encrypted = fs::read(config_path)?;

    // Decrypt
    let decrypted = decrypt_file(&encrypted, key)?;

    // Parse JSON
    let config: UserConfig = serde_json::from_slice(&decrypted)?;

    log::debug!("Loaded user config for '{}'", username);

    Ok(config)
}

/// Save user profile markdown (encrypted)
pub fn save_user_profile(
    username: &str,
    content: &str,
    key: &EncryptionKey,
    base_dir: Option<&Path>,
) -> Result<()> {
    let profile_path = get_user_profile_path(username, base_dir)?;

    // Encrypt
    let encrypted = encrypt_file(content.as_bytes(), key)?;

    // Write to file
    fs::write(profile_path, encrypted)?;

    log::debug!("Saved user profile for '{}'", username);

    Ok(())
}

/// Load user profile markdown (encrypted)
pub fn load_user_profile(
    username: &str,
    key: &EncryptionKey,
    base_dir: Option<&Path>,
) -> Result<String> {
    let profile_path = get_user_profile_path(username, base_dir)?;

    if !profile_path.exists() {
        // Return default template if file doesn't exist
        return Ok(create_default_user_profile(username));
    }

    // Read encrypted file
    let encrypted = fs::read(profile_path)?;

    // Decrypt
    let decrypted = decrypt_file(&encrypted, key)?;

    // Convert to string
    let content = String::from_utf8(decrypted)
        .map_err(|e| StorageError::InvalidPath(format!("Invalid UTF-8 in user profile: {}", e)))?;

    log::debug!("Loaded user profile for '{}'", username);

    Ok(content)
}

/// Create default user profile markdown template
fn create_default_user_profile(username: &str) -> String {
    format!(
        r#"# User Profile: {}

## Preferences
- (Add your automation preferences here)
- Example: Detail level, privacy concerns, technical comfort

## Goals
- (What do you want to accomplish with Facet?)
- Example: Automate research, streamline shopping, track competitors

## Language Style
- (How do you prefer to communicate?)
- Example: Tone, formality, preferred format

## Additional Context
(You or the AI can add more sections over time)
"#,
        username
    )
}

/// Save command markdown (encrypted)
pub fn save_command(
    username: &str,
    command_name: &str,
    content: &str,
    key: &EncryptionKey,
    base_dir: Option<&Path>,
) -> Result<()> {
    validate_command_name(command_name)?;
    let command_path = get_command_path(username, command_name, base_dir)?;

    // Encrypt
    let encrypted = encrypt_file(content.as_bytes(), key)?;

    // Write to file
    fs::write(command_path, encrypted)?;

    log::debug!("Saved command '{}' for user '{}'", command_name, username);

    Ok(())
}

/// Load command markdown (encrypted)
pub fn load_command(
    username: &str,
    command_name: &str,
    key: &EncryptionKey,
    base_dir: Option<&Path>,
) -> Result<String> {
    validate_command_name(command_name)?;
    let command_path = get_command_path(username, command_name, base_dir)?;

    if !command_path.exists() {
        return Err(StorageError::InvalidPath(format!(
            "Command '{}' not found",
            command_name
        )));
    }

    // Read encrypted file
    let encrypted = fs::read(command_path)?;

    // Decrypt
    let decrypted = decrypt_file(&encrypted, key)?;

    // Convert to string
    let content = String::from_utf8(decrypted)
        .map_err(|e| StorageError::InvalidPath(format!("Invalid UTF-8 in command file: {}", e)))?;

    log::debug!("Loaded command '{}' for user '{}'", command_name, username);

    Ok(content)
}

/// List all command names for a user
pub fn list_commands(username: &str, base_dir: Option<&Path>) -> Result<Vec<String>> {
    let commands_dir = get_commands_dir(username, base_dir)?;

    if !commands_dir.exists() {
        return Ok(Vec::new());
    }

    let mut command_names = Vec::new();

    for entry in fs::read_dir(commands_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            if let Some(name) = path.file_stem().and_then(|n| n.to_str()) {
                command_names.push(name.to_string());
            }
        }
    }

    command_names.sort();
    Ok(command_names)
}

/// Delete a command
pub fn delete_command(username: &str, command_name: &str, base_dir: Option<&Path>) -> Result<()> {
    validate_command_name(command_name)?;
    let command_path = get_command_path(username, command_name, base_dir)?;

    if !command_path.exists() {
        return Err(StorageError::InvalidPath(format!(
            "Command '{}' does not exist",
            command_name
        )));
    }

    fs::remove_file(command_path)?;

    log::info!("Deleted command '{}' for user '{}'", command_name, username);

    Ok(())
}

// ============================================================================
// Ephemeral Profile Management
// ============================================================================

/// Create an ephemeral browser profile with UUID
pub fn create_ephemeral_profile(base_dir: Option<&Path>) -> Result<PathBuf> {
    let tmp_dir = get_tmp_dir(base_dir)?;
    let profile_id = uuid::Uuid::new_v4();
    let profile_dir = tmp_dir.join(format!("ephemeral-{}", profile_id));

    fs::create_dir_all(&profile_dir)?;

    log::debug!("Created ephemeral profile: {}", profile_dir.display());

    Ok(profile_dir)
}

/// Delete an ephemeral profile directory
pub fn delete_ephemeral_profile(profile_path: &Path, base_dir: Option<&Path>) -> Result<()> {
    if !profile_path.exists() {
        return Ok(()); // Already deleted
    }

    // Safety check: only delete from tmp directory
    let tmp_dir = get_tmp_dir(base_dir)?;
    if !profile_path.starts_with(&tmp_dir) {
        return Err(StorageError::InvalidPath(
            "Can only delete ephemeral profiles from tmp directory".into(),
        ));
    }

    fs::remove_dir_all(profile_path)?;

    log::debug!("Deleted ephemeral profile: {}", profile_path.display());

    Ok(())
}

/// Cleanup orphaned ephemeral profiles
///
/// Removes all ephemeral profile directories from tmp directory.
/// Should be called on app startup to clean up profiles from crashed sessions.
pub fn cleanup_ephemeral_profiles(base_dir: Option<&Path>) -> Result<usize> {
    let tmp_dir = get_tmp_dir(base_dir)?;

    if !tmp_dir.exists() {
        return Ok(0);
    }

    let mut count = 0;

    for entry in fs::read_dir(tmp_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.starts_with("ephemeral-") {
                    fs::remove_dir_all(&path)?;
                    count += 1;
                    log::debug!("Cleaned up ephemeral profile: {}", path.display());
                }
            }
        }
    }

    if count > 0 {
        log::info!("Cleaned up {} orphaned ephemeral profiles", count);
    }

    Ok(count)
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_username() {
        assert!(validate_username("alice").is_ok());
        assert!(validate_username("bob_123").is_ok());
        assert!(validate_username("user-name").is_ok());
        assert!(validate_username("").is_err());
        assert!(validate_username("user name").is_err());
        assert!(validate_username("user@name").is_err());
    }

    #[test]
    fn test_validate_profile_name() {
        assert!(validate_profile_name("shopping").is_ok());
        assert!(validate_profile_name("work-profile").is_ok());
        assert!(validate_profile_name("profile_123").is_ok());
        assert!(validate_profile_name("").is_err());
        assert!(validate_profile_name("profile name").is_err());
    }

    #[test]
    fn test_validate_command_name() {
        assert!(validate_command_name("clothing-search").is_ok());
        assert!(validate_command_name("check-prices").is_ok());
        assert!(validate_command_name("command123").is_ok());
        assert!(validate_command_name("").is_err());
        assert!(validate_command_name("command_name").is_err());
        assert!(validate_command_name("command name").is_err());
    }

    #[test]
    fn test_get_facet_dir() {
        let facet_dir = get_facet_dir(None).unwrap();
        assert!(facet_dir.ends_with(".facet"));
    }

    #[test]
    fn test_get_user_dir() {
        let user_dir = get_user_dir("alice", None).unwrap();
        assert!(user_dir.ends_with("users/alice"));
    }

    #[test]
    fn test_get_browser_profile_dir() {
        let profile_dir = get_browser_profile_dir("alice", "shopping", None).unwrap();
        assert!(profile_dir.ends_with("browser-profiles/shopping"));
    }

    #[test]
    fn test_get_command_path() {
        let command_path = get_command_path("alice", "clothing-search", None).unwrap();
        assert!(command_path.ends_with("commands/clothing-search.md"));
    }

    #[test]
    fn test_create_default_user_profile() {
        let profile = create_default_user_profile("alice");
        assert!(profile.contains("# User Profile: alice"));
        assert!(profile.contains("## Preferences"));
        assert!(profile.contains("## Goals"));
    }
}
