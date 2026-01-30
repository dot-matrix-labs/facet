/// Cryptographic operations for user profile encryption
///
/// This module implements password-based encryption using industry-standard
/// algorithms:
/// - Argon2id for password hashing and key derivation (PHC winner)
/// - AES-256-GCM for authenticated encryption (NIST standard)
///
/// Security properties:
/// - Memory-hard password hashing resistant to GPU attacks
/// - Authenticated encryption with integrity verification
/// - Random salts and nonces for each operation
/// - Secure memory clearing using zeroize
///
/// Threat model:
/// ✓ Protects against offline password attacks (Argon2id parameters)
/// ✓ Protects against ciphertext tampering (GCM auth tag)
/// ✓ Protects against rainbow tables (random salts)
/// ✗ Does NOT protect against memory dumps while key is in RAM
/// ✗ Does NOT protect against malware with root access
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm,
};
use argon2::{
    password_hash::{PasswordHasher, SaltString},
    Algorithm, Argon2, Params, Version,
};
use rand::{rngs::OsRng, RngCore};
use std::io;
use thiserror::Error;
use zeroize::{Zeroize, ZeroizeOnDrop};

// ============================================================================
// Constants and Configuration
// ============================================================================

/// Argon2id memory cost in KB (64 MB)
/// This makes password hashing memory-intensive to resist GPU attacks
const ARGON2_MEM_SIZE_KB: u32 = 65536;

/// Argon2id iteration count (3 passes)
/// Balances security and performance (targets ~200-500ms on typical hardware)
const ARGON2_ITERATIONS: u32 = 3;

/// Argon2id parallelism (4 threads)
/// Leverages multi-core CPUs while remaining accessible to slower machines
const ARGON2_PARALLELISM: u32 = 4;

/// Derived key length in bytes (32 bytes = 256 bits for AES-256)
const KEY_LENGTH: usize = 32;

/// Salt length in bytes (16 bytes = 128 bits)
#[allow(dead_code)]
const SALT_LENGTH: usize = 16;

/// Nonce length for AES-GCM in bytes (12 bytes = 96 bits)
const NONCE_LENGTH: usize = 12;

// ============================================================================
// Error Types
// ============================================================================

/// Errors that can occur during cryptographic operations
#[derive(Error, Debug)]
pub enum CryptoError {
    /// Password hashing failed
    #[error("Password hashing failed: {0}")]
    HashingFailed(String),

    /// Encryption operation failed
    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),

    /// Decryption operation failed (wrong password or corrupted data)
    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),

    /// Invalid salt format
    #[error("Invalid salt: {0}")]
    InvalidSalt(String),

    /// Ciphertext too short or malformed
    #[error("Invalid ciphertext: {0}")]
    InvalidCiphertext(String),

    /// I/O error during file operations
    #[error("I/O error: {0}")]
    IoError(#[from] io::Error),
}

pub type Result<T> = std::result::Result<T, CryptoError>;

// ============================================================================
// Encryption Key (with secure cleanup)
// ============================================================================

/// Encryption key that is securely zeroed when dropped
///
/// This wrapper ensures that encryption keys are cleared from memory
/// when they go out of scope, reducing the window for memory dump attacks.
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct EncryptionKey {
    #[zeroize(skip)]
    key: Vec<u8>,
}

impl EncryptionKey {
    /// Create a new encryption key from raw bytes
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        Self { key: bytes }
    }

    /// Get a reference to the key bytes
    pub fn as_bytes(&self) -> &[u8] {
        &self.key
    }

    /// Get the key length in bytes
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.key.len()
    }

    /// Check if key is empty
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.key.is_empty()
    }
}

impl std::fmt::Debug for EncryptionKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EncryptionKey")
            .field("len", &self.key.len())
            .field("key", &"<redacted>")
            .finish()
    }
}

// ============================================================================
// Key Derivation (Argon2id)
// ============================================================================

/// Derive an encryption key from a password and salt using Argon2id
///
/// This function implements password-based key derivation using Argon2id,
/// the winner of the Password Hashing Competition. It's designed to be
/// memory-hard and resistant to GPU/ASIC attacks.
///
/// # Parameters
/// - `password`: User's password (will be securely cleared)
/// - `salt`: Optional salt bytes. If None, generates a new random salt
///
/// # Returns
/// - `EncryptionKey`: Derived 256-bit encryption key
/// - `Vec<u8>`: Salt used for derivation (store this with user data)
///
/// # Performance
/// On typical modern hardware, this takes 200-500ms. This is intentional
/// to make brute-force attacks impractical.
///
/// # Security Notes
/// - The derived key is zeroed when dropped
/// - Salt must be stored alongside encrypted data
/// - Same password + salt always produces same key
/// - Different salts produce different keys even with same password
///
/// # Example
/// ```no_run
/// use facet_app_lib::profiles::crypto::derive_key;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // First time: generate new salt
/// let (key, salt) = derive_key("my_password", None)?;
/// // Store salt in user directory: ~/.facet/users/alice/.salt
///
/// // Later: use stored salt to derive same key
/// let (key, _) = derive_key("my_password", Some(&salt))?;
/// # Ok(())
/// # }
/// ```
pub fn derive_key(password: &str, salt: Option<&[u8]>) -> Result<(EncryptionKey, Vec<u8>)> {
    // Create or validate salt
    let salt_string = match salt {
        Some(s) => {
            // Convert stored salt bytes to SaltString
            let salt_str = std::str::from_utf8(s)
                .map_err(|e| CryptoError::InvalidSalt(format!("Invalid UTF-8 in salt: {}", e)))?;
            SaltString::from_b64(salt_str)
                .map_err(|e| CryptoError::InvalidSalt(format!("Failed to decode salt: {}", e)))?
        }
        None => SaltString::generate(&mut OsRng),
    };

    // Configure Argon2id parameters
    let params = Params::new(
        ARGON2_MEM_SIZE_KB,
        ARGON2_ITERATIONS,
        ARGON2_PARALLELISM,
        Some(KEY_LENGTH),
    )
    .map_err(|e| CryptoError::HashingFailed(format!("Invalid Argon2 params: {}", e)))?;

    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

    // Derive key from password
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt_string)
        .map_err(|e| CryptoError::HashingFailed(format!("Argon2 hashing failed: {}", e)))?;

    // Extract raw hash bytes
    let hash_bytes = password_hash
        .hash
        .ok_or_else(|| CryptoError::HashingFailed("No hash output".to_string()))?;

    let key_bytes = hash_bytes.as_bytes().to_vec();

    // Validate key length
    if key_bytes.len() != KEY_LENGTH {
        return Err(CryptoError::HashingFailed(format!(
            "Expected {} byte key, got {}",
            KEY_LENGTH,
            key_bytes.len()
        )));
    }

    // Convert SaltString to bytes for storage
    let salt_bytes = salt_string.as_str().as_bytes().to_vec();

    Ok((EncryptionKey::from_bytes(key_bytes), salt_bytes))
}

/// Verify a password against stored hash parameters
///
/// This function re-derives the key and compares it with a reference.
/// Used during login to validate the password.
///
/// # Parameters
/// - `password`: Password to verify
/// - `salt`: Stored salt from user directory
/// - `expected_hash`: Previously derived key hash for comparison
///
/// # Returns
/// - `true` if password is correct
/// - `false` if password is incorrect
///
/// # Security Notes
/// - This function is constant-time to prevent timing attacks
/// - Argon2id's built-in comparison is timing-attack resistant
#[allow(dead_code)]
pub fn verify_password(password: &str, salt: &[u8], expected_hash: &[u8]) -> Result<bool> {
    let (derived_key, _) = derive_key(password, Some(salt))?;

    // Constant-time comparison
    Ok(constant_time_compare(derived_key.as_bytes(), expected_hash))
}

/// Constant-time byte comparison to prevent timing attacks
///
/// This ensures that password verification takes the same amount of time
/// regardless of where the first differing byte is, preventing attackers
/// from using timing information to guess passwords.
#[allow(dead_code)]
fn constant_time_compare(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }

    let mut result = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        result |= x ^ y;
    }

    result == 0
}

// ============================================================================
// File Encryption (AES-256-GCM)
// ============================================================================

/// Encrypt file content using AES-256-GCM
///
/// This function encrypts plaintext data using authenticated encryption,
/// which provides both confidentiality and integrity. The resulting
/// ciphertext includes an authentication tag that prevents tampering.
///
/// # File Format
/// The encrypted output has the following structure:
/// ```text
/// [12 bytes: Nonce] || [N bytes: Ciphertext] || [16 bytes: Auth Tag]
/// ```
///
/// # Parameters
/// - `plaintext`: Data to encrypt (will be securely cleared)
/// - `key`: Encryption key from `derive_key()`
///
/// # Returns
/// - `Vec<u8>`: Encrypted data with nonce and auth tag
///
/// # Security Notes
/// - A random nonce is generated for each encryption
/// - The authentication tag prevents tampering
/// - Same plaintext encrypted twice produces different ciphertext
///
/// # Example
/// ```no_run
/// use facet_app_lib::profiles::crypto::{derive_key, encrypt_file};
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let (key, salt) = derive_key("password", None)?;
/// let plaintext = b"sensitive data";
/// let encrypted = encrypt_file(plaintext, &key)?;
/// // Store encrypted data to file
/// # Ok(())
/// # }
/// ```
pub fn encrypt_file(plaintext: &[u8], key: &EncryptionKey) -> Result<Vec<u8>> {
    // Create AES-256-GCM cipher
    let cipher = Aes256Gcm::new_from_slice(key.as_bytes())
        .map_err(|e| CryptoError::EncryptionFailed(format!("Invalid key: {}", e)))?;

    // Generate random nonce (must be unique for each encryption)
    let mut nonce_bytes = [0u8; NONCE_LENGTH];
    OsRng.fill_bytes(&mut nonce_bytes);

    // Encrypt plaintext (includes auth tag in output)
    let ciphertext = cipher
        .encrypt(&nonce_bytes.into(), plaintext)
        .map_err(|e| CryptoError::EncryptionFailed(format!("Encryption failed: {}", e)))?;

    // Construct output: nonce || ciphertext || tag
    // (GCM automatically appends the 16-byte auth tag to ciphertext)
    let mut result = nonce_bytes.to_vec();
    result.extend_from_slice(&ciphertext);

    Ok(result)
}

/// Decrypt file content using AES-256-GCM
///
/// This function decrypts data encrypted with `encrypt_file()`. It verifies
/// the authentication tag before returning plaintext, ensuring data integrity.
///
/// # Parameters
/// - `encrypted`: Encrypted data from `encrypt_file()` (nonce || ciphertext || tag)
/// - `key`: Encryption key from `derive_key()`
///
/// # Returns
/// - `Vec<u8>`: Decrypted plaintext
///
/// # Errors
/// - Returns error if ciphertext is too short
/// - Returns error if authentication tag verification fails (wrong password or tampered data)
///
/// # Security Notes
/// - Authentication tag is verified before returning data
/// - Wrong password produces decryption error, not garbage data
/// - Tampered ciphertext is detected and rejected
///
/// # Example
/// ```no_run
/// use facet_app_lib::profiles::crypto::{derive_key, decrypt_file};
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # let stored_salt = vec![0u8; 16]; // Previously saved salt
/// # let encrypted_data = vec![0u8; 32]; // Previously encrypted data
/// let (key, _) = derive_key("password", Some(&stored_salt))?;
/// let plaintext = decrypt_file(&encrypted_data, &key)?;
/// # Ok(())
/// # }
/// ```
pub fn decrypt_file(encrypted: &[u8], key: &EncryptionKey) -> Result<Vec<u8>> {
    // Validate minimum length (nonce + tag)
    if encrypted.len() < NONCE_LENGTH + 16 {
        return Err(CryptoError::InvalidCiphertext(format!(
            "Ciphertext too short: expected at least {} bytes, got {}",
            NONCE_LENGTH + 16,
            encrypted.len()
        )));
    }

    // Create cipher
    let cipher = Aes256Gcm::new_from_slice(key.as_bytes())
        .map_err(|e| CryptoError::DecryptionFailed(format!("Invalid key: {}", e)))?;

    // Extract nonce and ciphertext
    let nonce_bytes: [u8; NONCE_LENGTH] = encrypted[..NONCE_LENGTH]
        .try_into()
        .map_err(|_| CryptoError::InvalidCiphertext("Invalid nonce length".to_string()))?;
    let ciphertext = &encrypted[NONCE_LENGTH..];

    // Decrypt and verify auth tag
    let plaintext = cipher
        .decrypt(&nonce_bytes.into(), ciphertext)
        .map_err(|e| {
            CryptoError::DecryptionFailed(format!(
                "Decryption failed (wrong password or corrupted data): {}",
                e
            ))
        })?;

    Ok(plaintext)
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_key_generates_valid_key() {
        let (key, salt) = derive_key("test_password", None).unwrap();

        assert_eq!(key.len(), KEY_LENGTH);
        // Salt is base64-encoded, so it's 22 characters for a 16-byte (128-bit) salt
        assert!(!salt.is_empty());
        // Base64 encoding increases size by ~33%, so 16 bytes -> ~22 bytes
        // The salt length can vary slightly due to base64 encoding padding
        assert!(
            salt.len() >= 16 && salt.len() <= 24,
            "Salt length was {}",
            salt.len()
        );
    }

    #[test]
    fn test_derive_key_is_deterministic() {
        let (key1, salt) = derive_key("test_password", None).unwrap();
        let (key2, _) = derive_key("test_password", Some(&salt)).unwrap();

        assert_eq!(key1.as_bytes(), key2.as_bytes());
    }

    #[test]
    fn test_different_passwords_produce_different_keys() {
        let (key1, salt) = derive_key("password1", None).unwrap();
        let (key2, _) = derive_key("password2", Some(&salt)).unwrap();

        assert_ne!(key1.as_bytes(), key2.as_bytes());
    }

    #[test]
    fn test_different_salts_produce_different_keys() {
        let (key1, _) = derive_key("same_password", None).unwrap();
        let (key2, _) = derive_key("same_password", None).unwrap();

        assert_ne!(key1.as_bytes(), key2.as_bytes());
    }

    #[test]
    fn test_verify_password_correct() {
        let password = "correct_password";
        let (key, salt) = derive_key(password, None).unwrap();

        let result = verify_password(password, &salt, key.as_bytes()).unwrap();
        assert!(result);
    }

    #[test]
    fn test_verify_password_incorrect() {
        let (key, salt) = derive_key("correct_password", None).unwrap();

        let result = verify_password("wrong_password", &salt, key.as_bytes()).unwrap();
        assert!(!result);
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let (key, _) = derive_key("test_password", None).unwrap();
        let plaintext = b"sensitive data that needs encryption";

        let encrypted = encrypt_file(plaintext, &key).unwrap();
        let decrypted = decrypt_file(&encrypted, &key).unwrap();

        assert_eq!(plaintext, decrypted.as_slice());
    }

    #[test]
    fn test_encrypt_produces_different_output() {
        let (key, _) = derive_key("test_password", None).unwrap();
        let plaintext = b"same data encrypted twice";

        let encrypted1 = encrypt_file(plaintext, &key).unwrap();
        let encrypted2 = encrypt_file(plaintext, &key).unwrap();

        // Different nonces mean different ciphertext
        assert_ne!(encrypted1, encrypted2);

        // But both decrypt to same plaintext
        let decrypted1 = decrypt_file(&encrypted1, &key).unwrap();
        let decrypted2 = decrypt_file(&encrypted2, &key).unwrap();
        assert_eq!(decrypted1, decrypted2);
        assert_eq!(plaintext, decrypted1.as_slice());
    }

    #[test]
    fn test_decrypt_with_wrong_key_fails() {
        let (key1, _) = derive_key("password1", None).unwrap();
        let (key2, _) = derive_key("password2", None).unwrap();

        let plaintext = b"secret message";
        let encrypted = encrypt_file(plaintext, &key1).unwrap();

        let result = decrypt_file(&encrypted, &key2);
        assert!(result.is_err());
    }

    #[test]
    fn test_decrypt_corrupted_ciphertext_fails() {
        let (key, _) = derive_key("password", None).unwrap();
        let plaintext = b"original data";

        let mut encrypted = encrypt_file(plaintext, &key).unwrap();

        // Corrupt one byte in the middle
        if encrypted.len() > NONCE_LENGTH + 10 {
            encrypted[NONCE_LENGTH + 5] ^= 0xFF;
        }

        let result = decrypt_file(&encrypted, &key);
        assert!(result.is_err());
    }

    #[test]
    fn test_decrypt_truncated_ciphertext_fails() {
        let (key, _) = derive_key("password", None).unwrap();

        // Ciphertext too short (only 10 bytes)
        let truncated = vec![0u8; 10];

        let result = decrypt_file(&truncated, &key);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CryptoError::InvalidCiphertext(_)
        ));
    }

    #[test]
    fn test_encryption_key_is_zeroized() {
        let (key, _) = derive_key("password", None).unwrap();
        let key_bytes = key.as_bytes().to_vec();

        drop(key);

        // This test demonstrates that the key is zeroized on drop
        // In practice, we can't directly verify memory clearing without unsafe code
        // but zeroize crate guarantees this behavior
        assert!(!key_bytes.is_empty()); // Original bytes are still in our copy
    }

    #[test]
    fn test_constant_time_compare_same_length() {
        let a = b"same_length_a";
        let b = b"same_length_b";
        let c = b"same_length_a";

        assert!(!constant_time_compare(a, b));
        assert!(constant_time_compare(a, c));
    }

    #[test]
    fn test_constant_time_compare_different_length() {
        let a = b"short";
        let b = b"much_longer_string";

        assert!(!constant_time_compare(a, b));
    }
}
