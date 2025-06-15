use crate::{Error, Result};
use std::path::Path;

use sd_braw::BrawMetadata;
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Serialize, Deserialize, Type)]
pub struct BrawMediaMetadata {
    pub width: u32,
    pub height: u32,
    pub frame_rate: f64,
    pub duration: Option<f64>,
    pub total_frames: u32,
    pub codec: String,
    pub color_space: Option<String>,
    pub bit_depth: u8,

    // Camera metadata
    pub camera_model: Option<String>,
    pub lens_info: Option<String>,
    pub iso: Option<u32>,
    pub shutter_speed: Option<String>,
    pub aperture: Option<f32>,
    pub color_temperature: Option<u32>,
    pub tint: Option<i32>,
    pub focal_length: Option<f32>,

    // Recording metadata
    pub recording_date: Option<chrono::DateTime<chrono::Utc>>,
    pub timecode: Option<String>,
    pub reel_name: Option<String>,
    pub scene: Option<String>,
    pub take: Option<String>,
    pub clip_name: Option<String>,

    // BRAW specific
    pub compression_ratio: Option<String>,
    pub gamma: Option<String>,
    pub gamut: Option<String>,
    pub quality: Option<String>,
    pub generation: Option<u32>,

    // File metadata
    pub file_size: Option<u64>,
}

impl From<BrawMetadata> for BrawMediaMetadata {
    fn from(metadata: BrawMetadata) -> Self {
        Self {
            width: metadata.width,
            height: metadata.height,
            frame_rate: metadata.frame_rate,
            duration: Some(metadata.duration_seconds),
            total_frames: metadata.total_frames,
            codec: metadata.codec,
            color_space: metadata.color_space,
            bit_depth: metadata.bit_depth,
            camera_model: metadata.camera_model,
            lens_info: metadata.lens_info,
            iso: metadata.iso,
            shutter_speed: metadata.shutter_speed,
            aperture: metadata.aperture,
            color_temperature: metadata.color_temperature,
            tint: metadata.tint,
            focal_length: metadata.focal_length,
            recording_date: metadata.recording_date,
            timecode: metadata.timecode,
            reel_name: metadata.reel_name,
            scene: metadata.scene,
            take: metadata.take,
            clip_name: metadata.clip_name,
            compression_ratio: metadata.compression_ratio,
            gamma: metadata.gamma,
            gamut: metadata.gamut,
            quality: metadata.quality,
            generation: metadata.generation,
            file_size: metadata.file_size,
        }
    }
}

impl BrawMediaMetadata {
    /// Extract BRAW metadata from a file path
    pub async fn from_path(path: impl AsRef<Path> + Send) -> Result<Self> {
        #[cfg(not(feature = "braw"))]
        {
            let _ = path;
            Err(Error::NoBraw)
        }

        #[cfg(feature = "braw")]
        {
            use sd_braw::BrawFile;

            let braw_file = BrawFile::open(path.as_ref()).await
                .map_err(|e| Error::BrawError(e.to_string()))?;

            let metadata = braw_file.get_metadata().await
                .map_err(|e| Error::BrawError(e.to_string()))?;

            Ok(metadata.into())
        }
    }

    /// Get aspect ratio as a string
    pub fn aspect_ratio(&self) -> String {
        if self.width == 0 || self.height == 0 {
            return "unknown".to_string();
        }

        let gcd = gcd(self.width, self.height);
        let w = self.width / gcd;
        let h = self.height / gcd;

        format!("{}:{}", w, h)
    }

    /// Get resolution as a string
    pub fn resolution(&self) -> String {
        format!("{}x{}", self.width, self.height)
    }

    /// Check if this is a high resolution format
    pub fn is_high_resolution(&self) -> bool {
        self.width >= 3840 || self.height >= 2160
    }
}

// Helper function to calculate GCD
fn gcd(a: u32, b: u32) -> u32 {
    if b == 0 {
        a
    } else {
        gcd(b, a % b)
    }
}