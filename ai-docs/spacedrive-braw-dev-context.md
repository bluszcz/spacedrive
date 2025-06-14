# Spacedrive BRAW Support - Development Context

## Quick Reference for AI Coding Assistants

### Project Overview
Adding BlackmagicRAW (BRAW) file support to Spacedrive - a cross-platform file explorer written in Rust + TypeScript.

### Key Files to Modify

```
spacedrive/
├── crates/file-ext/src/extensions.rs     # Add BRAW to VideoExtension enum
├── crates/media-metadata/src/            # Add BRAW metadata extraction
├── crates/braw/                          # NEW: BRAW SDK integration crate
└── core/src/                             # Core integration updates
```

### Architecture Pattern
Spacedrive uses:
- **File Extensions**: Magic bytes + extension detection in `crates/file-ext`
- **Media Metadata**: FFmpeg for most video, specialized crates for others
- **Thumbnails**: Extracted through media-metadata crate
- **Error Handling**: Custom error types with proper propagation

### Implementation Steps

#### 1. File Type Detection
```rust
// In crates/file-ext/src/extensions.rs
VideoExtension {
    // ... existing variants
    Braw = [/* magic bytes for BRAW */],
}
```

#### 2. Create BRAW Crate
```rust
// crates/braw/Cargo.toml
[package]
name = "sd-braw"
version = "0.1.0"

[dependencies]
thiserror = "1.0"
image = "0.24"

[build-dependencies]
bindgen = "0.69"  # For SDK bindings
```

#### 3. BRAW SDK Integration Pattern
```rust
// crates/braw/src/lib.rs
use std::path::Path;
use image::DynamicImage;

pub struct BrawFile {
    // SDK handle
}

impl BrawFile {
    pub fn open(path: &Path) -> Result<Self, BrawError>;
    pub fn get_metadata(&self) -> Result<BrawMetadata, BrawError>;
    pub fn extract_thumbnail(&self) -> Result<DynamicImage, BrawError>;
    pub fn extract_frame(&self, frame: u32) -> Result<DynamicImage, BrawError>;
}

pub struct BrawMetadata {
    pub width: u32,
    pub height: u32,
    pub frame_rate: f64,
    pub duration_seconds: f64,
    pub codec: String,
    pub color_space: String,
    // Camera-specific metadata
    pub iso: Option<u32>,
    pub shutter_speed: Option<String>,
    pub color_temperature: Option<u32>,
}

#[derive(thiserror::Error, Debug)]
pub enum BrawError {
    #[error("Failed to open BRAW file: {0}")]
    OpenFailed(String),
    #[error("SDK not available")]
    SdkUnavailable,
    #[error("Invalid file format")]
    InvalidFormat,
}
```

#### 4. Media Metadata Integration
```rust
// crates/media-metadata/src/braw.rs
use sd_braw::{BrawFile, BrawMetadata};

pub fn extract_braw_metadata(path: &Path) -> Result<MediaMetadata, MediaError> {
    let braw_file = BrawFile::open(path)?;
    let metadata = braw_file.get_metadata()?;
    
    Ok(MediaMetadata {
        duration: Some(metadata.duration_seconds),
        video_props: Some(VideoProps {
            width: metadata.width,
            height: metadata.height,
            // ... map other fields
        }),
        // ... handle other metadata fields
    })
}
```

### Integration Points

#### File Extension System
```rust
// Pattern from existing code in extensions.rs
extension_category_enum! {
    VideoExtension ALL_VIDEO_EXTENSIONS {
        // ... existing extensions
        Braw = [0x42, 0x52, 0x41, 0x57], // Example magic bytes
    }
}
```

#### Media Metadata Factory
```rust
// Integration pattern in media-metadata/src/lib.rs
pub fn extract_media_metadata(path: &Path) -> Result<MediaMetadata, MediaError> {
    let extension = Extension::resolve_conflicting(
        path.extension()?.to_str()?,
        true
    ).await?;
    
    match extension {
        Extension::Video(VideoExtension::Braw) => {
            crate::braw::extract_braw_metadata(path)
        },
        // ... other cases
    }
}
```

### Build System Integration

#### Cargo.toml Updates
```toml
# Add to workspace members
[workspace]
members = [
    # ... existing members
    "crates/braw",
]

# Add feature flag
[features]
default = ["braw"]
braw = ["sd-braw"]
```

#### Cross-Platform Considerations
```rust
// crates/braw/build.rs
fn main() {
    #[cfg(target_os = "windows")]
    link_windows_sdk();
    
    #[cfg(target_os = "macos")]
    link_macos_sdk();
    
    #[cfg(target_os = "linux")]
    link_linux_sdk();
}
```

### Testing Patterns
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_braw_detection() {
        let test_file = PathBuf::from("test-data/sample.braw");
        let extension = Extension::resolve_conflicting("braw", true).await;
        assert_eq!(extension, Some(Extension::Video(VideoExtension::Braw)));
    }

    #[tokio::test]
    async fn test_braw_metadata() {
        let test_file = PathBuf::from("test-data/sample.braw");
        let metadata = extract_media_metadata(&test_file).await.unwrap();
        assert!(metadata.video_props.is_some());
    }
}
```

### Error Handling Pattern
Follow Spacedrive's error handling:
```rust
// Use thiserror for error types
#[derive(thiserror::Error, Debug)]
pub enum BrawError {
    #[error("BRAW SDK error: {0}")]
    Sdk(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

// Convert to MediaError for integration
impl From<BrawError> for MediaError {
    fn from(err: BrawError) -> Self {
        MediaError::BrawProcessing(err.to_string())
    }
}
```

### Performance Considerations
- Use async/await for file operations
- Stream large file processing
- Cache thumbnails and metadata
- Handle memory management for large BRAW files
- Consider using Rayon for parallel processing

### Dependencies to Add
```toml
[dependencies]
# For BRAW SDK bindings
bindgen = "0.69"
libc = "0.2"

# For image processing
image = "0.24"

# For async file operations
tokio = { version = "1.0", features = ["fs"] }

# For error handling
thiserror = "1.0"
anyhow = "1.0"
```

### Development Tips
1. Start with file detection first
2. Mock the SDK initially for development
3. Use feature flags to make BRAW support optional
4. Test with sample BRAW files from BlackmagicRAW SDK
5. Follow existing patterns in `crates/ffmpeg` for reference
6. Ensure proper cleanup of SDK resources

### Resources
- BlackmagicRAW SDK: https://www.blackmagicdesign.com/developer/products/braw/sdk-and-software
- Spacedrive Architecture: https://github.com/spacedriveapp/spacedrive
- File Extension Patterns: `crates/file-ext/src/extensions.rs`
- Media Processing Patterns: `crates/media-metadata/src/`
