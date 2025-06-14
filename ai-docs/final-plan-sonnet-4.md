# Final Implementation Plan: BRAW Support for Spacedrive

> **Version**: 1.0  
> **Author**: Claude 3.5 Sonnet  
> **Date**: December 2024  
> **Scope**: Complete implementation roadmap for BlackmagicRAW support in Spacedrive

## Executive Summary

This plan consolidates the best approaches from all previous plans to implement BRAW (BlackmagicRAW) support in Spacedrive efficiently. The implementation follows Spacedrive's modular architecture with clear separation of concerns, async processing, and cross-platform compatibility.

## 🎯 Objectives

1. **File Detection**: Accurate BRAW file identification via extension + magic bytes
2. **Metadata Extraction**: Comprehensive technical and camera metadata
3. **Thumbnail Generation**: High-performance preview generation
4. **Modular Architecture**: Self-contained crate with clean API
5. **Cross-Platform**: Windows, macOS, Linux support
6. **Optional Feature**: Can be disabled for builds without SDK

## 🏗️ Architecture Overview

### Core Components
```
spacedrive/
├── crates/
│   ├── braw/                          # NEW: BRAW SDK integration
│   │   ├── Cargo.toml
│   │   ├── build.rs                   # SDK linking + bindgen
│   │   └── src/
│   │       ├── lib.rs                 # Public API
│   │       ├── sdk.rs                 # SDK wrapper
│   │       ├── metadata.rs            # Metadata extraction
│   │       ├── thumbnail.rs           # Thumbnail generation
│   │       └── error.rs               # Error handling
│   ├── file-ext/src/extensions.rs     # Add BRAW to VideoExtension
│   └── media-metadata/src/            # Integrate BRAW extractor
└── .data/                             # SDK binaries (git-ignored)
```

### Data Flow
```
BRAW File → File Detection → Metadata Extraction → Thumbnail Generation → UI Display
```

## 📋 Implementation Phases

### Phase 1: Project Setup & SDK Integration (Week 1)

#### 1.1 Project Hygiene
- [ ] Update `.gitignore` to exclude SDK binaries and `.data/` folder
- [ ] Create `ai-instructions.md` with BRAW context
- [ ] Set up `.data/sdk/` directory structure

#### 1.2 Create BRAW Crate
```toml
# crates/braw/Cargo.toml
[package]
name = "sd-braw"
version = "0.1.0"
edition = "2021"

[dependencies]
thiserror = "1.0"
image = "0.24"
tokio = { version = "1.0", features = ["fs"] }
libc = "0.2"
tracing = "0.1"

[build-dependencies]
bindgen = "0.69"
```

#### 1.3 SDK Integration
- [ ] Download BlackmagicRAW SDK to `.data/sdk/`
- [ ] Create `build.rs` for cross-platform SDK linking
- [ ] Generate FFI bindings with `bindgen`
- [ ] Implement basic SDK wrapper with RAII pattern

### Phase 2: File Type Detection (Week 1-2)

#### 2.1 Magic Bytes Research
- [ ] Analyze BRAW file headers from SDK samples
- [ ] Identify consistent magic byte patterns
- [ ] Test with multiple camera firmware versions

#### 2.2 Extension System Integration
```rust
// crates/file-ext/src/extensions.rs
extension_category_enum! {
    VideoExtension ALL_VIDEO_EXTENSIONS {
        // ... existing extensions
        Braw = [0x42, 0x52, 0x41, 0x57], // "BRAW" signature
    }
}
```

#### 2.3 Testing
- [ ] Unit tests for BRAW detection
- [ ] Integration tests with sample files
- [ ] Edge case handling (corrupted files, wrong extensions)

### Phase 3: Core BRAW Implementation (Week 2-3)

#### 3.1 Safe SDK Wrapper
```rust
// crates/braw/src/lib.rs
pub struct BrawFile {
    handle: *mut BrawFileHandle,
    path: PathBuf,
}

impl BrawFile {
    pub async fn open(path: &Path) -> Result<Self, BrawError>;
    pub async fn get_metadata(&self) -> Result<BrawMetadata, BrawError>;
    pub async fn extract_thumbnail(&self) -> Result<DynamicImage, BrawError>;
    pub async fn extract_frame(&self, frame: u32) -> Result<DynamicImage, BrawError>;
}

// Automatic cleanup
impl Drop for BrawFile {
    fn drop(&mut self) {
        unsafe { braw_close_file(self.handle) };
    }
}
```

#### 3.2 Metadata Structure
```rust
#[derive(Debug, Clone)]
pub struct BrawMetadata {
    // Technical metadata
    pub width: u32,
    pub height: u32,
    pub frame_rate: f64,
    pub duration_seconds: f64,
    pub total_frames: u32,
    pub codec: String,
    pub color_space: String,
    pub bit_depth: u8,
    
    // Camera metadata
    pub camera_model: Option<String>,
    pub lens_info: Option<String>,
    pub iso: Option<u32>,
    pub shutter_speed: Option<String>,
    pub aperture: Option<f32>,
    pub color_temperature: Option<u32>,
    pub tint: Option<i32>,
    
    // Recording metadata
    pub recording_date: Option<DateTime<Utc>>,
    pub timecode: Option<String>,
    pub reel_name: Option<String>,
    pub scene: Option<String>,
    pub take: Option<String>,
}
```

#### 3.3 Error Handling
```rust
#[derive(thiserror::Error, Debug)]
pub enum BrawError {
    #[error("Failed to open BRAW file: {0}")]
    OpenFailed(String),
    #[error("SDK not available or not initialized")]
    SdkUnavailable,
    #[error("Invalid file format or corrupted file")]
    InvalidFormat,
    #[error("Frame {frame} out of range (0-{max_frames})")]
    FrameOutOfRange { frame: u32, max_frames: u32 },
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Image processing error: {0}")]
    ImageProcessing(#[from] image::ImageError),
}
```

### Phase 4: Media Metadata Integration (Week 3)

#### 4.1 Feature Flag Setup
```toml
# Cargo.toml (workspace root)
[features]
default = ["braw"]
braw = ["sd-braw"]
```

#### 4.2 Metadata Extractor
```rust
// crates/media-metadata/src/braw.rs
use sd_braw::{BrawFile, BrawMetadata};

pub async fn extract_braw_metadata(path: &Path) -> Result<MediaMetadata, MediaError> {
    let braw_file = BrawFile::open(path).await?;
    let metadata = braw_file.get_metadata().await?;
    
    Ok(MediaMetadata {
        duration: Some(metadata.duration_seconds),
        video_props: Some(VideoProps {
            width: metadata.width,
            height: metadata.height,
            frame_rate: metadata.frame_rate,
            bit_depth: Some(metadata.bit_depth),
            // ... map other fields
        }),
        // ... handle camera metadata
    })
}
```

#### 4.3 Integration Point
```rust
// crates/media-metadata/src/lib.rs
pub async fn extract_media_metadata(path: &Path) -> Result<MediaMetadata, MediaError> {
    let extension = Extension::resolve_conflicting(
        path.extension()?.to_str()?,
        true
    ).await?;
    
    match extension {
        #[cfg(feature = "braw")]
        Extension::Video(VideoExtension::Braw) => {
            crate::braw::extract_braw_metadata(path).await
        },
        // ... other cases
    }
}
```

### Phase 5: Thumbnail Generation (Week 3-4)

#### 5.1 Thumbnail Extractor
```rust
// crates/braw/src/thumbnail.rs
impl BrawFile {
    pub async fn extract_thumbnail(&self) -> Result<DynamicImage, BrawError> {
        // Extract first frame or nearest I-frame
        let frame_data = self.extract_raw_frame(0).await?;
        
        // Use SDK to decode to RGB
        let rgb_data = self.decode_frame_to_rgb(&frame_data).await?;
        
        // Convert to DynamicImage
        let image = DynamicImage::ImageRgb8(
            image::RgbImage::from_raw(
                self.width,
                self.height,
                rgb_data
            ).ok_or(BrawError::ImageProcessing("Failed to create image".into()))?
        );
        
        Ok(image)
    }
}
```

#### 5.2 Async Processing
```rust
// Use tokio for non-blocking processing
pub async fn generate_braw_thumbnail(path: &Path, size: u32) -> Result<DynamicImage, BrawError> {
    let braw_file = BrawFile::open(path).await?;
    let thumbnail = braw_file.extract_thumbnail().await?;
    
    // Resize to requested size
    Ok(thumbnail.resize(size, size, image::imageops::FilterType::Lanczos3))
}
```

### Phase 6: Cross-Platform Build System (Week 4)

#### 6.1 Build Script
```rust
// crates/braw/build.rs
use std::env;
use std::path::PathBuf;

fn main() {
    let sdk_path = env::var("BRAW_SDK_PATH")
        .unwrap_or_else(|_| {
            // Default paths for different platforms
            if cfg!(target_os = "windows") {
                r"C:\Program Files\Blackmagic Design\BlackmagicRAW\SDK".to_string()
            } else if cfg!(target_os = "macos") {
                "/usr/local/include/BlackmagicRAW".to_string()
            } else {
                "/usr/include/BlackmagicRAW".to_string()
            }
        });

    link_sdk(&sdk_path);
    generate_bindings(&sdk_path);
}

fn link_sdk(sdk_path: &str) {
    println!("cargo:rustc-link-search=native={}/lib", sdk_path);
    
    #[cfg(target_os = "windows")]
    println!("cargo:rustc-link-lib=dylib=BlackmagicRAW");
    
    #[cfg(target_os = "macos")]
    println!("cargo:rustc-link-lib=dylib=BlackmagicRAW");
    
    #[cfg(target_os = "linux")]
    println!("cargo:rustc-link-lib=dylib=BlackmagicRAW");
}

fn generate_bindings(sdk_path: &str) {
    let bindings = bindgen::Builder::default()
        .header(format!("{}/include/BlackmagicRAW.h", sdk_path))
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
```

#### 6.2 CI/CD Integration
```yaml
# .github/workflows/ci.yml additions
- name: Build with BRAW support
  run: cargo build --features braw
  env:
    BRAW_SDK_PATH: ${{ github.workspace }}/.data/sdk
```

### Phase 7: Testing & Quality Assurance (Week 4-5)

#### 7.1 Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_braw_file_detection() {
        let test_file = std::path::Path::new("test-data/sample.braw");
        let extension = Extension::resolve_conflicting("braw", true).await;
        assert_eq!(extension, Some(Extension::Video(VideoExtension::Braw)));
    }
    
    #[tokio::test]
    async fn test_braw_metadata_extraction() {
        let test_file = std::path::Path::new("test-data/sample.braw");
        let metadata = extract_braw_metadata(test_file).await.unwrap();
        assert!(metadata.video_props.is_some());
        assert!(metadata.duration.is_some());
    }
    
    #[tokio::test]
    async fn test_braw_thumbnail_generation() {
        let test_file = std::path::Path::new("test-data/sample.braw");
        let thumbnail = generate_braw_thumbnail(test_file, 256).await.unwrap();
        assert_eq!(thumbnail.width(), 256);
    }
}
```

#### 7.2 Integration Tests
- [ ] Test with various BRAW file versions
- [ ] Test with different camera models (Pocket, URSA, etc.)
- [ ] Performance tests with large files (>1GB)
- [ ] Memory usage profiling
- [ ] Error handling with corrupted files

#### 7.3 Fuzzing Tests
```rust
#[cfg(test)]
mod fuzz_tests {
    use super::*;
    
    #[test]
    fn fuzz_braw_detection() {
        // Test with random bytes to ensure no crashes
        let random_data = generate_random_bytes(1024);
        let _ = detect_braw_magic_bytes(&random_data);
    }
}
```

### Phase 8: Documentation & Distribution (Week 5)

#### 8.1 Documentation
- [ ] API documentation for `sd-braw` crate
- [ ] Integration guide for developers
- [ ] SDK setup instructions
- [ ] Troubleshooting guide

#### 8.2 Distribution Strategy
- [ ] Optional feature flag for builds without SDK
- [ ] Environment variable for SDK path override
- [ ] Documentation for licensing compliance
- [ ] CI builds with and without BRAW support

## 🚀 Performance Optimizations

### Memory Management
```rust
// Stream processing for large files
pub async fn process_braw_stream(path: &Path) -> Result<(), BrawError> {
    let file = BrawFile::open(path).await?;
    
    // Process in chunks to avoid loading entire file
    let chunk_size = 1024 * 1024; // 1MB chunks
    let mut buffer = Vec::with_capacity(chunk_size);
    
    // Process frame by frame
    for frame_idx in 0..file.total_frames() {
        let frame = file.extract_frame(frame_idx).await?;
        // Process frame and drop immediately
        drop(frame);
    }
    
    Ok(())
}
```

### Async Processing
```rust
// Parallel thumbnail generation
pub async fn generate_thumbnails_batch(
    paths: &[PathBuf],
    size: u32
) -> Vec<Result<DynamicImage, BrawError>> {
    let tasks: Vec<_> = paths.iter()
        .map(|path| generate_braw_thumbnail(path, size))
        .collect();
    
    futures::future::join_all(tasks).await
}
```

## 🔒 Security Considerations

### Input Validation
```rust
pub fn validate_braw_file(path: &Path) -> Result<(), BrawError> {
    // Check file size limits
    let metadata = std::fs::metadata(path)?;
    if metadata.len() > MAX_BRAW_FILE_SIZE {
        return Err(BrawError::FileTooLarge);
    }
    
    // Validate magic bytes
    let mut file = std::fs::File::open(path)?;
    let mut buffer = [0u8; 16];
    file.read_exact(&mut buffer)?;
    
    if !is_valid_braw_header(&buffer) {
        return Err(BrawError::InvalidFormat);
    }
    
    Ok(())
}
```

### Memory Safety
```rust
// All unsafe SDK calls wrapped in safe functions
unsafe fn raw_sdk_call(handle: *mut BrawHandle) -> Result<i32, BrawError> {
    if handle.is_null() {
        return Err(BrawError::SdkUnavailable);
    }
    
    let result = braw_sdk_function(handle);
    if result < 0 {
        return Err(BrawError::SdkError(result));
    }
    
    Ok(result)
}
```

## 📊 Success Metrics

### Functional Requirements
- [ ] ✅ BRAW files correctly identified
- [ ] ✅ Metadata extracted accurately
- [ ] ✅ Thumbnails generated reliably
- [ ] ✅ Cross-platform compatibility
- [ ] ✅ No memory leaks

### Performance Requirements
- [ ] ✅ Thumbnail generation < 2 seconds for 4K files
- [ ] ✅ Metadata extraction < 500ms
- [ ] ✅ Memory usage < 100MB for large files
- [ ] ✅ No UI blocking during processing

### Quality Requirements
- [ ] ✅ 95% test coverage
- [ ] ✅ Zero critical security vulnerabilities
- [ ] ✅ Proper error handling for all edge cases
- [ ] ✅ Comprehensive documentation

## 🎯 Risk Mitigation

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| SDK License Changes | High | Low | Keep feature optional, graceful degradation |
| Platform Compatibility | Medium | Medium | Extensive cross-platform testing |
| Performance Issues | Medium | Medium | Async processing, memory management |
| SDK API Changes | Low | Low | Version pinning, comprehensive tests |

## 🔄 Maintenance Plan

### Long-term Considerations
- [ ] Regular SDK updates monitoring
- [ ] Performance optimization reviews
- [ ] Security audit schedule
- [ ] Documentation updates
- [ ] Community feedback integration

### Upgrade Strategy
- [ ] SDK version compatibility matrix
- [ ] Automated testing pipeline
- [ ] Rollback procedures
- [ ] Migration guides for breaking changes

## 🏁 Conclusion

This plan provides a comprehensive, modular approach to implementing BRAW support in Spacedrive. By following Spacedrive's existing architecture patterns and focusing on performance, security, and maintainability, we can deliver a robust solution that enhances the application's capabilities for video professionals.

The phased approach allows for incremental development and testing, ensuring each component works correctly before building the next layer. The emphasis on async processing, proper error handling, and cross-platform compatibility ensures the implementation will be production-ready and maintainable.

---

**Next Steps:**
1. Review and approve this plan
2. Set up development environment with BRAW SDK
3. Begin Phase 1 implementation
4. Regular progress reviews and adjustments as needed

*End of Document* 