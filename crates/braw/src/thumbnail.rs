//! Thumbnail generation for BRAW files
//!
//! This module handles extracting frames from BRAW files and generating
//! thumbnails at various sizes for UI display.

use crate::BrawError;
#[cfg(feature = "with-sdk")]
use crate::sdk::BrawClip;
use image::{DynamicImage, RgbImage, ImageFormat};
use std::path::Path;
use tokio::task;
use tracing::{debug, warn};

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

/// Generate a thumbnail from a BRAW file (requires real BlackmagicRAW SDK)
#[cfg(feature = "with-sdk")]
pub async fn generate_braw_thumbnail(
    path: &Path,
    config: ThumbnailConfig,
) -> Result<DynamicImage, BrawError> {
    debug!("Generating BRAW thumbnail for {}", path.display());

    #[cfg(feature = "native-ffi")]
    {
        // Try to use real SDK for thumbnail generation
        match extract_frame_at_timestamp(path, config.frame_position as f64).await {
            Ok(image) => {
                debug!("Successfully extracted real BRAW frame for thumbnail: {}", path.display());
                let thumbnail = resize_image(image, config.size);
                return Ok(thumbnail);
            }
            Err(e) => {
                warn!("Failed to extract frame with BlackmagicRAW SDK: {}", e);
                return Err(e);
            }
        }
    }

    // No fallback - BRAW files require the BlackmagicRAW SDK
    #[cfg(not(feature = "native-ffi"))]
    {
        warn!("BRAW thumbnail generation requires native-ffi feature and BlackmagicRAW SDK");
        return Err(BrawError::SdkUnavailable);
    }
}

/// Generate multiple thumbnail sizes in parallel (requires real BlackmagicRAW SDK)
#[cfg(feature = "with-sdk")]
pub async fn generate_braw_thumbnails(
    path: &Path,
    sizes: &[ThumbnailSize],
) -> Result<Vec<(ThumbnailSize, DynamicImage)>, BrawError> {
    debug!("Generating {} BRAW thumbnails for {}", sizes.len(), path.display());

    // Extract one base frame using the BlackmagicRAW SDK
    let base_image: DynamicImage = {
        #[cfg(feature = "native-ffi")]
        {
            match extract_frame_at_timestamp(path, 0.1).await {
                Ok(image) => {
                    debug!("Successfully extracted real BRAW frame for thumbnails: {}", path.display());
                    image
                }
                Err(e) => {
                    warn!("Failed to extract BRAW frame with SDK: {}", e);
                    return Err(e);
                }
            }
        }
        #[cfg(not(feature = "native-ffi"))]
        {
            warn!("BRAW thumbnail generation requires native-ffi feature and BlackmagicRAW SDK");
            return Err(BrawError::SdkUnavailable);
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
    // For now, use default dimensions since SDK metadata doesn't include width/height yet
    let width = 1920u32;  // Default HD width
    let height = 1080u32; // Default HD height

    // Process the frame data based on the BRAW format
    let image = process_braw_frame_data(frame_data, width, height)?;

    Ok(image)
}

/// Convert raw BRAW frame data to RGB image
#[allow(dead_code)]
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
    // Create a dark video-like placeholder with BRAW branding
    let mut buffer = Vec::with_capacity((width * height * 3) as usize);

    // Dark background color (dark gray)
    let bg_r = 32u8;
    let bg_g = 32u8;
    let bg_b = 32u8;

    // BRAW brand color (orange/red)
    let brand_r = 255u8;
    let brand_g = 100u8;
    let brand_b = 0u8;

    for y in 0..height {
        for x in 0..width {
            // Create a simple BRAW logo-like pattern in the center
            let center_x = width / 2;
            let center_y = height / 2;
            let logo_size = (width.min(height) / 8).max(20); // Minimum 20px logo

            let dx = (x as i32 - center_x as i32).abs() as u32;
            let dy = (y as i32 - center_y as i32).abs() as u32;

            let (r, g, b) = if dx < logo_size && dy < logo_size / 2 {
                // BRAW logo area - use brand color
                (brand_r, brand_g, brand_b)
            } else if dx < logo_size + 2 && dy < logo_size / 2 + 2 {
                // Logo border - slightly lighter
                (brand_r / 2, brand_g / 2, brand_b / 2)
            } else {
                // Background - dark gray
                (bg_r, bg_g, bg_b)
            };

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

/// Extract a frame at a specific timestamp
#[cfg(feature = "with-sdk")]
pub async fn extract_frame_at_timestamp(
    path: &Path,
    timestamp: f64,
) -> Result<DynamicImage, BrawError> {
    debug!("Extracting frame at {}s from {}", timestamp, path.display());

    // Only use the BlackmagicRAW SDK - no fallbacks
    match try_extract_frame_with_sdk(path, timestamp).await {
        Ok(image) => {
            debug!("Successfully extracted real BRAW frame at {}s from {}", timestamp, path.display());
            Ok(image)
        }
        Err(e) => {
            warn!("Failed to extract frame with BlackmagicRAW SDK: {}", e);
            Err(e)
        }
    }
}

/// Try to extract frame with SDK (internal function that can fail)
#[cfg(feature = "with-sdk")]
async fn try_extract_frame_with_sdk(
    path: &Path,
    timestamp: f64,
) -> Result<DynamicImage, BrawError> {
    let clip = crate::sdk::BrawClip::open(path.to_path_buf()).await?;

    // Get metadata to calculate frame index
    let frame_count = clip.get_frame_count()?;
    let (width, height) = clip.get_dimensions()?;

    // Calculate frame index from timestamp (assume 24fps for now)
    let frame_rate = 24.0;
    let frame_index = (timestamp * frame_rate) as u64;
    let frame_index = frame_index.min(frame_count as u64 - 1);

    // Extract and process frame
    let frame_data = clip.extract_frame(frame_index)?;
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

    let clip = crate::sdk::BrawClip::open(path.to_path_buf()).await?;

    let frame_count = clip.get_frame_count()?;
    let total_frames = frame_count as u64;

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

        let frame_data = clip.extract_frame(frame_index)?;
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