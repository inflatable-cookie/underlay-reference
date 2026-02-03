//! Application configuration with sensible defaults.
//!
//! This module provides a centralized configuration structure that consuming
//! apps can customize by overriding specific values.

/// Application configuration.
///
/// Provides sensible defaults for all settings. Consuming apps can override
/// specific values as needed.
///
/// # Example
///
/// ```
/// use acme_api::config::AcmeConfig;
///
/// // Use all defaults
/// let config = AcmeConfig::default();
///
/// // Override specific values
/// let config = AcmeConfig {
///     max_file_size_bytes: 100 * 1024 * 1024, // 100 MB
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Clone)]
pub struct AcmeConfig {
    /// Maximum allowed file size for media uploads in bytes.
    ///
    /// Default: 50 MB
    pub max_file_size_bytes: u64,

    /// Maximum thumbnail dimension (width or height) in pixels.
    ///
    /// Default: 300 px
    pub thumbnail_max_dimension: u32,
}

impl Default for AcmeConfig {
    fn default() -> Self {
        Self {
            max_file_size_bytes: 50 * 1024 * 1024, // 50 MB
            thumbnail_max_dimension: 300,
        }
    }
}

impl AcmeConfig {
    /// Create a new config with all defaults.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the maximum file size for uploads.
    pub fn max_file_size_mb(mut self, mb: u64) -> Self {
        self.max_file_size_bytes = mb * 1024 * 1024;
        self
    }

    /// Set the maximum thumbnail dimension.
    pub fn thumbnail_dimension(mut self, pixels: u32) -> Self {
        self.thumbnail_max_dimension = pixels;
        self
    }
}
