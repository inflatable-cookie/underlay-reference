//! Application configuration with sensible defaults.
//!
//! This module provides a centralized configuration structure for Acme,
//! combining reusable Underlay components with app-specific settings.

// Re-export MediaConfig from underlay-blob for convenience
pub use underlay_blob::MediaConfig;

/// Acme application configuration.
///
/// Combines Underlay's `MediaConfig` with any app-specific settings.
/// Provides sensible defaults that apps can override as needed.
///
/// # Example
///
/// ```ignore
/// use acme_api::config::AcmeConfig;
///
/// // Use all defaults
/// let config = AcmeConfig::default();
///
/// // Override media settings
/// let config = AcmeConfig::default()
///     .with_media(|m| m.max_file_size_mb(100));
///
/// // Or replace media config entirely
/// use acme_api::config::MediaConfig;
/// let config = AcmeConfig {
///     media: MediaConfig::default().max_file_size_mb(100),
/// };
/// ```
#[derive(Debug, Clone, Default)]
pub struct AcmeConfig {
    /// Media handling configuration (file size limits, thumbnail dimensions).
    pub media: MediaConfig,
}

impl AcmeConfig {
    /// Create a new config with all defaults.
    pub fn new() -> Self {
        Self::default()
    }

    /// Modify the media configuration.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let config = AcmeConfig::default()
    ///     .with_media(|m| m.max_file_size_mb(100).thumbnail_dimension(400));
    /// ```
    pub fn with_media<F>(mut self, f: F) -> Self
    where
        F: FnOnce(MediaConfig) -> MediaConfig,
    {
        self.media = f(self.media);
        self
    }
}
