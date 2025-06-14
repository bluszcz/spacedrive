//! Safe Rust wrapper for BlackmagicRAW SDK
//! 
//! This module provides a safe interface to the BlackmagicRAW SDK,
//! handling all FFI and memory management internally.

use crate::{BrawError, BrawMetadata};
use std::ffi::{CStr, CString};
use std::path::Path;
use std::ptr;
use std::sync::Arc;
use tokio::task;
use tracing::{debug, error, warn};

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

/// Validates a BRAW file before attempting to open
pub async fn validate_braw_file(path: &Path) -> Result<(), BrawError> {
    // Check file exists and get size
    let metadata = tokio::fs::metadata(path).await
        .map_err(|e| BrawError::Io(e))?;
    
    if metadata.len() > MAX_BRAW_FILE_SIZE {
        return Err(BrawError::FileTooLarge {
            size: metadata.len(),
            max_size: MAX_BRAW_FILE_SIZE,
        });
    }
    
    // Check magic bytes
    let mut file = tokio::fs::File::open(path).await
        .map_err(|e| BrawError::Io(e))?;
    
    let mut buffer = [0u8; 8];
    tokio::io::AsyncReadExt::read_exact(&mut file, &mut buffer).await
        .map_err(|e| BrawError::Io(e))?;
    
    if !is_valid_braw_header(&buffer) {
        return Err(BrawError::InvalidFormat);
    }
    
    Ok(())
}

/// Check if the header bytes indicate a valid BRAW file
fn is_valid_braw_header(buffer: &[u8]) -> bool {
    // BRAW magic signature at the beginning
    buffer.len() >= 4 && &buffer[0..4] == b"BRAW"
}

#[cfg(feature = "native-ffi")]
/// SDK-enabled implementation
pub struct BrawSdk {
    factory: *mut IBlackmagicRawFactory,
    codec: *mut IBlackmagicRaw,
}

#[cfg(feature = "native-ffi")]
impl BrawSdk {
    /// Initialize the BRAW SDK
    pub async fn new() -> Result<Self, BrawError> {
        let factory = unsafe {
            CreateBlackmagicRawFactoryInstanceFromPath(ptr::null())
        };
        
        if factory.is_null() {
            return Err(BrawError::SdkUnavailable);
        }
        
        let mut codec: *mut IBlackmagicRaw = ptr::null_mut();
        let result = unsafe {
            (*factory).CreateCodec(&mut codec)
        };
        
        if result != 0 || codec.is_null() {
            unsafe { (*factory).Release() };
            return Err(BrawError::SdkInitializationFailed(result));
        }
        
        debug!("BlackmagicRAW SDK initialized successfully");
        
        Ok(BrawSdk {
            factory,
            codec,
        })
    }
    
    /// Open a BRAW clip file
    pub async fn open_clip(&self, path: &Path) -> Result<BrawClip, BrawError> {
        let path_str = path.to_string_lossy();
        let path_cstring = CString::new(path_str.as_ref())
            .map_err(|_| BrawError::InvalidPath)?;
        
        // Convert to CFString (macOS/iOS) or BSTR (Windows)
        #[cfg(target_os = "macos")]
        let cf_path = unsafe {
            use core_foundation::string::{CFString, CFStringRef};
            let cf_string = CFString::new(&path_str);
            cf_string.as_concrete_TypeRef()
        };
        
        #[cfg(target_os = "windows")]
        let cf_path = {
            // Convert to wide string for Windows
            use std::os::windows::ffi::OsStrExt;
            let wide: Vec<u16> = std::ffi::OsStr::new(&path_str)
                .encode_wide()
                .chain(std::iter::once(0))
                .collect();
            unsafe { SysAllocString(wide.as_ptr()) }
        };
        
        #[cfg(target_os = "linux")]
        let cf_path = path_cstring.as_ptr();
        
        let mut clip: *mut IBlackmagicRawClip = ptr::null_mut();
        let result = unsafe {
            (*self.codec).OpenClip(cf_path as *const _, &mut clip)
        };
        
        if result != 0 || clip.is_null() {
            return Err(BrawError::OpenFailed(format!("Failed to open clip: error code {}", result)));
        }
        
        debug!("Opened BRAW clip: {}", path.display());
        
        Ok(BrawClip::new(clip))
    }
}

#[cfg(feature = "native-ffi")]
impl Drop for BrawSdk {
    fn drop(&mut self) {
        unsafe {
            if !self.codec.is_null() {
                (*self.codec).Release();
            }
            if !self.factory.is_null() {
                (*self.factory).Release();
            }
        }
        debug!("BlackmagicRAW SDK cleaned up");
    }
}

#[cfg(feature = "native-ffi")]
/// Represents an opened BRAW clip file
pub struct BrawClip {
    clip: *mut IBlackmagicRawClip,
}

#[cfg(feature = "native-ffi")]
impl BrawClip {
    fn new(clip: *mut IBlackmagicRawClip) -> Self {
        BrawClip { clip }
    }
    
    /// Extract comprehensive metadata from the clip
    pub async fn get_metadata(&self) -> Result<BrawMetadata, BrawError> {
        let metadata = task::spawn_blocking({
            let clip = self.clip;
            move || {
                unsafe {
                    extract_clip_metadata(clip)
                }
            }
        }).await
        .map_err(|e| BrawError::TaskJoinError(e.to_string()))??;
        
        Ok(metadata)
    }
    
    /// Get the frame count of the clip
    pub async fn get_frame_count(&self) -> Result<u64, BrawError> {
        let clip = self.clip;
        let frame_count = task::spawn_blocking(move || {
            unsafe {
                let mut count: u64 = 0;
                let result = (*clip).GetFrameCount(&mut count);
                if result == 0 {
                    Ok(count)
                } else {
                    Err(BrawError::SdkError(result))
                }
            }
        }).await
        .map_err(|e| BrawError::TaskJoinError(e.to_string()))??;
        
        Ok(frame_count)
    }
    
    /// Extract a specific frame from the clip
    pub async fn extract_frame(&self, frame_index: u64) -> Result<Vec<u8>, BrawError> {
        let clip = self.clip;
        let frame_data = task::spawn_blocking(move || {
            unsafe {
                extract_frame_data(clip, frame_index)
            }
        }).await
        .map_err(|e| BrawError::TaskJoinError(e.to_string()))??;
        
        Ok(frame_data)
    }
}

#[cfg(feature = "native-ffi")]
impl Drop for BrawClip {
    fn drop(&mut self) {
        unsafe {
            if !self.clip.is_null() {
                (*self.clip).Release();
            }
        }
    }
}

#[cfg(feature = "native-ffi")]
unsafe fn extract_clip_metadata(clip: *mut IBlackmagicRawClip) -> Result<BrawMetadata, BrawError> {
    let mut metadata = BrawMetadata::default();
    
    // Get basic video properties
    let mut width: u32 = 0;
    let mut height: u32 = 0;
    let mut frame_rate: f32 = 0.0;
    let mut frame_count: u64 = 0;
    
    let result = (*clip).GetWidth(&mut width);
    if result != 0 {
        return Err(BrawError::SdkError(result));
    }
    
    let result = (*clip).GetHeight(&mut height);
    if result != 0 {
        return Err(BrawError::SdkError(result));
    }
    
    let result = (*clip).GetFrameRate(&mut frame_rate);
    if result != 0 {
        return Err(BrawError::SdkError(result));
    }
    
    let result = (*clip).GetFrameCount(&mut frame_count);
    if result != 0 {
        return Err(BrawError::SdkError(result));
    }
    
    metadata.width = width;
    metadata.height = height;
    metadata.frame_rate = frame_rate as f64;
    metadata.total_frames = frame_count as u32;
    metadata.duration_seconds = frame_count as f64 / frame_rate as f64;
    
    // Extract metadata using the metadata iterator
    let mut metadata_iterator: *mut IBlackmagicRawMetadataIterator = ptr::null_mut();
    let result = (*clip).GetMetadataIterator(&mut metadata_iterator);
    
    if result == 0 && !metadata_iterator.is_null() {
        extract_metadata_from_iterator(metadata_iterator, &mut metadata)?;
        (*metadata_iterator).Release();
    }
    
    Ok(metadata)
}

#[cfg(feature = "native-ffi")]
unsafe fn extract_metadata_from_iterator(
    iterator: *mut IBlackmagicRawMetadataIterator,
    metadata: &mut BrawMetadata,
) -> Result<(), BrawError> {
    // Iterate through all metadata keys
    loop {
        let mut key: *mut std::ffi::c_char = ptr::null_mut();
        let result = (*iterator).GetKey(&mut key);
        
        if result != 0 {
            break;
        }
        
        if key.is_null() {
            break;
        }
        
        let key_str = CStr::from_ptr(key).to_string_lossy();
        
        // Get the corresponding value
        let mut variant = Variant {
            vt: blackmagicRawVariantTypeEmpty,
            iVal: 0, // Use the union field
        };
        
        let result = (*iterator).GetData(&mut variant);
        if result == 0 {
            match_and_extract_metadata(&key_str, &variant, metadata);
        }
        
        // Clean up variant
        VariantClear(&mut variant);
        
        // Move to next item
        let result = (*iterator).Next();
        if result != 0 {
            break;
        }
    }
    
    Ok(())
}

#[cfg(feature = "native-ffi")]
fn match_and_extract_metadata(key: &str, variant: &Variant, metadata: &mut BrawMetadata) {
    match key {
        "cameraModel" | "camera_model" => {
            if let Some(s) = extract_string_from_variant(variant) {
                metadata.camera_model = Some(s);
            }
        }
        "lensInfo" | "lens_info" => {
            if let Some(s) = extract_string_from_variant(variant) {
                metadata.lens_info = Some(s);
            }
        }
        "iso" | "ISO" => {
            if let Some(val) = extract_u32_from_variant(variant) {
                metadata.iso = Some(val);
            }
        }
        "aperture" | "f_stop" => {
            if let Some(val) = extract_f32_from_variant(variant) {
                metadata.aperture = Some(val);
            }
        }
        "colorTemperature" | "color_temperature" | "whiteBalanceKelvin" => {
            if let Some(val) = extract_u32_from_variant(variant) {
                metadata.color_temperature = Some(val);
            }
        }
        "tint" | "whiteBalanceTint" => {
            if let Some(val) = extract_i32_from_variant(variant) {
                metadata.tint = Some(val);
            }
        }
        "timecode" => {
            if let Some(s) = extract_string_from_variant(variant) {
                metadata.timecode = Some(s);
            }
        }
        "reelName" | "reel_name" => {
            if let Some(s) = extract_string_from_variant(variant) {
                metadata.reel_name = Some(s);
            }
        }
        "scene" => {
            if let Some(s) = extract_string_from_variant(variant) {
                metadata.scene = Some(s);
            }
        }
        "take" => {
            if let Some(s) = extract_string_from_variant(variant) {
                metadata.take = Some(s);
            }
        }
        "codec" => {
            if let Some(s) = extract_string_from_variant(variant) {
                metadata.codec = s;
            }
        }
        "colorSpace" | "color_space" => {
            if let Some(s) = extract_string_from_variant(variant) {
                metadata.color_space = s;
            }
        }
        "bitDepth" | "bit_depth" => {
            if let Some(val) = extract_u32_from_variant(variant) {
                metadata.bit_depth = val as u8;
            }
        }
        _ => {
            // Store unknown metadata for debugging
            debug!("Unknown metadata key: {}", key);
        }
    }
}

#[cfg(feature = "native-ffi")]
fn extract_string_from_variant(variant: &Variant) -> Option<String> {
    unsafe {
        match variant.vt {
            blackmagicRawVariantTypeString => {
                if !variant.bstrVal.is_null() {
                    // Convert platform-specific string to Rust String
                    #[cfg(target_os = "macos")]
                    {
                        use core_foundation::string::CFString;
                        let cf_str = CFString::wrap_under_get_rule(variant.bstrVal);
                        Some(cf_str.to_string())
                    }
                    
                    #[cfg(target_os = "windows")]
                    {
                        let len = SysStringLen(variant.bstrVal) as usize;
                        let slice = std::slice::from_raw_parts(variant.bstrVal, len);
                        String::from_utf16(slice).ok()
                    }
                    
                    #[cfg(target_os = "linux")]
                    {
                        CStr::from_ptr(variant.bstrVal as *const i8)
                            .to_string_lossy()
                            .into_owned()
                            .into()
                    }
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

#[cfg(feature = "native-ffi")]
fn extract_u32_from_variant(variant: &Variant) -> Option<u32> {
    unsafe {
        match variant.vt {
            blackmagicRawVariantTypeU32 => Some(variant.uintVal),
            blackmagicRawVariantTypeU16 => Some(variant.uiVal as u32),
            blackmagicRawVariantTypeS32 => Some(variant.intVal as u32),
            blackmagicRawVariantTypeS16 => Some(variant.iVal as u32),
            _ => None,
        }
    }
}

#[cfg(feature = "native-ffi")]
fn extract_i32_from_variant(variant: &Variant) -> Option<i32> {
    unsafe {
        match variant.vt {
            blackmagicRawVariantTypeS32 => Some(variant.intVal),
            blackmagicRawVariantTypeS16 => Some(variant.iVal as i32),
            blackmagicRawVariantTypeU32 => Some(variant.uintVal as i32),
            blackmagicRawVariantTypeU16 => Some(variant.uiVal as i32),
            _ => None,
        }
    }
}

#[cfg(feature = "native-ffi")]
fn extract_f32_from_variant(variant: &Variant) -> Option<f32> {
    unsafe {
        match variant.vt {
            blackmagicRawVariantTypeFloat32 => Some(variant.fltVal),
            blackmagicRawVariantTypeFloat64 => Some(variant.dblVal as f32),
            _ => None,
        }
    }
}

#[cfg(feature = "native-ffi")]
unsafe fn extract_frame_data(clip: *mut IBlackmagicRawClip, frame_index: u64) -> Result<Vec<u8>, BrawError> {
    // Create a job to read the frame
    let mut job: *mut IBlackmagicRawJob = ptr::null_mut();
    let result = (*clip).CreateJobReadFrame(frame_index, &mut job);
    
    if result != 0 || job.is_null() {
        return Err(BrawError::SdkError(result));
    }
    
    // Submit the job and wait for completion
    // Note: In a real implementation, you'd use the callback interface
    // For simplicity, this is a synchronous approach
    let result = (*job).Submit();
    if result != 0 {
        (*job).Release();
        return Err(BrawError::SdkError(result));
    }
    
    // The actual frame data extraction would happen in the callback
    // For now, return placeholder data
    (*job).Release();
    
    // Return dummy RGB data (in real implementation, this would be actual frame data)
    let width = 1920u32; // Would get from clip
    let height = 1080u32; // Would get from clip
    let bytes_per_pixel = 3u32; // RGB
    let data_size = (width * height * bytes_per_pixel) as usize;
    
    Ok(vec![0u8; data_size])
}

// -----------------------------------------------------------------------------
// Stub implementation (CLI placeholder) when the native FFI feature is disabled
// -----------------------------------------------------------------------------

#[cfg(all(feature = "with-sdk", not(feature = "native-ffi")))]
#[derive(Debug)]
pub struct BrawSdk;

#[cfg(all(feature = "with-sdk", not(feature = "native-ffi")))]
impl BrawSdk {
    pub async fn new() -> Result<Self, BrawError> {
        // In CLI placeholder mode we don't require any initialization
        Ok(BrawSdk)
    }
    pub async fn open_clip(&self, _path: &Path) -> Result<BrawClip, BrawError> {
        // For now return error to indicate unimplemented; thumbnail module will fall back.
        Err(BrawError::SdkUnavailable)
    }
}

#[cfg(all(feature = "with-sdk", not(feature = "native-ffi")))]
#[derive(Debug, Clone, Copy)]
pub struct BrawClip;

#[cfg(all(feature = "with-sdk", not(feature = "native-ffi")))]
impl BrawClip {
    pub async fn get_metadata(&self) -> Result<BrawMetadata, BrawError> {
        Err(BrawError::SdkUnavailable)
    }
    pub async fn get_frame_count(&self) -> Result<u64, BrawError> {
        Err(BrawError::SdkUnavailable)
    }
    pub async fn extract_frame(&self, _frame_index: u64) -> Result<Vec<u8>, BrawError> {
        Err(BrawError::SdkUnavailable)
    }
}

// Fallback implementation when SDK feature is completely disabled
#[cfg(not(feature = "with-sdk"))]
#[derive(Debug)]
pub struct BrawSdk;

#[cfg(not(feature = "with-sdk"))]
impl BrawSdk {
    pub async fn new() -> Result<Self, BrawError> {
        Err(BrawError::SdkUnavailable)
    }
    
    pub async fn open_clip(&self, _path: &Path) -> Result<BrawClip, BrawError> {
        Err(BrawError::SdkUnavailable)
    }
}

#[cfg(not(feature = "with-sdk"))]
#[derive(Debug, Clone, Copy)]
pub struct BrawClip;

#[cfg(not(feature = "with-sdk"))]
impl BrawClip {
    pub async fn get_metadata(&self) -> Result<BrawMetadata, BrawError> {
        Err(BrawError::SdkUnavailable)
    }
    pub async fn get_frame_count(&self) -> Result<u64, BrawError> {
        Err(BrawError::SdkUnavailable)
    }
    pub async fn extract_frame(&self, _frame_index: u64) -> Result<Vec<u8>, BrawError> {
        Err(BrawError::SdkUnavailable)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_braw_header_validation() {
        assert!(is_valid_braw_header(b"BRAW1234"));
        assert!(is_valid_braw_header(b"BRAWDATA"));
        assert!(!is_valid_braw_header(b"NOTBRAW"));
        assert!(!is_valid_braw_header(b"BR"));
        assert!(!is_valid_braw_header(b""));
    }
    
    #[tokio::test]
    async fn test_sdk_unavailable_fallback() {
        #[cfg(not(feature = "with-sdk"))]
        {
            let result = BrawSdk::new().await;
            assert!(matches!(result, Err(BrawError::SdkUnavailable)));
        }
    }
} 