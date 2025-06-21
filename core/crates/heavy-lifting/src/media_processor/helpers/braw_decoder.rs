use std::ffi::CString;
use std::ptr;
use std::path::Path;
use std::collections::HashMap;

use image::DynamicImage;
use serde::{Deserialize, Serialize};

#[repr(C)]
struct Variant {
    vt: u32,
    data: VariantData,
}

#[repr(C)]
union VariantData {
    i_val: i16,
    ui_val: u16,
    int_val: i32,
    uint_val: u32,
    flt_val: f32,
    dbl_val: f64,
    bstr_val: *mut std::ffi::c_void,
}

const VARIANT_TYPE_EMPTY: u32 = 0;
const VARIANT_TYPE_S16: u32 = 2;
const VARIANT_TYPE_U16: u32 = 3;
const VARIANT_TYPE_S32: u32 = 4;
const VARIANT_TYPE_U32: u32 = 5;
const VARIANT_TYPE_FLOAT32: u32 = 6;
const VARIANT_TYPE_STRING: u32 = 7;
const VARIANT_TYPE_FLOAT64: u32 = 9;

extern "C" {
    fn create_blackmagic_raw_factory_instance_from_path(path: *const std::os::raw::c_char) -> *mut std::ffi::c_void;
    fn blackmagic_raw_factory_create_codec(factory_ptr: *mut std::ffi::c_void, codec_ptr: *mut *mut std::ffi::c_void) -> std::os::raw::c_long;
    fn blackmagic_raw_set_callback(codec_ptr: *mut std::ffi::c_void) -> std::os::raw::c_long;
    fn blackmagic_raw_open_clip(codec_ptr: *mut std::ffi::c_void, filename: *const std::os::raw::c_char, clip_ptr: *mut *mut std::ffi::c_void) -> std::os::raw::c_long;
    fn blackmagic_raw_clip_get_width(clip_ptr: *mut std::ffi::c_void, width: *mut u32) -> std::os::raw::c_long;
    fn blackmagic_raw_clip_get_height(clip_ptr: *mut std::ffi::c_void, height: *mut u32) -> std::os::raw::c_long;
    fn blackmagic_raw_clip_get_frame_rate(clip_ptr: *mut std::ffi::c_void, frame_rate: *mut f32) -> std::os::raw::c_long;
    fn blackmagic_raw_clip_get_frame_count(clip_ptr: *mut std::ffi::c_void, frame_count: *mut u64) -> std::os::raw::c_long;
    fn blackmagic_raw_clip_get_metadata_iterator(clip_ptr: *mut std::ffi::c_void, iterator_ptr: *mut *mut std::ffi::c_void) -> std::os::raw::c_long;
    fn blackmagic_raw_metadata_iterator_next(iterator_ptr: *mut std::ffi::c_void) -> std::os::raw::c_long;
    fn blackmagic_raw_metadata_iterator_get_key(iterator_ptr: *mut std::ffi::c_void, key_ptr: *mut *mut std::ffi::c_void) -> std::os::raw::c_long;
    fn blackmagic_raw_metadata_iterator_get_data(iterator_ptr: *mut std::ffi::c_void, data: *mut Variant) -> std::os::raw::c_long;
    fn blackmagic_raw_variant_init(variant: *mut Variant) -> std::os::raw::c_long;
    fn blackmagic_raw_variant_clear(variant: *mut Variant) -> std::os::raw::c_long;
    fn blackmagic_raw_clip_create_job_read_frame(clip_ptr: *mut std::ffi::c_void, frame_index: u64, job_ptr: *mut *mut std::ffi::c_void) -> std::os::raw::c_long;
    fn blackmagic_raw_job_submit(job_ptr: *mut std::ffi::c_void) -> std::os::raw::c_long;
    fn is_extraction_completed() -> bool;
    fn get_extraction_result() -> std::os::raw::c_long;
    fn reset_extraction_state();
    fn get_extracted_image_width() -> u32;
    fn get_extracted_image_height() -> u32;
    fn get_extracted_image_data() -> *mut std::ffi::c_void;
    fn blackmagic_raw_unknown_release(ptr: *mut std::ffi::c_void);
    fn buffer_release(ptr: *mut std::ffi::c_void);
    fn CFStringGetLength(theString: *mut std::ffi::c_void) -> isize;
    fn CFStringGetCString(theString: *mut std::ffi::c_void, buffer: *mut std::os::raw::c_char, bufferSize: isize, encoding: u32) -> bool;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BrawMetadata {
    pub width: u32,
    pub height: u32,
    pub frame_rate: f32,
    pub frame_count: u64,
    pub metadata: HashMap<String, String>,
}

pub struct BrawDecoder {
    factory: *mut std::ffi::c_void,
    codec: *mut std::ffi::c_void,
    clip: *mut std::ffi::c_void,
}

impl BrawDecoder {
    pub fn new(file_path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let sdk_path = CString::new("/Applications/Blackmagic RAW/Blackmagic RAW SDK/Mac/Libraries")?;
        let filename = CString::new(file_path.to_string_lossy().as_ref())?;

        unsafe {
            let factory = create_blackmagic_raw_factory_instance_from_path(sdk_path.as_ptr());
            if factory.is_null() {
                return Err("Failed to create BRAW factory".into());
            }

            let mut codec = ptr::null_mut();
            let hr = blackmagic_raw_factory_create_codec(factory, &mut codec);
            if hr != 0 || codec.is_null() {
                blackmagic_raw_unknown_release(factory);
                return Err("Failed to create codec".into());
            }

            let hr = blackmagic_raw_set_callback(codec);
            if hr != 0 {
                blackmagic_raw_unknown_release(codec);
                blackmagic_raw_unknown_release(factory);
                return Err("Failed to set callback".into());
            }

            let mut clip = ptr::null_mut();
            let hr = blackmagic_raw_open_clip(codec, filename.as_ptr(), &mut clip);
            if hr != 0 || clip.is_null() {
                blackmagic_raw_unknown_release(codec);
                blackmagic_raw_unknown_release(factory);
                return Err("Failed to open clip".into());
            }

            Ok(Self { factory, codec, clip })
        }
    }

    pub fn extract_metadata(&self) -> Result<BrawMetadata, Box<dyn std::error::Error>> {
        unsafe {
            let mut width = 0u32;
            let mut height = 0u32;
            let mut frame_rate = 0.0f32;
            let mut frame_count = 0u64;

            blackmagic_raw_clip_get_width(self.clip, &mut width);
            blackmagic_raw_clip_get_height(self.clip, &mut height);
            blackmagic_raw_clip_get_frame_rate(self.clip, &mut frame_rate);
            blackmagic_raw_clip_get_frame_count(self.clip, &mut frame_count);

            let mut metadata_map = HashMap::new();
            
            let mut metadata_iter = ptr::null_mut();
            let hr = blackmagic_raw_clip_get_metadata_iterator(self.clip, &mut metadata_iter);
            
            if hr == 0 && !metadata_iter.is_null() {
                loop {
                    let hr = blackmagic_raw_metadata_iterator_next(metadata_iter);
                    if hr == 1 { break; }
                    if hr != 0 { break; }

                    let mut key_ptr = ptr::null_mut();
                    if blackmagic_raw_metadata_iterator_get_key(metadata_iter, &mut key_ptr) == 0 && !key_ptr.is_null() {
                        let key_str = cfstring_to_string(key_ptr);
                        
                        let mut variant = Variant {
                            vt: VARIANT_TYPE_EMPTY,
                            data: VariantData { int_val: 0 },
                        };
                        blackmagic_raw_variant_init(&mut variant);
                        
                        if blackmagic_raw_metadata_iterator_get_data(metadata_iter, &mut variant) == 0 {
                            let value_str = variant_to_string(&variant);
                            if !key_str.is_empty() {
                                metadata_map.insert(key_str, value_str);
                            }
                        }
                        
                        blackmagic_raw_variant_clear(&mut variant);
                        buffer_release(key_ptr);
                    }
                }
                
                blackmagic_raw_unknown_release(metadata_iter);
            }

            Ok(BrawMetadata {
                width,
                height,
                frame_rate,
                frame_count,
                metadata: metadata_map,
            })
        }
    }

    pub fn extract_frame(&self, frame_index: u64) -> Result<DynamicImage, Box<dyn std::error::Error>> {
        unsafe {
            reset_extraction_state();

            let mut read_job = ptr::null_mut();
            let hr = blackmagic_raw_clip_create_job_read_frame(self.clip, frame_index, &mut read_job);
            if hr != 0 || read_job.is_null() {
                return Err("Failed to create read job".into());
            }

            let hr = blackmagic_raw_job_submit(read_job);
            blackmagic_raw_unknown_release(read_job);

            if hr != 0 {
                return Err("Failed to submit read job".into());
            }

            let mut timeout = 500;
            while !is_extraction_completed() && timeout > 0 {
                std::thread::sleep(std::time::Duration::from_millis(10));
                timeout -= 1;
            }

            if timeout == 0 {
                return Err("Timeout waiting for frame extraction".into());
            }

            let result = get_extraction_result();
            if result != 0 {
                return Err("Frame extraction failed".into());
            }

            let width = get_extracted_image_width();
            let height = get_extracted_image_height();
            let data_ptr = get_extracted_image_data();

            if width == 0 || height == 0 || data_ptr.is_null() {
                return Err("No image data extracted".into());
            }

            // BRAW SDK outputs RGBA data (4 bytes per pixel)
            let data_size = (width * height * 4) as usize;
            let data_slice = std::slice::from_raw_parts(data_ptr as *const u8, data_size);
            let data_vec = data_slice.to_vec();

            let rgba_image = image::RgbaImage::from_raw(width, height, data_vec)
                .ok_or("Failed to create RGBA image")?;

            Ok(DynamicImage::ImageRgba8(rgba_image))
        }
    }
}

impl Drop for BrawDecoder {
    fn drop(&mut self) {
        unsafe {
            if !self.clip.is_null() {
                blackmagic_raw_unknown_release(self.clip);
            }
            if !self.codec.is_null() {
                blackmagic_raw_unknown_release(self.codec);
            }
            if !self.factory.is_null() {
                blackmagic_raw_unknown_release(self.factory);
            }
        }
    }
}

unsafe fn cfstring_to_string(cf_string: *mut std::ffi::c_void) -> String {
    if cf_string.is_null() {
        return String::new();
    }
    
    let length = CFStringGetLength(cf_string);
    if length == 0 {
        return String::new();
    }
    
    let buffer_size = length * 4 + 1;
    let mut buffer = vec![0u8; buffer_size as usize];
    
    const K_CF_STRING_ENCODING_UTF8: u32 = 0x08000100;
    if CFStringGetCString(cf_string, buffer.as_mut_ptr() as *mut std::os::raw::c_char, buffer_size, K_CF_STRING_ENCODING_UTF8) {
        if let Some(null_pos) = buffer.iter().position(|&x| x == 0) {
            buffer.truncate(null_pos);
        }
        String::from_utf8_lossy(&buffer).into_owned()
    } else {
        String::new()
    }
}

unsafe fn variant_to_string(variant: &Variant) -> String {
    match variant.vt {
        VARIANT_TYPE_S16 => variant.data.i_val.to_string(),
        VARIANT_TYPE_U16 => variant.data.ui_val.to_string(),
        VARIANT_TYPE_S32 => variant.data.int_val.to_string(),
        VARIANT_TYPE_U32 => variant.data.uint_val.to_string(),
        VARIANT_TYPE_FLOAT32 => variant.data.flt_val.to_string(),
        VARIANT_TYPE_FLOAT64 => variant.data.dbl_val.to_string(),
        VARIANT_TYPE_STRING => cfstring_to_string(variant.data.bstr_val),
        VARIANT_TYPE_EMPTY => String::new(),
        _ => "Unknown type".to_string(),
    }
}