//! Encryption service for sensitive data at rest.
//!
//! Provides AES-256-GCM encryption for sensitive fields like TOTP secrets.
//! The encryption key should be stored separately from the database.

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use rand::RngCore;
use thiserror::Error;

/// Errors that can occur during encryption/decryption.
#[derive(Error, Debug)]
pub enum EncryptionError {
    #[error("Invalid key length: expected 32 bytes, got {0}")]
    InvalidKeyLength(usize),
    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),
    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),
    #[error("Invalid base64: {0}")]
    InvalidBase64(#[from] base64::DecodeError),
    #[error("Invalid ciphertext format")]
    InvalidFormat,
}

/// Result type for encryption operations.
pub type Result<T> = std::result::Result<T, EncryptionError>;

/// AES-256-GCM encryption service.
///
/// Uses a 256-bit key and generates random 96-bit nonces for each encryption.
/// Ciphertext format: base64(nonce || ciphertext || tag)
#[derive(Clone)]
pub struct EncryptionService {
    cipher: Aes256Gcm,
}

impl EncryptionService {
    /// Create a new encryption service from a base64-encoded key.
    ///
    /// The key must be exactly 32 bytes (256 bits) when decoded.
    pub fn from_base64_key(key_b64: &str) -> Result<Self> {
        let key = BASE64.decode(key_b64)?;
        Self::from_key(&key)
    }

    /// Create a new encryption service from a raw key.
    ///
    /// The key must be exactly 32 bytes (256 bits).
    pub fn from_key(key: &[u8]) -> Result<Self> {
        if key.len() != 32 {
            return Err(EncryptionError::InvalidKeyLength(key.len()));
        }

        let cipher = Aes256Gcm::new_from_slice(key)
            .map_err(|e| EncryptionError::EncryptionFailed(e.to_string()))?;

        Ok(Self { cipher })
    }

    /// Create encryption service from environment variable.
    ///
    /// Reads `ENCRYPTION_KEY` from environment (base64 encoded).
    /// Returns None if the key is not set.
    pub fn from_env() -> Option<Self> {
        std::env::var("ENCRYPTION_KEY")
            .ok()
            .and_then(|key| Self::from_base64_key(&key).ok())
    }

    /// Encrypt plaintext and return base64-encoded ciphertext.
    ///
    /// Format: base64(nonce || ciphertext || tag)
    /// Nonce: 12 bytes
    /// Tag: 16 bytes (GCM authentication tag)
    pub fn encrypt(&self, plaintext: &str) -> Result<String> {
        let mut nonce_bytes = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = self
            .cipher
            .encrypt(nonce, plaintext.as_bytes())
            .map_err(|e| EncryptionError::EncryptionFailed(e.to_string()))?;

        // Combine nonce + ciphertext
        let mut result = Vec::with_capacity(12 + ciphertext.len());
        result.extend_from_slice(&nonce_bytes);
        result.extend_from_slice(&ciphertext);

        Ok(BASE64.encode(result))
    }

    /// Decrypt base64-encoded ciphertext.
    ///
    /// Expects format: base64(nonce || ciphertext || tag)
    pub fn decrypt(&self, ciphertext_b64: &str) -> Result<String> {
        let data = BASE64.decode(ciphertext_b64)?;

        if data.len() < 12 + 16 {
            // Minimum: nonce (12) + tag (16)
            return Err(EncryptionError::InvalidFormat);
        }

        let (nonce_bytes, ciphertext) = data.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);

        let plaintext = self
            .cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| EncryptionError::DecryptionFailed(e.to_string()))?;

        String::from_utf8(plaintext)
            .map_err(|e| EncryptionError::DecryptionFailed(format!("Invalid UTF-8: {}", e)))
    }

    /// Check if a string appears to be encrypted (base64 encoded with our format).
    ///
    /// This is a heuristic check - it verifies the string is valid base64
    /// and has the minimum length for our format.
    pub fn is_encrypted(value: &str) -> bool {
        if BASE64.decode(value).is_err() {
            return false;
        }
        // Check minimum length for nonce + tag
        match BASE64.decode(value) {
            Ok(bytes) => bytes.len() >= 12 + 16,
            Err(_) => false,
        }
    }
}

/// Generate a new random encryption key.
///
/// Returns the key as a base64-encoded string.
pub fn generate_encryption_key() -> String {
    let mut key = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut key);
    BASE64.encode(key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_roundtrip() {
        let key = generate_encryption_key();
        let service = EncryptionService::from_base64_key(&key).unwrap();

        let plaintext = "my-secret-totp-key";
        let encrypted = service.encrypt(plaintext).unwrap();
        let decrypted = service.decrypt(&encrypted).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_different_nonces() {
        let key = generate_encryption_key();
        let service = EncryptionService::from_base64_key(&key).unwrap();

        let plaintext = "same-plaintext";
        let encrypted1 = service.encrypt(plaintext).unwrap();
        let encrypted2 = service.encrypt(plaintext).unwrap();

        // Same plaintext should produce different ciphertext (due to random nonce)
        assert_ne!(encrypted1, encrypted2);

        // Both should decrypt to the same value
        assert_eq!(service.decrypt(&encrypted1).unwrap(), plaintext);
        assert_eq!(service.decrypt(&encrypted2).unwrap(), plaintext);
    }

    #[test]
    fn test_is_encrypted() {
        let key = generate_encryption_key();
        let service = EncryptionService::from_base64_key(&key).unwrap();

        let plaintext = "test";
        let encrypted = service.encrypt(plaintext).unwrap();

        assert!(EncryptionService::is_encrypted(&encrypted));
        assert!(!EncryptionService::is_encrypted(plaintext));
        assert!(!EncryptionService::is_encrypted("not-base64!!!"));
    }

    #[test]
    fn test_invalid_key_length() {
        let short_key = BASE64.encode(b"short");
        let result = EncryptionService::from_base64_key(&short_key);
        assert!(matches!(result, Err(EncryptionError::InvalidKeyLength(5))));
    }

    #[test]
    fn test_decrypt_invalid_data() {
        let key = generate_encryption_key();
        let service = EncryptionService::from_base64_key(&key).unwrap();

        // Too short
        let short = BASE64.encode(b"short");
        assert!(service.decrypt(&short).is_err());

        // Valid base64 but wrong format
        let wrong_format = BASE64.encode(vec![0u8; 100]);
        assert!(service.decrypt(&wrong_format).is_err());
    }
}
