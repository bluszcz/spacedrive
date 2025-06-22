use std::ffi::CString;
use std::os::raw::{c_char, c_int};
use std::path::Path;

use image::{ImageBuffer, Rgba, RgbaImage};
use tokio::task;
use tracing::error;

// External C function declarations for ProRes RAW decoder wrapper
extern "C" {
    fn extract_prores_raw_frame(
        file_path: *const c_char,
        frame_number: c_int,
        width: *mut c_int,
        height: *mut c_int,
        data: *mut *mut u8,
        data_size: *mut usize,
    ) -> c_int;
    
    fn free_frame_data(data: *mut u8);
}

#[derive(Debug, thiserror::Error)]
pub enum ProResRawError {
    #[error("Failed to extract frame: {0}")]
    ExtractionFailed(String),
    #[error("Invalid file path")]
    InvalidPath,
    #[error("Memory allocation failed")]
    MemoryError,
    #[error("Image creation failed: {0}")]
    ImageError(String),
}

pub async fn extract_frame_from_prores_raw(
    path: impl AsRef<Path>,
    frame_number: u32,
) -> Result<RgbaImage, ProResRawError> {
    let path = path.as_ref();
    let path_str = path
        .to_str()
        .ok_or(ProResRawError::InvalidPath)?
        .to_string();

    // Run the blocking FFI call in a separate thread
    task::spawn_blocking(move || {
        let c_path = CString::new(path_str).map_err(|_| ProResRawError::InvalidPath)?;
        
        let mut width: c_int = 0;
        let mut height: c_int = 0;
        let mut data: *mut u8 = std::ptr::null_mut();
        let mut data_size: usize = 0;

        let result = unsafe {
            extract_prores_raw_frame(
                c_path.as_ptr(),
                frame_number as c_int,
                &mut width,
                &mut height,
                &mut data,
                &mut data_size,
            )
        };

        if result != 0 {
            return Err(ProResRawError::ExtractionFailed(format!(
                "C function returned error code: {}",
                result
            )));
        }

        if data.is_null() || width <= 0 || height <= 0 {
            return Err(ProResRawError::MemoryError);
        }

        // Convert C data to Rust Vec
        let data_vec = unsafe {
            let slice = std::slice::from_raw_parts(data, data_size);
            slice.to_vec()
        };

        // Free the C-allocated memory
        unsafe {
            free_frame_data(data);
        }

        // Ensure we have the right amount of data (RGBA = 4 bytes per pixel)
        let expected_size = (width as usize) * (height as usize) * 4;
        if data_size != expected_size {
            error!(
                "Data size mismatch: expected {}, got {}",
                expected_size, data_size
            );
            return Err(ProResRawError::ImageError(format!(
                "Data size mismatch: expected {}, got {}",
                expected_size, data_size
            )));
        }

        // Create RGBA image from raw data
        let rgba_image = ImageBuffer::<Rgba<u8>, _>::from_raw(width as u32, height as u32, data_vec)
            .ok_or_else(|| {
                ProResRawError::ImageError("Failed to create image from raw data".to_string())
            })?;

        Ok(rgba_image)
    })
    .await
    .map_err(|e| ProResRawError::ExtractionFailed(format!("Task join error: {}", e)))?
}

pub async fn extract_first_frame(path: impl AsRef<Path>) -> Result<RgbaImage, ProResRawError> {
    extract_frame_from_prores_raw(path, 0).await
}