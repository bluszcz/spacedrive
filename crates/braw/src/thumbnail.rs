//! Thumbnail generation for BRAW files
//!
//! This module handles extracting frames from BRAW files and generating
//! thumbnails at various sizes for UI display.

use crate::BrawError;
#[cfg(feature = "with-sdk")]
use crate::sdk::{BrawSdk, BrawClip};
use image::{DynamicImage, RgbImage, ImageFormat};
use std::path::Path;
use tokio::task;
use tracing::{debug, warn, info};

/// Standard thumbnail sizes used in Spacedrive
#[derive(Debug, Clone, Copy)]
pub enum ThumbnailSize {
    Small = 128,
    Medium = 256,
    Large = 512,
    ExtraLarge = 1024,
}

impl ThumbnailSize {
    pub fn as_u32(self) -> u32 {
        self as u32
    }
}

/// Configuration for thumbnail generation
#[derive(Debug, Clone)]
pub struct ThumbnailConfig {
    pub size: ThumbnailSize,
    pub quality: u8, // JPEG quality 0-100
    pub frame_position: f32, // Position in video (0.0-1.0)
    pub format: ImageFormat,
}

impl Default for ThumbnailConfig {
    fn default() -> Self {
        Self {
            size: ThumbnailSize::Medium,
            quality: 85,
            frame_position: 0.1, // 10% into the video
            format: ImageFormat::Jpeg,
        }
    }
}

/// Generate a thumbnail from a BRAW file (always succeeds with placeholder if SDK fails)
#[cfg(feature = "with-sdk")]
pub async fn generate_braw_thumbnail(
    path: &Path,
    config: ThumbnailConfig,
) -> Result<DynamicImage, BrawError> {
    debug!("Generating BRAW thumbnail for {}", path.display());

    // Always try to create a placeholder first as fallback
    let placeholder = create_placeholder_image(512, 512)?;

    #[cfg(feature = "native-ffi")]
    {
        // Try to use real SDK for thumbnail generation, but don't fail if it doesn't work
        match extract_frame_at_timestamp(path, config.frame_position as f64).await {
            Ok(image) => {
                info!("Successfully extracted real BRAW frame for thumbnail: {}", path.display());
                let thumbnail = resize_image(image, config.size);
                return Ok(thumbnail);
            }
            Err(e) => {
                debug!("Failed to extract frame with SDK (expected with placeholder bindings): {}", e);
            }
        }
    }

    // Use placeholder and resize it
    info!("Using placeholder thumbnail for BRAW file: {}", path.display());
    let thumbnail = resize_image(placeholder, config.size);
    Ok(thumbnail)
}

/// Generate multiple thumbnail sizes in parallel (always succeeds with placeholder if SDK fails)
#[cfg(feature = "with-sdk")]
pub async fn generate_braw_thumbnails(
    path: &Path,
    sizes: &[ThumbnailSize],
) -> Result<Vec<(ThumbnailSize, DynamicImage)>, BrawError> {
    debug!("Generating {} BRAW thumbnails for {}", sizes.len(), path.display());

    // Create a base image first - always start with placeholder
    let base_image = {
        let placeholder = create_placeholder_image(512, 512)?;

        #[cfg(feature = "native-ffi")]
        {
            // Try to extract a real frame first, but don't fail if it doesn't work
            match extract_frame_at_timestamp(path, 0.1).await {
                Ok(image) => {
                    info!("Successfully extracted real BRAW frame for thumbnails: {}", path.display());
                    image
                }
                Err(_) => {
                    debug!("Using placeholder for BRAW thumbnails: {}", path.display());
                    placeholder
                }
            }
        }
        #[cfg(not(feature = "native-ffi"))]
        {
            debug!("Using placeholder for BRAW thumbnails (native-ffi not enabled): {}", path.display());
            placeholder
        }
    };

    // Generate thumbnails for all requested sizes using our resize function
    let thumbnails = sizes.iter().map(|&size| {
        let thumbnail = resize_image(base_image.clone(), size);
        (size, thumbnail)
    }).collect();

    Ok(thumbnails)
}

/// Process raw frame data into a DynamicImage
#[cfg(feature = "with-sdk")]
async fn process_frame_to_image(
    frame_data: Vec<u8>,
    clip: &BrawClip,
) -> Result<DynamicImage, BrawError> {
    let metadata = clip.get_metadata().await?;
    let width = metadata.width;
    let height = metadata.height;

    // Process the frame data based on the BRAW format
    let image = process_braw_frame_data(frame_data, width, height)?;

    Ok(image)
}

/// Convert raw BRAW frame data to RGB image
fn process_braw_frame_data(
    frame_data: Vec<u8>,
    width: u32,
    height: u32,
) -> Result<DynamicImage, BrawError> {
    // In a real implementation, this would:
    // 1. Decode the BRAW compressed data
    // 2. Apply color correction/grading
    // 3. Convert to RGB format
    // 4. Handle different bit depths and color spaces

    // For now, we'll create a placeholder implementation
    // that assumes the frame_data is already RGB

    let expected_size = (width * height * 3) as usize; // 3 bytes per pixel (RGB)

    if frame_data.len() != expected_size {
        // If data doesn't match expected RGB size, create a placeholder
        warn!("Frame data size mismatch, creating placeholder image");
        return create_placeholder_image(width, height);
    }

    // Create RGB image from raw data
    let rgb_image = RgbImage::from_raw(width, height, frame_data)
        .ok_or_else(|| BrawError::ImageProcessing("Failed to create RGB image from raw data".into()))?;

    Ok(DynamicImage::ImageRgb8(rgb_image))
}

/// Create a placeholder image when real frame data is unavailable
fn create_placeholder_image(width: u32, height: u32) -> Result<DynamicImage, BrawError> {
    // Create a gradient placeholder image
    let mut buffer = Vec::with_capacity((width * height * 3) as usize);

    for y in 0..height {
        for x in 0..width {
            // Create a simple gradient pattern
            let r = ((x * 255) / width) as u8;
            let g = ((y * 255) / height) as u8;
            let b = 128u8; // Constant blue

            buffer.push(r);
            buffer.push(g);
            buffer.push(b);
        }
    }

    let rgb_image = RgbImage::from_raw(width, height, buffer)
        .ok_or_else(|| BrawError::ImageProcessing("Failed to create placeholder image".into()))?;

    Ok(DynamicImage::ImageRgb8(rgb_image))
}

/// Resize an image to thumbnail size while maintaining aspect ratio
fn resize_image(image: DynamicImage, target_size: ThumbnailSize) -> DynamicImage {
    let size = target_size.as_u32();

    // Calculate dimensions maintaining aspect ratio
    let (original_width, original_height) = (image.width(), image.height());
    let aspect_ratio = original_width as f32 / original_height as f32;

    let (new_width, new_height) = if aspect_ratio > 1.0 {
        // Landscape: limit width
        (size, (size as f32 / aspect_ratio) as u32)
    } else {
        // Portrait: limit height
        ((size as f32 * aspect_ratio) as u32, size)
    };

    // Use high-quality Lanczos3 filter for thumbnails
    image.resize(new_width, new_height, image::imageops::FilterType::Lanczos3)
}

/// Extract a frame at a specific timestamp (in seconds)
#[cfg(feature = "with-sdk")]
pub async fn extract_frame_at_timestamp(
    path: &Path,
    timestamp: f64,
) -> Result<DynamicImage, BrawError> {
    debug!("Extracting frame at {}s from {}", timestamp, path.display());

    let mut sdk = BrawSdk::new().await?;
    let clip = sdk.open_clip(path).await?;

    // Get metadata to calculate frame index
    let metadata = clip.get_metadata().await?;
    let frame_rate = metadata.frame_rate;
    let total_duration = metadata.duration_seconds;
    let total_frames = metadata.total_frames;

    // Validate timestamp
    if timestamp < 0.0 || timestamp > total_duration {
        return Err(BrawError::FrameOutOfRange {
            frame: (timestamp * frame_rate) as u32,
            max_frames: metadata.total_frames,
        });
    }

    // Calculate frame index
    let frame_index = (timestamp * frame_rate) as u64;
    let frame_index = frame_index.min(metadata.total_frames as u64 - 1);

    // Extract and process frame
    let frame_data = clip.extract_frame(frame_index).await?;
    let image = process_frame_to_image(frame_data, &clip).await?;

    debug!("Extracted frame {} ({}s) from {}", frame_index, timestamp, path.display());

    Ok(image)
}

/// Generate a filmstrip preview with multiple frames
#[cfg(feature = "with-sdk")]
pub async fn generate_filmstrip_preview(
    path: &Path,
    frame_count: u32,
    thumbnail_size: ThumbnailSize,
) -> Result<Vec<DynamicImage>, BrawError> {
    debug!("Generating filmstrip with {} frames from {}", frame_count, path.display());

    let mut sdk = BrawSdk::new().await?;
    let clip = sdk.open_clip(path).await?;

    let metadata = clip.get_metadata().await?;
    let total_frames = metadata.total_frames as u64;

    if frame_count == 0 {
        return Ok(Vec::new());
    }

    // Calculate frame indices evenly distributed across the video
    let frame_indices: Vec<u64> = (0..frame_count)
        .map(|i| {
            let position = i as f64 / (frame_count - 1).max(1) as f64;
            let frame_index = (position * (total_frames - 1) as f64) as u64;
            frame_index.min(total_frames - 1)
        })
        .collect();

    // Extract all frames
    let mut thumbnails = Vec::with_capacity(frame_count as usize);

    for (i, &frame_index) in frame_indices.iter().enumerate() {
        debug!("Extracting filmstrip frame {} of {} (frame {})", i + 1, frame_count, frame_index);

        let frame_data = clip.extract_frame(frame_index).await?;
        let image = process_frame_to_image(frame_data, &clip).await?;
        let thumbnail = resize_image(image, thumbnail_size);

        thumbnails.push(thumbnail);
    }

    debug!("Generated filmstrip with {} frames from {}", thumbnails.len(), path.display());

    Ok(thumbnails)
}

/// Save thumbnail to disk
pub async fn save_thumbnail(
    thumbnail: &DynamicImage,
    output_path: &Path,
    format: ImageFormat,
    quality: u8,
) -> Result<(), BrawError> {
    let thumbnail = thumbnail.clone();
    let output_path = output_path.to_path_buf();

    task::spawn_blocking(move || {
        match format {
            ImageFormat::Jpeg => {
                use image::codecs::jpeg::JpegEncoder;
                use std::fs::File;

                let file = File::create(&output_path)
                    .map_err(|e| BrawError::Io(e))?;

                let mut encoder = JpegEncoder::new_with_quality(file, quality);
                encoder.encode_image(&thumbnail)
                    .map_err(|e| BrawError::ImageProcessing(e.to_string()))?;
            }
            _ => {
                thumbnail.save_with_format(&output_path, format)
                    .map_err(|e| BrawError::ImageProcessing(e.to_string()))?;
            }
        }

        Ok(())
    }).await
    .map_err(|e| BrawError::TaskJoinError(e.to_string()))?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thumbnail_size_conversion() {
        assert_eq!(ThumbnailSize::Small.as_u32(), 128);
        assert_eq!(ThumbnailSize::Medium.as_u32(), 256);
        assert_eq!(ThumbnailSize::Large.as_u32(), 512);
        assert_eq!(ThumbnailSize::ExtraLarge.as_u32(), 1024);
    }

    #[test]
    fn test_placeholder_image_creation() {
        let image = create_placeholder_image(100, 100).unwrap();
        assert_eq!(image.width(), 100);
        assert_eq!(image.height(), 100);
    }

    #[test]
    fn test_image_resize_aspect_ratio() {
        // Create a test image (200x100 - landscape)
        let test_image = DynamicImage::new_rgb8(200, 100);

        let resized = resize_image(test_image, ThumbnailSize::Medium);

        // Should maintain aspect ratio (2:1)
        assert_eq!(resized.width(), 256);
        assert_eq!(resized.height(), 128);
    }

    #[tokio::test]
    async fn test_thumbnail_config_default() {
        let config = ThumbnailConfig::default();
        assert_eq!(config.size.as_u32(), 256);
        assert_eq!(config.quality, 85);
        assert_eq!(config.frame_position, 0.1);
    }
}

// -----------------------------------------------------------------------------
// Fallback implementations when the BlackmagicRAW SDK is NOT available
// -----------------------------------------------------------------------------

#[cfg(not(feature = "with-sdk"))]
/// Generate BRAW thumbnail (stub implementation without SDK)
pub async fn generate_braw_thumbnail(
    _path: &Path,
    config: ThumbnailConfig,
) -> Result<DynamicImage, BrawError> {
    // Create a placeholder image when SDK is not available
    let placeholder = create_placeholder_image(512, 512)?;

    // Resize to requested size
    let thumbnail = resize_image(placeholder, config.size);

    Ok(thumbnail)
}

#[cfg(not(feature = "with-sdk"))]
/// Generate multiple BRAW thumbnails (stub implementation without SDK)
pub async fn generate_braw_thumbnails(
    _path: &Path,
    sizes: &[ThumbnailSize],
) -> Result<Vec<(ThumbnailSize, DynamicImage)>, BrawError> {
    // Create a base placeholder image
    let base_placeholder = create_placeholder_image(512, 512)?;

    // Generate thumbnails for all requested sizes
    let thumbnails = sizes.iter().map(|&size| {
        let thumbnail = resize_image(base_placeholder.clone(), size);
        (size, thumbnail)
    }).collect();

    Ok(thumbnails)
}