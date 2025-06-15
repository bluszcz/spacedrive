//! BlackmagicRAW (BRAW) file support for Spacedrive
//!
//! This crate provides support for BlackmagicRAW video files, including:
//! - File detection and validation
//! - Metadata extraction
//! - Thumbnail generation
//! - Frame extraction
//!
//! ## Features
//!
//! - `with-sdk`: Enable actual BlackmagicRAW SDK integration (requires SDK installation)
//! - Default: Provides file detection and basic metadata without SDK
//!
//! ## Usage
//!
//! ```rust
//! use sd_braw::{BrawFile, thumbnail::generate_braw_thumbnail};
//!
//! // Basic file operations
//! let file = BrawFile::open("video.braw").await?;
//! let metadata = file.get_metadata().await?;
//!
//! // Generate thumbnail
//! let thumbnail = generate_braw_thumbnail(
//!     Path::new("video.braw"),
//!     ThumbnailConfig::default()
//! ).await?;
//! ```

pub mod error;
pub mod metadata;

#[cfg(feature = "with-sdk")]
pub mod sdk;
pub mod thumbnail;

// Re-export key types for convenience
pub use error::{BrawError, BrawResult, ErrorCategory};
pub use metadata::BrawMetadata;

#[cfg(feature = "with-sdk")]
pub use sdk::{BrawSdk, BrawClip, validate_braw_file};
pub use thumbnail::{
    ThumbnailSize, ThumbnailConfig,
    generate_braw_thumbnail, generate_braw_thumbnails,
    save_thumbnail
};
#[cfg(feature = "with-sdk")]
pub use thumbnail::{
    extract_frame_at_timestamp, generate_filmstrip_preview,
};

use std::path::Path;
use tokio::fs;
use tracing::debug;
#[cfg(feature = "with-sdk")]
use tracing::warn;

/// Magic bytes for BRAW file identification
pub const BRAW_MAGIC_BYTES: &[u8] = b"BRAW";

/// Main interface for BRAW file operations
#[derive(Debug)]
pub struct BrawFile {
    path: std::path::PathBuf,
    #[cfg(feature = "with-sdk")]
    clip: Option<sdk::BrawClip>,
}

impl BrawFile {
    /// Open a BRAW file
    pub async fn open<P: AsRef<Path>>(path: P) -> BrawResult<Self> {
        let path = path.as_ref().to_path_buf();

        // Validate file exists and is accessible
        if !path.exists() {
            return Err(BrawError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("File not found: {}", path.display())
            )));
        }

        // Basic validation without SDK
        if !is_braw_file(&path).await? {
            return Err(BrawError::InvalidFormat);
        }

        debug!("Opening BRAW file: {}", path.display());

        #[cfg(feature = "with-sdk")]
        {
            // Try to open with SDK
            match sdk::BrawSdk::new().await {
                Ok(sdk) => {
                    match sdk.open_clip(&path).await {
                        Ok(clip) => {
                            return Ok(BrawFile {
                                path,
                                clip: Some(clip),
                            });
                        }
                        Err(e) => {
                            warn!("Failed to open BRAW clip with SDK: {}, falling back to basic mode", e);
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to initialize BRAW SDK: {}, falling back to basic mode", e);
                }
            }
        }

        // Fallback to basic mode (no SDK)
        Ok(BrawFile {
            path,
            #[cfg(feature = "with-sdk")]
            clip: None,
        })
    }

    /// Get metadata from the BRAW file
    pub async fn get_metadata(&self) -> BrawResult<BrawMetadata> {
        #[cfg(feature = "with-sdk")]
        {
            if let Some(ref clip) = self.clip {
                return clip.get_metadata().await;
            }
        }

        // Fallback to basic metadata extraction
        extract_basic_metadata(&self.path).await
    }

    /// Check if the file was opened with SDK support
    #[cfg(feature = "with-sdk")]
    pub fn has_sdk_support(&self) -> bool {
        self.clip.is_some()
    }

    /// Check if the file was opened with SDK support (always false without SDK)
    #[cfg(not(feature = "with-sdk"))]
    pub fn has_sdk_support(&self) -> bool {
        false
    }

    /// Get the file path
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Generate a thumbnail (requires SDK support)
    #[cfg(feature = "with-sdk")]
    pub async fn generate_thumbnail(&self, config: crate::thumbnail::ThumbnailConfig) -> BrawResult<image::DynamicImage> {
        crate::thumbnail::generate_braw_thumbnail(&self.path, config).await
    }

    /// Generate a thumbnail (not available without SDK)
    #[cfg(not(feature = "with-sdk"))]
    pub async fn generate_thumbnail(&self, config: crate::thumbnail::ThumbnailConfig) -> BrawResult<image::DynamicImage> {
        crate::thumbnail::generate_braw_thumbnail(&self.path, config).await
    }

    /// Extract a frame at timestamp (requires SDK support)
    #[cfg(feature = "with-sdk")]
    pub async fn extract_frame_at_timestamp(&self, timestamp: f64) -> BrawResult<image::DynamicImage> {
        crate::thumbnail::extract_frame_at_timestamp(&self.path, timestamp).await
    }

    /// Extract a frame at timestamp (not available without SDK)
    #[cfg(not(feature = "with-sdk"))]
    pub async fn extract_frame_at_timestamp(&self, _timestamp: f64) -> BrawResult<()> {
        Err(BrawError::SdkUnavailable)
    }
}

/// Check if a file is a BRAW file by examining magic bytes
pub async fn is_braw_file<P: AsRef<Path>>(path: P) -> BrawResult<bool> {
    let path = path.as_ref();

    // Check file extension first (quick check)
    if let Some(ext) = path.extension() {
        if ext.to_string_lossy().to_lowercase() != "braw" {
            return Ok(false);
        }
    } else {
        return Ok(false);
    }

    // Check magic bytes
    let mut file = fs::File::open(path).await?;
    let mut buffer = [0u8; 4];

    use tokio::io::AsyncReadExt;
    match file.read_exact(&mut buffer).await {
        Ok(_) => Ok(&buffer == BRAW_MAGIC_BYTES),
        Err(_) => Ok(false), // File too small or read error
    }
}

/// Detect BRAW magic bytes in a buffer
pub fn detect_braw_magic_bytes(buffer: &[u8]) -> bool {
    buffer.len() >= BRAW_MAGIC_BYTES.len()
        && &buffer[0..BRAW_MAGIC_BYTES.len()] == BRAW_MAGIC_BYTES
}

/// Extract basic metadata without SDK (limited information)
async fn extract_basic_metadata(path: &Path) -> BrawResult<BrawMetadata> {
    debug!("Extracting basic metadata from: {}", path.display());

    // This is a fallback implementation that provides minimal metadata
    // In a real implementation, you might parse some basic BRAW header information

    let file_metadata = fs::metadata(path).await?;

    Ok(BrawMetadata {
        width: 0, // Unknown without SDK
        height: 0, // Unknown without SDK
        frame_rate: 0.0, // Unknown without SDK
        duration_seconds: 0.0, // Unknown without SDK
        total_frames: 0, // Unknown without SDK
        codec: "BlackmagicRAW".to_string(),
        color_space: Some("Unknown".to_string()),
        bit_depth: 0, // Unknown without SDK

        // File system metadata
        file_size: Some(file_metadata.len()),

        // All other fields remain None/default
        ..Default::default()
    })
}

/// Get SDK availability status
pub fn sdk_available() -> bool {
    cfg!(feature = "with-sdk")
}

/// Get SDK version information
#[cfg(feature = "with-sdk")]
pub fn sdk_version() -> &'static str {
    // This would be populated from actual SDK version info
    "BlackmagicRAW SDK (compiled with Rust bindings)"
}

/// Get SDK version information (fallback)
#[cfg(not(feature = "with-sdk"))]
pub fn sdk_version() -> &'static str {
    "BlackmagicRAW SDK not available (compile with --features with-sdk)"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_magic_bytes_detection() {
        assert!(detect_braw_magic_bytes(b"BRAW1234"));
        assert!(detect_braw_magic_bytes(b"BRAWDATA"));
        assert!(!detect_braw_magic_bytes(b"NOTBRAW"));
        assert!(!detect_braw_magic_bytes(b"BR"));
        assert!(!detect_braw_magic_bytes(b""));
    }

    #[test]
    fn test_sdk_availability() {
        // This will be different depending on compilation features
        let available = sdk_available();
        let version = sdk_version();

        #[cfg(feature = "with-sdk")]
        {
            assert!(available);
            assert!(version.contains("BlackmagicRAW SDK"));
        }

        #[cfg(not(feature = "with-sdk"))]
        {
            assert!(!available);
            assert!(version.contains("not available"));
        }
    }

    #[tokio::test]
    async fn test_basic_metadata_fallback() {
        // Test with a non-existent file should return error
        let result = extract_basic_metadata(Path::new("nonexistent.braw")).await;
        assert!(result.is_err());
    }
}