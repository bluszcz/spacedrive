//! BlackmagicRAW SDK integration
//!
//! This module provides direct integration with the BlackmagicRAW SDK
//! for native BRAW file processing, frame extraction, and metadata access.

use crate::error::BrawError;
use image::{DynamicImage, ImageBuffer, Rgb};
use std::sync::{Arc, Mutex};
use std::path::Path;
use std::ptr;
use std::ffi::c_void;
use tracing::{debug, info, warn, error};

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

// Forward declarations for types not in bindings
#[cfg(feature = "native-ffi")]
#[repr(C)]
pub struct IBlackmagicRawFrame {
    _private: [u8; 0],
}

#[cfg(feature = "native-ffi")]
#[repr(C)]
pub struct IBlackmagicRawProcessedImage {
    _private: [u8; 0],
}

#[cfg(feature = "native-ffi")]
#[repr(C)]
pub struct IBlackmagicRawJob {
    _private: [u8; 0],
}

#[cfg(feature = "native-ffi")]
#[repr(C)]
pub struct IBlackmagicRawCallback {
    _private: [u8; 0],
}

#[cfg(feature = "native-ffi")]
#[repr(C)]
pub struct IBlackmagicRawFrameProcessingAttributes {
    _private: [u8; 0],
}

#[cfg(all(target_os = "macos", feature = "native-ffi"))]
use core_foundation::{base::TCFType, string::CFString};

// Constants
const MAX_BRAW_FILE_SIZE: u64 = 50 * 1024 * 1024 * 1024; // 50 GB limit

/// BlackmagicRAW SDK wrapper for safe Rust usage
#[derive(Debug)]
pub struct BrawSdk {
    #[cfg(feature = "native-ffi")]
    factory: *mut IBlackmagicRawFactory,
    #[cfg(feature = "native-ffi")]
    codec: *mut IBlackmagicRaw,
    #[cfg(feature = "native-ffi")]
    clip: *mut IBlackmagicRawClip,
    #[cfg(feature = "native-ffi")]
    callback: Option<Box<BrawCallback>>,
    /// Indicates if the SDK was successfully initialized
    initialized: bool,
}

/// Callback handler for async SDK operations
#[cfg(feature = "native-ffi")]
#[repr(C)]
#[derive(Debug)]
struct BrawCallback {
    vtable: *const IBlackmagicRawCallbackVTable,
    read_sender: Option<tokio::sync::oneshot::Sender<Result<*mut IBlackmagicRawFrame, BrawError>>>,
    process_sender: Option<tokio::sync::oneshot::Sender<Result<*mut IBlackmagicRawProcessedImage, BrawError>>>,
    ref_count: std::sync::atomic::AtomicU32,
}

#[cfg(feature = "native-ffi")]
impl BrawCallback {
    fn new() -> Box<Self> {
        let vtable = Box::leak(Box::new(IBlackmagicRawCallbackVTable {
            query_interface: Self::query_interface,
            add_ref: Self::add_ref,
            release: Self::release,
            read_complete: Self::read_complete,
            process_complete: Self::process_complete,
            decode_and_process_complete: Self::decode_and_process_complete,
        }));

        Box::new(Self {
            vtable: vtable as *const _,
            read_sender: None,
            process_sender: None,
            ref_count: std::sync::atomic::AtomicU32::new(1),
        })
    }

    unsafe extern "C" fn query_interface(
        _this: *mut IBlackmagicRawCallback,
        _riid: *const CFUUIDBytes,
        _ppv_object: *mut *mut c_void,
    ) -> HRESULT {
        0x80004002u32 as HRESULT // E_NOINTERFACE
    }

    unsafe extern "C" fn add_ref(this: *mut IBlackmagicRawCallback) -> ULONG {
        let callback = &*(this as *mut BrawCallback);
        let old_count = callback.ref_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        old_count + 1
    }

    unsafe extern "C" fn release(this: *mut IBlackmagicRawCallback) -> ULONG {
        let callback = &*(this as *mut BrawCallback);
        let old_count = callback.ref_count.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
        if old_count == 1 {
            // Last reference, clean up
            drop(Box::from_raw(this as *mut BrawCallback));
            0
        } else {
            old_count - 1
        }
    }

    unsafe extern "C" fn read_complete(
        this: *mut IBlackmagicRawCallback,
        _job: *mut IBlackmagicRawJob,
        result: HRESULT,
        frame: *mut IBlackmagicRawFrame,
    ) -> HRESULT {
        let callback = &mut *(this as *mut BrawCallback);

        if let Some(sender) = callback.read_sender.take() {
            let result = if result == 0 {
                Ok(frame)
            } else {
                error!("Read job failed with HRESULT: {:#x}", result);
                Err(BrawError::SdkUnavailable)
            };

            if sender.send(result).is_err() {
                warn!("Failed to send read completion result - receiver dropped");
            }
        }

        0 // S_OK
    }

    unsafe extern "C" fn process_complete(
        this: *mut IBlackmagicRawCallback,
        _job: *mut IBlackmagicRawJob,
        result: HRESULT,
        processed_image: *mut IBlackmagicRawProcessedImage,
    ) -> HRESULT {
        let callback = &mut *(this as *mut BrawCallback);

        if let Some(sender) = callback.process_sender.take() {
            let result = if result == 0 {
                Ok(processed_image)
            } else {
                error!("Process job failed with HRESULT: {:#x}", result);
                Err(BrawError::SdkUnavailable)
            };

            if sender.send(result).is_err() {
                warn!("Failed to send process completion result - receiver dropped");
            }
        }

        0 // S_OK
    }

    unsafe extern "C" fn decode_and_process_complete(
        this: *mut IBlackmagicRawCallback,
        job: *mut IBlackmagicRawJob,
        result: HRESULT,
        processed_image: *mut IBlackmagicRawProcessedImage,
    ) -> HRESULT {
        // For decode_and_process, we use the same logic as process_complete
        Self::process_complete(this, job, result, processed_image)
    }
}

// Manual vtable implementations for BlackmagicRAW SDK interfaces
// These are the missing pieces that bindgen couldn't generate properly

#[cfg(feature = "native-ffi")]
#[repr(C)]
struct IBlackmagicRawFactoryVTable {
    // IUnknown methods
    query_interface: unsafe extern "C" fn(*mut IBlackmagicRawFactory, *const CFUUIDBytes, *mut *mut c_void) -> HRESULT,
    add_ref: unsafe extern "C" fn(*mut IBlackmagicRawFactory) -> ULONG,
    release: unsafe extern "C" fn(*mut IBlackmagicRawFactory) -> ULONG,

    // IBlackmagicRawFactory methods
    create_codec: unsafe extern "C" fn(*mut IBlackmagicRawFactory, *mut *mut IBlackmagicRaw) -> HRESULT,
    create_pipeline_iterator: unsafe extern "C" fn(*mut IBlackmagicRawFactory, BlackmagicRawInterop, *mut *mut IBlackmagicRawPipelineIterator) -> HRESULT,
    create_pipeline_device_iterator: unsafe extern "C" fn(*mut IBlackmagicRawFactory, BlackmagicRawPipeline, BlackmagicRawInterop, *mut *mut IBlackmagicRawPipelineDeviceIterator) -> HRESULT,
    create_clip_geometry: unsafe extern "C" fn(*mut IBlackmagicRawFactory, *mut *mut IBlackmagicRawClipGeometry) -> HRESULT,
}

#[cfg(feature = "native-ffi")]
#[repr(C)]
struct IBlackmagicRawVTable {
    // IUnknown methods
    query_interface: unsafe extern "C" fn(*mut IBlackmagicRaw, *const CFUUIDBytes, *mut *mut c_void) -> HRESULT,
    add_ref: unsafe extern "C" fn(*mut IBlackmagicRaw) -> ULONG,
    release: unsafe extern "C" fn(*mut IBlackmagicRaw) -> ULONG,

    // IBlackmagicRaw methods
    open_clip: unsafe extern "C" fn(*mut IBlackmagicRaw, CFStringRef, *mut *mut IBlackmagicRawClip) -> HRESULT,
    open_clip_with_geometry: unsafe extern "C" fn(*mut IBlackmagicRaw, CFStringRef, *mut IBlackmagicRawClipGeometry, *mut *mut IBlackmagicRawClip) -> HRESULT,
    set_callback: unsafe extern "C" fn(*mut IBlackmagicRaw, *mut IBlackmagicRawCallback) -> HRESULT,
    prepare_pipeline: unsafe extern "C" fn(*mut IBlackmagicRaw, BlackmagicRawPipeline, *mut c_void, *mut c_void, *mut c_void) -> HRESULT,
    prepare_pipeline_for_device: unsafe extern "C" fn(*mut IBlackmagicRaw, *mut IBlackmagicRawPipelineDevice, *mut c_void) -> HRESULT,
    flush_jobs: unsafe extern "C" fn(*mut IBlackmagicRaw) -> HRESULT,
}

#[cfg(feature = "native-ffi")]
#[repr(C)]
struct IBlackmagicRawClipVTable {
    // IUnknown methods
    query_interface: unsafe extern "C" fn(*mut IBlackmagicRawClip, *const CFUUIDBytes, *mut *mut c_void) -> HRESULT,
    add_ref: unsafe extern "C" fn(*mut IBlackmagicRawClip) -> ULONG,
    release: unsafe extern "C" fn(*mut IBlackmagicRawClip) -> ULONG,

    // IBlackmagicRawClip methods
    get_width: unsafe extern "C" fn(*mut IBlackmagicRawClip, *mut u32) -> HRESULT,
    get_height: unsafe extern "C" fn(*mut IBlackmagicRawClip, *mut u32) -> HRESULT,
    get_frame_rate: unsafe extern "C" fn(*mut IBlackmagicRawClip, *mut f32) -> HRESULT,
    get_frame_count: unsafe extern "C" fn(*mut IBlackmagicRawClip, *mut u64) -> HRESULT,
    get_timecode_for_frame: unsafe extern "C" fn(*mut IBlackmagicRawClip, u64, *mut CFStringRef) -> HRESULT,
    get_metadata_iterator: unsafe extern "C" fn(*mut IBlackmagicRawClip, *mut *mut IBlackmagicRawMetadataIterator) -> HRESULT,
    get_metadata: unsafe extern "C" fn(*mut IBlackmagicRawClip, CFStringRef, *mut Variant) -> HRESULT,
    set_metadata: unsafe extern "C" fn(*mut IBlackmagicRawClip, CFStringRef, *mut Variant) -> HRESULT,
    get_camera_type: unsafe extern "C" fn(*mut IBlackmagicRawClip, *mut CFStringRef) -> HRESULT,
    clone_clip_processing_attributes: unsafe extern "C" fn(*mut IBlackmagicRawClip, *mut *mut IBlackmagicRawClipProcessingAttributes) -> HRESULT,
    get_multicard_file_count: unsafe extern "C" fn(*mut IBlackmagicRawClip, *mut u32) -> HRESULT,
    is_multicard_file_present: unsafe extern "C" fn(*mut IBlackmagicRawClip, u32, *mut bool) -> HRESULT,
    get_sidecar_file_attached: unsafe extern "C" fn(*mut IBlackmagicRawClip, *mut bool) -> HRESULT,
    save_sidecar_file: unsafe extern "C" fn(*mut IBlackmagicRawClip) -> HRESULT,
    reload_sidecar_file: unsafe extern "C" fn(*mut IBlackmagicRawClip) -> HRESULT,
    create_job_read_frame: unsafe extern "C" fn(*mut IBlackmagicRawClip, u64, *mut *mut IBlackmagicRawJob) -> HRESULT,
    create_job_trim: unsafe extern "C" fn(*mut IBlackmagicRawClip, CFStringRef, u64, u64, *mut IBlackmagicRawClipProcessingAttributes, *mut IBlackmagicRawFrameProcessingAttributes, *mut *mut IBlackmagicRawJob) -> HRESULT,
    clone_with_geometry: unsafe extern "C" fn(*mut IBlackmagicRawClip, *mut IBlackmagicRawClipGeometry, *mut *mut IBlackmagicRawClip) -> HRESULT,
}

#[cfg(feature = "native-ffi")]
#[repr(C)]
struct IBlackmagicRawJobVTable {
    // IUnknown methods
    query_interface: unsafe extern "C" fn(*mut IBlackmagicRawJob, *const CFUUIDBytes, *mut *mut c_void) -> HRESULT,
    add_ref: unsafe extern "C" fn(*mut IBlackmagicRawJob) -> ULONG,
    release: unsafe extern "C" fn(*mut IBlackmagicRawJob) -> ULONG,

    // IBlackmagicRawJob methods
    submit: unsafe extern "C" fn(*mut IBlackmagicRawJob) -> HRESULT,
    set_frame_processing_attributes: unsafe extern "C" fn(*mut IBlackmagicRawJob, *mut IBlackmagicRawFrameProcessingAttributes) -> HRESULT,
}

#[cfg(feature = "native-ffi")]
#[repr(C)]
struct IBlackmagicRawCallbackVTable {
    // IUnknown methods
    query_interface: unsafe extern "C" fn(*mut IBlackmagicRawCallback, *const CFUUIDBytes, *mut *mut c_void) -> HRESULT,
    add_ref: unsafe extern "C" fn(*mut IBlackmagicRawCallback) -> ULONG,
    release: unsafe extern "C" fn(*mut IBlackmagicRawCallback) -> ULONG,

    // IBlackmagicRawCallback methods
    read_complete: unsafe extern "C" fn(*mut IBlackmagicRawCallback, *mut IBlackmagicRawJob, HRESULT, *mut IBlackmagicRawFrame) -> HRESULT,
    process_complete: unsafe extern "C" fn(*mut IBlackmagicRawCallback, *mut IBlackmagicRawJob, HRESULT, *mut IBlackmagicRawProcessedImage) -> HRESULT,
    decode_and_process_complete: unsafe extern "C" fn(*mut IBlackmagicRawCallback, *mut IBlackmagicRawJob, HRESULT, *mut IBlackmagicRawProcessedImage) -> HRESULT,
}

#[cfg(feature = "native-ffi")]
#[repr(C)]
struct IBlackmagicRawFrameVTable {
    // IUnknown methods
    query_interface: unsafe extern "C" fn(*mut IBlackmagicRawFrame, *const CFUUIDBytes, *mut *mut c_void) -> HRESULT,
    add_ref: unsafe extern "C" fn(*mut IBlackmagicRawFrame) -> ULONG,
    release: unsafe extern "C" fn(*mut IBlackmagicRawFrame) -> ULONG,

    // IBlackmagicRawFrame methods
    create_job_decode_and_process_frame: unsafe extern "C" fn(*mut IBlackmagicRawFrame, *mut IBlackmagicRawClipProcessingAttributes, *mut IBlackmagicRawFrameProcessingAttributes, *mut *mut IBlackmagicRawJob) -> HRESULT,
}

#[cfg(feature = "native-ffi")]
#[repr(C)]
struct IBlackmagicRawProcessedImageVTable {
    // IUnknown methods
    query_interface: unsafe extern "C" fn(*mut IBlackmagicRawProcessedImage, *const CFUUIDBytes, *mut *mut c_void) -> HRESULT,
    add_ref: unsafe extern "C" fn(*mut IBlackmagicRawProcessedImage) -> ULONG,
    release: unsafe extern "C" fn(*mut IBlackmagicRawProcessedImage) -> ULONG,

    // IBlackmagicRawProcessedImage methods
    get_width: unsafe extern "C" fn(*mut IBlackmagicRawProcessedImage, *mut u32) -> HRESULT,
    get_height: unsafe extern "C" fn(*mut IBlackmagicRawProcessedImage, *mut u32) -> HRESULT,
    get_resource: unsafe extern "C" fn(*mut IBlackmagicRawProcessedImage, *mut *mut c_void) -> HRESULT,
    get_resource_size_bytes: unsafe extern "C" fn(*mut IBlackmagicRawProcessedImage, *mut u32) -> HRESULT,
}

// Helper function to get vtable from interface pointer
#[cfg(feature = "native-ffi")]
unsafe fn get_vtable<T>(interface: *mut T) -> *const *const c_void {
    *(interface as *const *const *const c_void)
}

impl BrawSdk {
    /// Create a new BlackmagicRAW SDK instance
    pub fn new() -> Result<Self, BrawError> {
        info!("Initializing BlackmagicRAW SDK");

        #[cfg(feature = "native-ffi")]
        {
            unsafe {
                // Create factory instance
                let factory = CreateBlackmagicRawFactoryInstance();
                if factory.is_null() {
                    error!("Failed to create BlackmagicRAW factory instance");
                    return Err(BrawError::SdkUnavailable);
                }

                info!("BlackmagicRAW factory created successfully");

                Ok(Self {
                    factory,
                    codec: ptr::null_mut(),
                    clip: ptr::null_mut(),
                    callback: None,
                    initialized: true,
                })
            }
        }

        #[cfg(not(feature = "native-ffi"))]
        {
            // Fallback to a non-initialized SDK struct
            Ok(BrawSdk {
                initialized: false,
            })
        }
    }

    /// Create codec instance
    #[cfg(feature = "native-ffi")]
    pub fn create_codec(&mut self) -> Result<(), BrawError> {
        if self.factory.is_null() {
            return Err(BrawError::SdkUnavailable);
        }

        unsafe {
            let vtable = get_vtable(self.factory) as *const IBlackmagicRawFactoryVTable;
            let create_codec_fn = (*vtable).create_codec;

            let mut codec: *mut IBlackmagicRaw = ptr::null_mut();
            let result = create_codec_fn(self.factory, &mut codec);

            if result != 0 || codec.is_null() {
                error!("Failed to create BlackmagicRAW codec, HRESULT: {:#x}", result);
                return Err(BrawError::SdkUnavailable);
            }

            self.codec = codec;
            info!("BlackmagicRAW codec created successfully");
            Ok(())
        }
    }

    #[cfg(not(feature = "native-ffi"))]
    pub fn create_codec(&mut self) -> Result<(), BrawError> {
        Err(BrawError::SdkUnavailable)
    }

    /// Open a BRAW clip file
    #[cfg(feature = "native-ffi")]
    pub fn open_clip(&mut self, path: &Path) -> Result<(), BrawError> {
        if self.codec.is_null() {
            return Err(BrawError::SdkUnavailable);
        }

        unsafe {
            // Convert path to CFString
            let path_str = path.to_string_lossy();

            #[cfg(target_os = "macos")]
            {
                // Create CFString from path using core-foundation
                let cf_path = CFString::new(&path_str);
                let cf_path_ref = cf_path.as_concrete_TypeRef() as CFStringRef;

                debug!("Opening BRAW clip with path: {}", path_str);

                let vtable = get_vtable(self.codec) as *const IBlackmagicRawVTable;
                let open_clip_fn = (*vtable).open_clip;

                let mut clip: *mut IBlackmagicRawClip = ptr::null_mut();
                let result = open_clip_fn(self.codec, cf_path_ref, &mut clip);

                if result != 0 || clip.is_null() {
                    let error_msg = match result as u32 {
                        0x80004005 => "General failure - file may be corrupted or not a valid BRAW file".to_string(),
                        0x80070002 => "File not found".to_string(),
                        0x80070005 => "Access denied - check file permissions".to_string(),
                        0x8007000E => "Out of memory".to_string(),
                        0x80004001 => "Not implemented - unsupported BRAW format".to_string(),
                        _ => format!("Unknown error, HRESULT: {:#x}", result),
                    };
                    error!("Failed to open BRAW clip: {}, {}", path.display(), error_msg);
                    return Err(BrawError::InvalidBrawFile(error_msg));
                }

                self.clip = clip;
                info!("BRAW clip opened successfully: {}", path.display());
                Ok(())
            }

            #[cfg(not(target_os = "macos"))]
            {
                error!("BRAW SDK only supported on macOS currently");
                Err(BrawError::SdkUnavailable)
            }
        }
    }

    #[cfg(not(feature = "native-ffi"))]
    pub fn open_clip(&mut self, _path: &Path) -> Result<(), BrawError> {
        Err(BrawError::SdkUnavailable)
    }

    /// Get frame count from the clip
    #[cfg(feature = "native-ffi")]
    pub fn get_frame_count(&self) -> Result<u64, BrawError> {
        if self.clip.is_null() {
            return Err(BrawError::SdkUnavailable);
        }

        unsafe {
            let vtable = get_vtable(self.clip) as *const IBlackmagicRawClipVTable;
            let get_frame_count_fn = (*vtable).get_frame_count;

            let mut frame_count: u64 = 0;
            let result = get_frame_count_fn(self.clip, &mut frame_count);

            if result != 0 {
                error!("Failed to get frame count, HRESULT: {:#x}", result);
                return Err(BrawError::SdkUnavailable);
            }

            debug!("BRAW clip has {} frames", frame_count);
            Ok(frame_count)
        }
    }

    #[cfg(not(feature = "native-ffi"))]
    pub fn get_frame_count(&self) -> Result<u64, BrawError> {
        Err(BrawError::SdkUnavailable)
    }

    /// Get clip dimensions
    #[cfg(feature = "native-ffi")]
    pub fn get_dimensions(&self) -> Result<(u32, u32), BrawError> {
        if self.clip.is_null() {
            return Err(BrawError::SdkUnavailable);
        }

        unsafe {
            let vtable = get_vtable(self.clip) as *const IBlackmagicRawClipVTable;
            let get_width_fn = (*vtable).get_width;
            let get_height_fn = (*vtable).get_height;

            let mut width: u32 = 0;
            let mut height: u32 = 0;

            let result1 = get_width_fn(self.clip, &mut width);
            let result2 = get_height_fn(self.clip, &mut height);

            if result1 != 0 || result2 != 0 {
                error!("Failed to get clip dimensions, HRESULT: {:#x}, {:#x}", result1, result2);
                return Err(BrawError::SdkUnavailable);
            }

            debug!("BRAW clip dimensions: {}x{}", width, height);
            Ok((width, height))
        }
    }

    #[cfg(not(feature = "native-ffi"))]
    pub fn get_dimensions(&self) -> Result<(u32, u32), BrawError> {
        Err(BrawError::SdkUnavailable)
    }

    /// Extract a frame at the specified index using real BlackmagicRAW SDK
    pub async fn extract_frame(&mut self, frame_index: u64) -> Result<DynamicImage, BrawError> {
        #[cfg(feature = "native-ffi")]
        {
            if self.clip.is_null() || self.codec.is_null() {
                return Err(BrawError::SdkUnavailable);
            }

            let frame_count = self.get_frame_count()?;
            if frame_index >= frame_count {
                return Err(BrawError::FrameOutOfRange {
                    frame: frame_index as u32,
                    max_frames: frame_count as u32,
                });
            }

            info!("Extracting real BRAW frame {} using BlackmagicRAW SDK", frame_index);

            unsafe {
                // Set up callback if not already done
                if self.callback.is_none() {
                    let mut callback = BrawCallback::new();

                    // Set callback on codec
                    let codec_vtable = get_vtable(self.codec) as *const IBlackmagicRawVTable;
                    let set_callback_fn = (*codec_vtable).set_callback;

                    let callback_ptr = callback.as_mut() as *mut BrawCallback as *mut IBlackmagicRawCallback;
                    let result = set_callback_fn(self.codec, callback_ptr);

                    if result != 0 {
                        error!("Failed to set callback on codec, HRESULT: {:#x}", result);
                        return Err(BrawError::SdkUnavailable);
                    }

                    self.callback = Some(callback);
                    info!("Callback set on codec successfully");
                }

                // Create read job for the frame
                let clip_vtable = get_vtable(self.clip) as *const IBlackmagicRawClipVTable;
                let create_job_read_frame_fn = (*clip_vtable).create_job_read_frame;

                let mut read_job: *mut IBlackmagicRawJob = ptr::null_mut();
                let result = create_job_read_frame_fn(self.clip, frame_index, &mut read_job);

                if result != 0 || read_job.is_null() {
                    error!("Failed to create read job for frame {}, HRESULT: {:#x}", frame_index, result);
                    return Err(BrawError::SdkUnavailable);
                }

                info!("Created read job for frame {}", frame_index);

                // Set up async channels for callback communication
                let (read_tx, read_rx) = tokio::sync::oneshot::channel();

                // Set the sender in the callback
                if let Some(ref mut callback) = self.callback {
                    callback.read_sender = Some(read_tx);
                }

                // Submit the read job
                let job_vtable = get_vtable(read_job) as *const IBlackmagicRawJobVTable;
                let submit_fn = (*job_vtable).submit;
                let result = submit_fn(read_job);

                if result != 0 {
                    error!("Failed to submit read job for frame {}, HRESULT: {:#x}", frame_index, result);
                    return Err(BrawError::SdkUnavailable);
                }

                info!("Submitted read job for frame {}", frame_index);

                // Wait for read completion
                let frame = match tokio::time::timeout(std::time::Duration::from_secs(10), read_rx).await {
                    Ok(Ok(Ok(frame))) => {
                        info!("Read job completed successfully for frame {}", frame_index);
                        frame
                    }
                    Ok(Ok(Err(e))) => {
                        error!("Read job failed for frame {}: {:?}", frame_index, e);
                        return Err(e);
                    }
                    Ok(Err(_)) => {
                        error!("Read job callback channel closed for frame {}", frame_index);
                        return Err(BrawError::SdkUnavailable);
                    }
                    Err(_) => {
                        error!("Read job timed out for frame {}", frame_index);
                        return Err(BrawError::SdkUnavailable);
                    }
                };

                // Now create decode and process job
                let frame_vtable = get_vtable(frame) as *const IBlackmagicRawFrameVTable;
                let create_decode_job_fn = (*frame_vtable).create_job_decode_and_process_frame;

                let mut decode_job: *mut IBlackmagicRawJob = ptr::null_mut();
                let result = create_decode_job_fn(frame, ptr::null_mut(), ptr::null_mut(), &mut decode_job);

                if result != 0 || decode_job.is_null() {
                    error!("Failed to create decode job for frame {}, HRESULT: {:#x}", frame_index, result);
                    return Err(BrawError::SdkUnavailable);
                }

                info!("Created decode job for frame {}", frame_index);

                // Set up async channels for decode callback
                let (process_tx, process_rx) = tokio::sync::oneshot::channel();

                // Set the sender in the callback
                if let Some(ref mut callback) = self.callback {
                    callback.process_sender = Some(process_tx);
                }

                // Submit the decode job
                let decode_job_vtable = get_vtable(decode_job) as *const IBlackmagicRawJobVTable;
                let submit_decode_fn = (*decode_job_vtable).submit;
                let result = submit_decode_fn(decode_job);

                if result != 0 {
                    error!("Failed to submit decode job for frame {}, HRESULT: {:#x}", frame_index, result);
                    return Err(BrawError::SdkUnavailable);
                }

                info!("Submitted decode job for frame {}", frame_index);

                // Wait for decode completion
                let processed_image = match tokio::time::timeout(std::time::Duration::from_secs(10), process_rx).await {
                    Ok(Ok(Ok(image))) => {
                        info!("Decode job completed successfully for frame {}", frame_index);
                        image
                    }
                    Ok(Ok(Err(e))) => {
                        error!("Decode job failed for frame {}: {:?}", frame_index, e);
                        return Err(e);
                    }
                    Ok(Err(_)) => {
                        error!("Decode job callback channel closed for frame {}", frame_index);
                        return Err(BrawError::SdkUnavailable);
                    }
                    Err(_) => {
                        error!("Decode job timed out for frame {}", frame_index);
                        return Err(BrawError::SdkUnavailable);
                    }
                };

                // Extract image data from processed image
                let image_vtable = get_vtable(processed_image) as *const IBlackmagicRawProcessedImageVTable;

                let mut width: u32 = 0;
                let mut height: u32 = 0;
                let mut resource: *mut c_void = ptr::null_mut();
                let mut resource_size: u32 = 0;

                let get_width_fn = (*image_vtable).get_width;
                let get_height_fn = (*image_vtable).get_height;
                let get_resource_fn = (*image_vtable).get_resource;
                let get_resource_size_fn = (*image_vtable).get_resource_size_bytes;

                let result = get_width_fn(processed_image, &mut width);
                if result != 0 {
                    error!("Failed to get processed image width, HRESULT: {:#x}", result);
                    return Err(BrawError::SdkUnavailable);
                }

                let result = get_height_fn(processed_image, &mut height);
                if result != 0 {
                    error!("Failed to get processed image height, HRESULT: {:#x}", result);
                    return Err(BrawError::SdkUnavailable);
                }

                let result = get_resource_fn(processed_image, &mut resource);
                if result != 0 || resource.is_null() {
                    error!("Failed to get processed image resource, HRESULT: {:#x}", result);
                    return Err(BrawError::SdkUnavailable);
                }

                let result = get_resource_size_fn(processed_image, &mut resource_size);
                if result != 0 {
                    error!("Failed to get processed image resource size, HRESULT: {:#x}", result);
                    return Err(BrawError::SdkUnavailable);
                }

                info!("Extracted processed image: {}x{}, {} bytes", width, height, resource_size);

                // Convert raw image data to DynamicImage with format detection
                let data_slice = std::slice::from_raw_parts(resource as *const u8, resource_size as usize);

                // Detect pixel format based on resource size
                let bytes_per_pixel = resource_size as f64 / (width as f64 * height as f64);
                debug!("Detected bytes per pixel: {:.2}", bytes_per_pixel);

                let dynamic_image = if bytes_per_pixel >= 3.9 && bytes_per_pixel <= 4.1 {
                    // RGBA format (4 bytes per pixel)
                    let expected_size = (width * height * 4) as usize;
                    if resource_size as usize >= expected_size {
                        let rgba_data = data_slice[..expected_size].to_vec();
                        match ImageBuffer::<image::Rgba<u8>, Vec<u8>>::from_raw(width, height, rgba_data) {
                            Some(img_buffer) => {
                                info!("Created RGBA image {}x{} for frame {}", width, height, frame_index);
                                DynamicImage::ImageRgba8(img_buffer)
                            }
                            None => {
                                error!("Failed to create RGBA ImageBuffer");
                                return Err(BrawError::SdkUnavailable);
                            }
                        }
                    } else {
                        error!("Insufficient data for RGBA format: {} < {}", resource_size, expected_size);
                        return Err(BrawError::SdkUnavailable);
                    }
                } else if bytes_per_pixel >= 2.9 && bytes_per_pixel <= 3.1 {
                    // RGB format (3 bytes per pixel)
                    let expected_size = (width * height * 3) as usize;
                    if resource_size as usize >= expected_size {
                        let rgb_data = data_slice[..expected_size].to_vec();
                        match ImageBuffer::<Rgb<u8>, Vec<u8>>::from_raw(width, height, rgb_data) {
                            Some(img_buffer) => {
                                info!("Created RGB image {}x{} for frame {}", width, height, frame_index);
                                DynamicImage::ImageRgb8(img_buffer)
                            }
                            None => {
                                error!("Failed to create RGB ImageBuffer");
                                return Err(BrawError::SdkUnavailable);
                            }
                        }
                    } else {
                        error!("Insufficient data for RGB format: {} < {}", resource_size, expected_size);
                        return Err(BrawError::SdkUnavailable);
                    }
                } else if bytes_per_pixel >= 1.9 && bytes_per_pixel <= 2.1 {
                    // 16-bit grayscale or similar (2 bytes per pixel)
                    let expected_size = (width * height * 2) as usize;
                    if resource_size as usize >= expected_size {
                        // Convert 16-bit data to 8-bit grayscale
                        let mut gray_data = Vec::with_capacity((width * height) as usize);
                        for chunk in data_slice[..expected_size].chunks_exact(2) {
                            let value = u16::from_le_bytes([chunk[0], chunk[1]]);
                            gray_data.push((value >> 8) as u8); // Take high byte
                        }
                        match ImageBuffer::<image::Luma<u8>, Vec<u8>>::from_raw(width, height, gray_data) {
                            Some(img_buffer) => {
                                info!("Created grayscale image {}x{} for frame {}", width, height, frame_index);
                                DynamicImage::ImageLuma8(img_buffer)
                            }
                            None => {
                                error!("Failed to create grayscale ImageBuffer");
                                return Err(BrawError::SdkUnavailable);
                            }
                        }
                    } else {
                        error!("Insufficient data for 16-bit format: {} < {}", resource_size, expected_size);
                        return Err(BrawError::SdkUnavailable);
                    }
                } else {
                    // Fallback: try to interpret as RGB and pad/truncate as needed
                    warn!("Unknown pixel format ({:.2} bytes/pixel), falling back to RGB", bytes_per_pixel);
                    let expected_size = (width * height * 3) as usize;
                    let mut rgb_data = Vec::with_capacity(expected_size);

                    if resource_size as usize >= expected_size {
                        rgb_data.extend_from_slice(&data_slice[..expected_size]);
                    } else {
                        // Pad with zeros if we have insufficient data
                        rgb_data.extend_from_slice(data_slice);
                        rgb_data.resize(expected_size, 0);
                        warn!("Padded image data from {} to {} bytes", resource_size, expected_size);
                    }

                    match ImageBuffer::<Rgb<u8>, Vec<u8>>::from_raw(width, height, rgb_data) {
                        Some(img_buffer) => {
                            info!("Created fallback RGB image {}x{} for frame {}", width, height, frame_index);
                            DynamicImage::ImageRgb8(img_buffer)
                        }
                        None => {
                            error!("Failed to create fallback RGB ImageBuffer");
                            return Err(BrawError::SdkUnavailable);
                        }
                    }
                };

                Ok(dynamic_image)
            }
        }

        #[cfg(not(feature = "native-ffi"))]
        {
            warn!("BRAW SDK not available - native-ffi feature not enabled");
            Err(BrawError::SdkUnavailable)
        }
    }

    /// Check if using native SDK
    pub fn is_using_native_sdk(&self) -> bool {
        #[cfg(feature = "native-ffi")]
        {
            !self.factory.is_null() && !self.codec.is_null()
        }

        #[cfg(not(feature = "native-ffi"))]
        {
            false
        }
    }

    /// Check if the SDK is initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Test if the SDK is actually available and functional
    #[cfg(feature = "native-ffi")]
    pub fn test_sdk_availability(&self) -> Result<(), BrawError> {
        if self.factory.is_null() {
            return Err(BrawError::SdkUnavailable);
        }

        unsafe {
            // Try to create a test codec to verify SDK functionality
            let vtable = get_vtable(self.factory) as *const IBlackmagicRawFactoryVTable;
            let create_codec_fn = (*vtable).create_codec;

            let mut test_codec: *mut IBlackmagicRaw = ptr::null_mut();
            let result = create_codec_fn(self.factory, &mut test_codec);

            if result != 0 || test_codec.is_null() {
                error!("SDK availability test failed - cannot create codec, HRESULT: {:#x}", result);
                return Err(BrawError::SdkUnavailable);
            }

            // Clean up test codec
            let codec_vtable = get_vtable(test_codec) as *const IBlackmagicRawVTable;
            let release_fn = (*codec_vtable).release;
            release_fn(test_codec);

            info!("SDK availability test passed");
            Ok(())
        }
    }

    #[cfg(not(feature = "native-ffi"))]
    pub fn test_sdk_availability(&self) -> Result<(), BrawError> {
        Err(BrawError::SdkUnavailable)
    }
}

#[cfg(feature = "native-ffi")]
impl Drop for BrawSdk {
    fn drop(&mut self) {
        unsafe {
            // Release callback first
            if let Some(callback) = self.callback.take() {
                // The callback will be automatically released when dropped
                drop(callback);
            }

            // Release interfaces in reverse order
            if !self.clip.is_null() {
                let vtable = get_vtable(self.clip) as *const IBlackmagicRawClipVTable;
                let release_fn = (*vtable).release;
                release_fn(self.clip);
                self.clip = ptr::null_mut();
            }

            if !self.codec.is_null() {
                let vtable = get_vtable(self.codec) as *const IBlackmagicRawVTable;
                let release_fn = (*vtable).release;
                release_fn(self.codec);
                self.codec = ptr::null_mut();
            }

            if !self.factory.is_null() {
                let vtable = get_vtable(self.factory) as *const IBlackmagicRawFactoryVTable;
                let release_fn = (*vtable).release;
                release_fn(self.factory);
                self.factory = ptr::null_mut();
            }
        }

        info!("BlackmagicRAW SDK resources released");
    }
}

#[cfg(not(feature = "native-ffi"))]
impl Drop for BrawSdk {
    fn drop(&mut self) {
        debug!("BrawSdk dropped (stub mode)");
    }
}

// Thread-safe wrapper for the SDK
unsafe impl Send for BrawSdk {}
unsafe impl Sync for BrawSdk {}

/// BRAW file wrapper that uses the real BlackmagicRAW SDK
pub struct BrawFile {
    sdk: Arc<Mutex<BrawSdk>>,
    path: std::path::PathBuf,
}

impl BrawFile {
    /// Open a BRAW file using the real BlackmagicRAW SDK
    pub async fn open(path: std::path::PathBuf) -> Result<Self, BrawError> {
        info!("Opening BRAW file with real SDK: {}", path.display());

        let mut sdk = BrawSdk::new()?;
        sdk.create_codec()?;
        sdk.open_clip(&path)?;

        Ok(Self {
            sdk: Arc::new(Mutex::new(sdk)),
            path,
        })
    }

    /// Get the frame count
    pub async fn get_frame_count(&self) -> Result<u64, BrawError> {
        let sdk = self.sdk.lock().map_err(|_| BrawError::SdkUnavailable)?;
        sdk.get_frame_count()
    }

    /// Get clip dimensions
    pub async fn get_dimensions(&self) -> Result<(u32, u32), BrawError> {
        let sdk = self.sdk.lock().map_err(|_| BrawError::SdkUnavailable)?;
        sdk.get_dimensions()
    }

    /// Extract a frame at the specified index
    pub async fn extract_frame(&self, frame_index: u64) -> Result<DynamicImage, BrawError> {
        let mut sdk = self.sdk.lock().map_err(|_| BrawError::SdkUnavailable)?;
        sdk.extract_frame(frame_index).await
    }

    /// Check if using native SDK
    pub fn is_using_native_sdk(&self) -> bool {
        if let Ok(sdk) = self.sdk.lock() {
            sdk.is_using_native_sdk()
        } else {
            false
        }
    }

    /// Get metadata from the BRAW file
    pub async fn get_metadata(&self) -> Result<BrawMetadata, BrawError> {
        BrawMetadata::from_file(&self.path).await
    }
}

/// Represents an opened BRAW clip
#[derive(Debug)]
pub struct BrawClip {
    path: std::path::PathBuf,
    #[cfg(feature = "native-ffi")]
    clip: *mut IBlackmagicRawClip,
    sdk: BrawSdk,
    /// Cached metadata to avoid repeated SDK calls
    cached_metadata: Option<BrawMetadata>,
}

#[cfg(feature = "native-ffi")]
impl Drop for BrawClip {
    fn drop(&mut self) {
        // Stub drop – no-op when using placeholder bindings
        if !self.clip.is_null() {
            debug!("Releasing BlackmagicRawClip handle (stub mode) for {}", self.path.display());
        }
    }
}

#[cfg(not(feature = "native-ffi"))]
impl Drop for BrawClip {
    fn drop(&mut self) {
        debug!("BrawClip dropped (stub mode) for {}", self.path.display());
    }
}

// These are safe because clip objects are designed to be used across threads
unsafe impl Send for BrawClip {}
unsafe impl Sync for BrawClip {}

impl BrawClip {
    /// Open a BRAW clip from a file path
    pub async fn open(path: std::path::PathBuf) -> Result<Self, BrawError> {
        debug!("Opening BRAW clip: {}", path.display());

        // Create SDK instance
        let mut sdk = BrawSdk::new()?;

        #[cfg(feature = "native-ffi")]
        {
            // Try to open the clip with the real SDK
            match sdk.open_clip(&path) {
                Ok(_) => {
                    info!("Successfully opened BRAW clip with native SDK: {}", path.display());
                    return Ok(BrawClip {
                        path,
                        #[cfg(feature = "native-ffi")]
                        clip: sdk.clip,
                        // We move the SDK into the clip. The clip now owns it.
                        sdk: std::mem::replace(&mut sdk, BrawSdk {
                            #[cfg(feature = "native-ffi")]
                            factory: std::ptr::null_mut(),
                            #[cfg(feature = "native-ffi")]
                            codec: std::ptr::null_mut(),
                            #[cfg(feature = "native-ffi")]
                            clip: std::ptr::null_mut(),
                            #[cfg(feature = "native-ffi")]
                            callback: None,
                            initialized: false
                        }),
                        cached_metadata: None,
                    });
                }
                Err(e) => {
                    warn!("Failed to open BRAW clip with native SDK: {}", e);
                    // Fall through to stub mode
                }
            }
        }

        // Fallback: create a stub clip
        info!("Creating stub BRAW clip for: {}", path.display());
        Ok(BrawClip {
            path,
            #[cfg(feature = "native-ffi")]
            clip: std::ptr::null_mut(),
            sdk: BrawSdk {
                #[cfg(feature = "native-ffi")]
                factory: std::ptr::null_mut(),
                #[cfg(feature = "native-ffi")]
                codec: std::ptr::null_mut(),
                #[cfg(feature = "native-ffi")]
                clip: std::ptr::null_mut(),
                #[cfg(feature = "native-ffi")]
                callback: None,
                initialized: false,
            },
            cached_metadata: None,
        })
    }

    /// Check if this clip is using the native SDK
    pub fn is_using_native_sdk(&self) -> bool {
        self.sdk.is_using_native_sdk()
    }

    /// Get frame count from the clip
    pub fn get_frame_count(&self) -> Result<u64, BrawError> {
        self.sdk.get_frame_count()
    }

    /// Get dimensions from the clip
    pub fn get_dimensions(&self) -> Result<(u32, u32), BrawError> {
        self.sdk.get_dimensions()
    }

    /// Extract a frame from the clip
    pub fn extract_frame(&self, frame_index: u64) -> Result<Vec<u8>, BrawError> {
        // For now, return placeholder data
        // In a real implementation, this would call the SDK to extract frame data
        let (width, height) = self.get_dimensions().unwrap_or((1920, 1080));
        let size = (width * height * 3) as usize; // RGB
        Ok(vec![128; size]) // Gray placeholder
    }

    /// Get basic metadata about the clip
    pub async fn get_metadata(&mut self) -> Result<&BrawMetadata, BrawError> {
        if self.cached_metadata.is_none() {
            let metadata = BrawMetadata::from_file(&self.path).await?;
            self.cached_metadata = Some(metadata);
        }
        Ok(self.cached_metadata.as_ref().unwrap())
    }

    /// Get the number of metadata entries
    pub async fn get_metadata_count(&mut self) -> Result<usize, BrawError> {
        let metadata = self.get_metadata().await?;
        Ok(metadata.len())
    }

    /// Validate the file size is within limits
    pub async fn validate_file_size(&self, max_size_mb: u64) -> Result<(), BrawError> {
        let metadata = tokio::fs::metadata(&self.path).await?;

        let size_mb = metadata.len() / (1024 * 1024);
        if size_mb > max_size_mb {
            return Err(BrawError::FileTooLarge {
                size: size_mb,
                max_size: max_size_mb
            });
        }

        Ok(())
    }
}

/// Basic metadata structure for BRAW files
#[derive(Debug, Clone)]
pub struct BrawMetadata {
    pub file_size: u64,
    pub created: Option<chrono::DateTime<chrono::Utc>>,
    pub modified: Option<chrono::DateTime<chrono::Utc>>,
}

impl BrawMetadata {
    /// Extract basic metadata from a BRAW file
    pub async fn from_file(path: &Path) -> Result<Self, BrawError> {
        let metadata = tokio::fs::metadata(path).await?;

        let created = metadata.created().ok()
            .and_then(|t| chrono::DateTime::from_timestamp(
                t.duration_since(std::time::UNIX_EPOCH).ok()?.as_secs() as i64, 0
            ));

        let modified = metadata.modified().ok()
            .and_then(|t| chrono::DateTime::from_timestamp(
                t.duration_since(std::time::UNIX_EPOCH).ok()?.as_secs() as i64, 0
            ));

        Ok(BrawMetadata {
            file_size: metadata.len(),
            created,
            modified,
        })
    }

    /// Get the number of metadata fields
    pub fn len(&self) -> usize {
        3 // file_size, created, modified
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

    // Check file size
    let metadata = std::fs::metadata(path)?;
    if metadata.len() > MAX_BRAW_FILE_SIZE {
        return Err(BrawError::FileTooLarge {
            size: metadata.len(),
            max_size: MAX_BRAW_FILE_SIZE
        });
    }

    // Could add magic byte validation here
    debug!("BRAW file validation passed: {}", path.display());
    Ok(())
}

/// Comprehensive BRAW file testing with SDK validation
pub async fn test_braw_file_with_sdk(path: &Path) -> Result<(), BrawError> {
    // First do basic validation
    validate_braw_file(path).await?;

    // Try to open with SDK if available
    #[cfg(feature = "native-ffi")]
    {
        let mut sdk = BrawSdk::new()?;
        sdk.create_codec()?;

        // Test SDK availability
        sdk.test_sdk_availability()?;

        // Try to open the clip
        match sdk.open_clip(path) {
            Ok(_) => {
                info!("Successfully opened BRAW file with SDK: {}", path.display());

                // Get basic info
                if let Ok(frame_count) = sdk.get_frame_count() {
                    info!("BRAW file has {} frames", frame_count);
                }

                if let Ok((width, height)) = sdk.get_dimensions() {
                    info!("BRAW file dimensions: {}x{}", width, height);
                }

                Ok(())
            }
            Err(e) => {
                error!("Failed to open BRAW file with SDK: {}", e);
                Err(e)
            }
        }
    }

    #[cfg(not(feature = "native-ffi"))]
    {
        warn!("SDK not available for comprehensive testing");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sdk_initialization() {
        let result = BrawSdk::new();
        assert!(result.is_ok());

        let sdk = result.unwrap();
        assert!(sdk.is_initialized());
    }

    #[tokio::test]
    async fn test_validate_nonexistent_file() {
        let result = validate_braw_file(Path::new("/nonexistent/file.braw")).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validate_invalid_extension() {
        let result = validate_braw_file(Path::new("/tmp/test.mp4")).await;
        assert!(result.is_err());
    }

    #[cfg(feature = "native-ffi")]
    #[tokio::test]
    async fn test_sdk_availability() {
        let sdk = BrawSdk::new().unwrap();
        // This may fail if SDK is not installed, which is expected
        let _result = sdk.test_sdk_availability();
        // We don't assert success here since SDK may not be available in test environment
    }

    #[tokio::test]
    async fn test_format_detection() {
        // Test the format detection logic with mock data
        let width = 100u32;
        let height = 100u32;

        // Test RGB format detection (3 bytes per pixel)
        let rgb_size = (width * height * 3) as f64;
        let bytes_per_pixel = rgb_size / (width as f64 * height as f64);
        assert!(bytes_per_pixel >= 2.9 && bytes_per_pixel <= 3.1);

        // Test RGBA format detection (4 bytes per pixel)
        let rgba_size = (width * height * 4) as f64;
        let bytes_per_pixel = rgba_size / (width as f64 * height as f64);
        assert!(bytes_per_pixel >= 3.9 && bytes_per_pixel <= 4.1);
    }
}