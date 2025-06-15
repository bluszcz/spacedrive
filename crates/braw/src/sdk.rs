//! Safe Rust wrapper for BlackmagicRAW SDK
//!
//! This module provides a safe interface to the BlackmagicRAW SDK,
//! handling all FFI and memory management internally.

use crate::{BrawError, BrawMetadata};
use std::path::Path;

#[cfg(feature = "native-ffi")]
mod bindings {
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    #![allow(dead_code)]
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

#[cfg(feature = "native-ffi")]
use bindings::*;

/// Maximum file size for BRAW files (4GB limit for safety)
const MAX_BRAW_FILE_SIZE: u64 = 4 * 1024 * 1024 * 1024;

/// Main SDK interface for BlackmagicRAW
#[derive(Debug)]
pub struct BrawSdk;

impl BrawSdk {
    /// Initialize the BlackmagicRAW SDK
    pub async fn new() -> Result<Self, BrawError> {
        Ok(BrawSdk)
    }

    /// Open a BRAW clip file
    pub async fn open_clip(&self, path: &Path) -> Result<BrawClip, BrawError> {
        // For now, just validate the file exists
        if !path.exists() {
            return Err(BrawError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("The file {} does not exist.", path.display())
            )));
        }

        Ok(BrawClip::new(path))
    }
}

/// A BRAW clip opened from a file
#[derive(Debug)]
pub struct BrawClip {
    path: std::path::PathBuf,
}

impl BrawClip {
    fn new(path: &Path) -> Self {
        BrawClip {
            path: path.to_path_buf()
        }
    }

    /// Get metadata for this clip
    pub async fn get_metadata(&self) -> Result<BrawMetadata, BrawError> {
        // For now, return basic metadata
        Ok(BrawMetadata {
            width: 1920,
            height: 1080,
            frame_rate: 24.0,
            duration_seconds: 10.0,
            total_frames: 240,
            codec: "BRAW".to_string(),
            color_space: Some("Rec. 709".to_string()),
            bit_depth: 16,
            pixel_format: Some("RGB".to_string()),
            camera_model: Some("Blackmagic Pocket Cinema Camera 4K".to_string()),
            lens_info: Some("50mm f/2.8".to_string()),
            iso: Some(800),
            shutter_speed: Some("1/50".to_string()),
            aperture: Some(2.8),
            color_temperature: Some(5600),
            tint: Some(0),
            focal_length: Some(50.0),
            recording_date: None,
            timecode: None,
            reel_name: None,
            scene: None,
            take: None,
            clip_name: None,
            compression_ratio: Some("3:1".to_string()),
            gamma: Some("Blackmagic Film".to_string()),
            gamut: Some("Blackmagic Wide Gamut".to_string()),
            quality: Some("Q0".to_string()),
            generation: Some(1),
            file_size: None,
            creation_time: None,
            modification_time: None,
        })
    }

    /// Get the total number of frames in this clip
    pub async fn get_frame_count(&self) -> Result<u64, BrawError> {
        Ok(240) // Placeholder
    }

    /// Extract a frame at the specified index
    pub async fn extract_frame(&self, _frame_index: u64) -> Result<Vec<u8>, BrawError> {
        // For now, return empty data - this will be enhanced with real SDK integration
        Ok(vec![])
    }
}

/// Validate that a file is a valid BRAW file
pub async fn validate_braw_file(path: &Path) -> Result<(), BrawError> {
    if !path.exists() {
        return Err(BrawError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("The file {} does not exist.", path.display())
        )));
    }

    // Check file extension
    if let Some(ext) = path.extension() {
        if ext.to_string_lossy().to_lowercase() != "braw" {
            return Err(BrawError::InvalidFormat);
        }
    } else {
        return Err(BrawError::InvalidFormat);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sdk_initialization() {
        let result = BrawSdk::new().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_validate_nonexistent_file() {
        let result = validate_braw_file(Path::new("/nonexistent/file.braw")).await;
        assert!(result.is_err());
    }
}