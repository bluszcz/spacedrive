# AI Instructions & Memory Bank - Spacedrive BRAW Support

## Project Context

This document serves as a memory bank for AI coding assistants working on the Spacedrive BRAW (BlackmagicRAW) support implementation.

### Project Overview
- **Goal**: Add comprehensive BlackmagicRAW file support to Spacedrive
- **Architecture**: Rust-based modular implementation following Spacedrive patterns
- **Scope**: File detection, metadata extraction, thumbnail generation
- **Status**: Planning phase complete, ready for implementation

### Key Implementation Principles
1. **Modular Design**: Separate `crates/braw` crate for SDK integration
2. **Async Processing**: Non-blocking operations using tokio
3. **Cross-Platform**: Windows, macOS, Linux support
4. **Optional Feature**: Can be disabled for builds without SDK
5. **Memory Safety**: All unsafe SDK calls wrapped in safe Rust functions
6. **Performance**: Stream processing for large files, async thumbnail generation

### Codebase Architecture

#### File Structure
```
spacedrive/
├── crates/
│   ├── braw/                          # NEW: BRAW SDK integration
│   ├── file-ext/src/extensions.rs     # Add BRAW to VideoExtension enum
│   ├── media-metadata/src/            # Integrate BRAW metadata extraction
│   └── ffmpeg/                        # Reference for video processing patterns
├── vendor/                            # Plans and documentation
│   ├── final-plan-sonnet-4.md        # Main implementation plan
│   └── ai-instructions.md             # This file
└── .data/                             # SDK binaries (git-ignored)
```

#### Key Integration Points
1. **File Detection**: `crates/file-ext/src/extensions.rs` - Add BRAW to VideoExtension enum
2. **Metadata**: `crates/media-metadata/src/` - Add BRAW-specific extractor
3. **SDK Integration**: `crates/braw/` - New crate for BlackmagicRAW SDK
4. **Build System**: Cross-platform linking and bindgen for FFI

### Development Guidelines

#### Error Handling Pattern
```rust
#[derive(thiserror::Error, Debug)]
pub enum BrawError {
    #[error("Failed to open BRAW file: {0}")]
    OpenFailed(String),
    #[error("SDK not available or not initialized")]
    SdkUnavailable,
    #[error("Invalid file format or corrupted file")]
    InvalidFormat,
    // ... other variants
}
```

#### Async Pattern
```rust
pub async fn extract_braw_metadata(path: &Path) -> Result<MediaMetadata, MediaError> {
    let braw_file = BrawFile::open(path).await?;
    let metadata = braw_file.get_metadata().await?;
    // ... process metadata
    Ok(metadata)
}
```

#### Memory Management
- Use RAII pattern for SDK handles
- Implement Drop trait for proper cleanup
- Stream processing for large files
- Avoid loading entire files into memory

### Testing Strategy
1. **Unit Tests**: File detection, metadata extraction, thumbnail generation
2. **Integration Tests**: Cross-platform compatibility, various BRAW versions
3. **Performance Tests**: Large file handling, memory usage profiling
4. **Fuzzing Tests**: Random input safety testing

### Security Considerations
- Input validation for file size and format
- Magic byte verification
- Wrapped unsafe SDK calls
- Memory bounds checking

### Performance Targets
- Thumbnail generation: < 2 seconds for 4K files
- Metadata extraction: < 500ms
- Memory usage: < 100MB for large files
- No UI blocking during processing

### Dependencies
- `thiserror` - Error handling
- `image` - Image processing
- `tokio` - Async runtime
- `bindgen` - FFI bindings generation
- `libc` - System calls
- `tracing` - Logging

### Known Challenges
1. **SDK Licensing**: Must comply with BlackmagicRAW SDK terms
2. **Cross-Platform**: Different linking requirements per OS
3. **Performance**: Large BRAW files (multiple GB)
4. **Magic Bytes**: May vary across camera firmware versions

### References
- Final Implementation Plan: `vendor/final-plan-sonnet-4.md`
- BlackmagicRAW SDK Documentation
- Existing FFmpeg integration patterns in `crates/ffmpeg`
- File extension patterns in `crates/file-ext`

### Development Status
- ✅ Planning phase complete
- ✅ Architecture design finalized
- ⏳ Implementation phase pending
- ⏳ Testing phase pending
- ⏳ Documentation phase pending

### Next Actions
1. Set up development environment with BRAW SDK
2. Create `crates/braw` crate skeleton
3. Implement file detection in `file-ext`
4. Add BRAW metadata extraction
5. Implement thumbnail generation
6. Cross-platform testing and optimization

---

*Last Updated: December 2024*
*For questions or clarifications, refer to the main implementation plan in vendor/final-plan-sonnet-4.md* 