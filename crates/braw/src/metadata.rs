use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrawMetadata {
    // Technical metadata
    pub width: u32,
    pub height: u32,
    pub frame_rate: f64,
    pub duration_seconds: f64,
    pub total_frames: u32,
    pub codec: String,
    pub color_space: Option<String>,
    pub bit_depth: u8,
    pub pixel_format: Option<String>,
    
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
    pub recording_date: Option<DateTime<Utc>>,
    pub timecode: Option<String>,
    pub reel_name: Option<String>,
    pub scene: Option<String>,
    pub take: Option<String>,
    pub clip_name: Option<String>,
    
    // BRAW specific metadata
    pub compression_ratio: Option<String>,
    pub gamma: Option<String>,
    pub gamut: Option<String>,
    pub quality: Option<String>,
    pub generation: Option<u32>,
    
    // File metadata
    pub file_size: Option<u64>,
    pub creation_time: Option<DateTime<Utc>>,
    pub modification_time: Option<DateTime<Utc>>,
}

impl Default for BrawMetadata {
    fn default() -> Self {
        Self {
            width: 0,
            height: 0,
            frame_rate: 0.0,
            duration_seconds: 0.0,
            total_frames: 0,
            codec: "BRAW".to_string(),
            color_space: None,
            bit_depth: 16, // BRAW is typically 16-bit
            pixel_format: None,
            camera_model: None,
            lens_info: None,
            iso: None,
            shutter_speed: None,
            aperture: None,
            color_temperature: None,
            tint: None,
            focal_length: None,
            recording_date: None,
            timecode: None,
            reel_name: None,
            scene: None,
            take: None,
            clip_name: None,
            compression_ratio: None,
            gamma: None,
            gamut: None,
            quality: None,
            generation: None,
            file_size: None,
            creation_time: None,
            modification_time: None,
        }
    }
}

impl BrawMetadata {
    /// Get aspect ratio as a string (e.g., "16:9")
    pub fn aspect_ratio(&self) -> String {
        if self.width == 0 || self.height == 0 {
            return "unknown".to_string();
        }
        
        let gcd = gcd(self.width, self.height);
        let w = self.width / gcd;
        let h = self.height / gcd;
        
        format!("{}:{}", w, h)
    }
    
    /// Get resolution as a string (e.g., "4096x2160")
    pub fn resolution(&self) -> String {
        format!("{}x{}", self.width, self.height)
    }
    
    /// Check if this is a high resolution format (4K or higher)
    pub fn is_high_resolution(&self) -> bool {
        self.width >= 3840 || self.height >= 2160
    }
    
    /// Get frame rate as a string with proper formatting
    pub fn frame_rate_string(&self) -> String {
        if self.frame_rate.fract() == 0.0 {
            format!("{:.0} fps", self.frame_rate)
        } else {
            format!("{:.2} fps", self.frame_rate)
        }
    }
    
    /// Get duration as a formatted string (HH:MM:SS)
    pub fn duration_string(&self) -> String {
        let total_seconds = self.duration_seconds as u64;
        let hours = total_seconds / 3600;
        let minutes = (total_seconds % 3600) / 60;
        let seconds = total_seconds % 60;
        
        if hours > 0 {
            format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
        } else {
            format!("{:02}:{:02}", minutes, seconds)
        }
    }
}

// Helper function to calculate GCD for aspect ratio
fn gcd(a: u32, b: u32) -> u32 {
    if b == 0 {
        a
    } else {
        gcd(b, a % b)
    }
} 