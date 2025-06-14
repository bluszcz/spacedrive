# Adding BRAW File Support to Spacedrive

## Overview
This document outlines the steps required to add BlackmagicRAW (BRAW) file support to the Spacedrive file explorer. BRAW is a high-quality codec developed by Blackmagic Design for their cameras and is widely used in professional video production.

## Background
- **BRAW Format**: BlackmagicRAW is a professional video codec offering high quality with manageable file sizes
- **SDK Available**: Blackmagic provides a free SDK for developers
- **Current State**: Spacedrive currently supports various video formats via FFmpeg, but BRAW may require specialized handling

## Resources
- [Blackmagic RAW Official Page](https://www.blackmagicdesign.com/products/blackmagicraw)
- [Blackmagic RAW SDK](https://www.blackmagicdesign.com/developer/products/braw/sdk-and-software)
- [SDK Documentation](https://documents.blackmagicdesign.com/DeveloperManuals/BlackmagicRAW-SDK.pdf?_v=1668672010000)

## Implementation Steps

### Phase 1: Research and Analysis
1. **Download and Study the BRAW SDK**
   - Download the BlackmagicRAW SDK from the official website
   - Study the API documentation and sample code
   - Understand file structure, metadata handling, and decoding capabilities
   - Identify licensing requirements and redistribution terms

2. **Analyze Spacedrive Architecture**
   - Study the `crates/file-ext` module to understand file type detection
   - Examine `crates/media-metadata` for metadata extraction patterns
   - Review `crates/ffmpeg` implementation for video processing reference
   - Understand how thumbnails are generated for video files

### Phase 2: File Type Recognition
1. **Add BRAW Extension to File Type System**
   - Modify `crates/file-ext/src/extensions.rs`
   - Add `Braw` variant to `VideoExtension` enum
   - Add magic bytes detection for BRAW files (if available)
   - Update tests to include BRAW file detection

2. **Magic Bytes Investigation**
   - Research BRAW file headers/magic bytes
   - Test with sample BRAW files to identify consistent patterns
   - Implement magic bytes detection in the file extension system

### Phase 3: SDK Integration
1. **Create BRAW Processing Crate**
   - Create new crate: `crates/braw` 
   - Integrate BlackmagicRAW SDK
   - Handle cross-platform compilation (Windows, macOS, Linux)
   - Manage dynamic library loading and linking

2. **Implement Core BRAW Functionality**
   ```rust
   // Example structure for crates/braw/src/lib.rs
   pub struct BrawFile {
       // SDK handle and file info
   }
   
   impl BrawFile {
       pub fn open(path: &Path) -> Result<Self, BrawError>;
       pub fn get_metadata(&self) -> Result<BrawMetadata, BrawError>;
       pub fn extract_frame(&self, frame_num: u32) -> Result<Image, BrawError>;
       pub fn get_thumbnail(&self) -> Result<Image, BrawError>;
   }
   
   pub struct BrawMetadata {
       pub resolution: (u32, u32),
       pub frame_rate: f64,
       pub duration: Duration,
       pub codec_info: String,
       pub color_space: String,
       // ... other BRAW-specific metadata
   }
   ```

### Phase 4: Metadata Extraction
1. **Extend Media Metadata System**
   - Add BRAW support to `crates/media-metadata`
   - Extract technical metadata (resolution, frame rate, duration, etc.)
   - Extract camera metadata (ISO, shutter speed, color temperature, etc.)
   - Handle BRAW-specific metadata fields

2. **Update Database Schema**
   - Review Prisma schema for video metadata storage
   - Add BRAW-specific fields if necessary
   - Handle migration for existing databases

### Phase 5: Thumbnail Generation
1. **Implement BRAW Thumbnail Extraction**
   - Use BRAW SDK to extract preview frames
   - Integrate with existing thumbnail system
   - Ensure consistent thumbnail sizes and formats
   - Optimize performance for large BRAW files

2. **Update Thumbnail Pipeline**
   - Modify thumbnail generation logic to handle BRAW files
   - Ensure fallback mechanisms work correctly
   - Test thumbnail caching and storage

### Phase 6: Preview and Playback (Optional)
1. **Basic Frame Extraction**
   - Implement frame-by-frame extraction for preview
   - Consider memory management for large files
   - Integrate with existing media preview system

2. **Advanced Features (Future)**
   - Color grading controls
   - RAW parameter adjustment
   - Timeline scrubbing
   - Proxy generation

### Phase 7: Build System and Distribution
1. **Cross-Platform Compilation**
   - Ensure BRAW SDK works on all target platforms
   - Handle dynamic library distribution
   - Update build scripts and CI/CD pipelines

2. **Licensing and Distribution**
   - Review BlackmagicRAW SDK license terms
   - Ensure compliance with redistribution requirements
   - Update project documentation and dependencies

### Phase 8: Testing and Quality Assurance
1. **Unit Tests**
   - Test file type detection
   - Test metadata extraction
   - Test thumbnail generation
   - Test error handling

2. **Integration Tests**
   - Test with various BRAW file versions
   - Test with different camera models
   - Performance testing with large files
   - Memory usage optimization

3. **User Testing**
   - Create test BRAW files or obtain samples
   - Test in different Spacedrive configurations
   - Validate user experience and performance

## Technical Considerations

### Performance
- BRAW files can be very large (several GB)
- SDK operations should be asynchronous
- Consider caching strategies for metadata and thumbnails
- Memory management is crucial for video processing

### Error Handling
- Handle corrupted BRAW files gracefully
- Provide meaningful error messages
- Implement proper fallbacks when SDK is unavailable

### Platform Support
- Ensure BlackmagicRAW SDK supports all Spacedrive target platforms
- Handle cases where SDK is not available
- Consider optional feature flags for BRAW support

### Security
- Validate BRAW files before processing
- Ensure SDK calls are memory-safe
- Handle potential malformed file attacks

## File Structure Changes

```
spacedrive/
├── crates/
│   ├── file-ext/
│   │   └── src/
│   │       └── extensions.rs          # Add BRAW extension
│   ├── media-metadata/
│   │   └── src/
│   │       ├── braw/                  # New BRAW metadata module
│   │       │   ├── mod.rs
│   │       │   └── metadata.rs
│   │       └── lib.rs                 # Update to include BRAW
│   ├── braw/                          # New crate for BRAW SDK integration
│   │   ├── Cargo.toml
│   │   ├── build.rs                   # SDK linking and setup
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── sdk.rs                 # SDK wrapper
│   │       ├── metadata.rs            # Metadata extraction
│   │       ├── thumbnail.rs           # Thumbnail generation
│   │       └── error.rs               # Error types
│   └── images/                        # Update for BRAW thumbnails
└── core/
    └── crates/
        └── media-metadata/            # Update core integration
```

## Development Timeline

### Week 1-2: Research and Setup
- Download and study BRAW SDK
- Set up development environment
- Create initial crate structure

### Week 3-4: Basic Integration
- Implement file type detection
- Create basic SDK wrapper
- Add metadata extraction

### Week 5-6: Thumbnail Support
- Implement thumbnail extraction
- Integration with existing systems
- Performance optimization

### Week 7-8: Testing and Polish
- Comprehensive testing
- Documentation
- Code review and refinement

## Potential Challenges

1. **SDK Complexity**: The BlackmagicRAW SDK may have complex initialization or usage patterns
2. **Platform Differences**: Different SDK versions or behaviors across platforms
3. **Performance**: Large BRAW files may require careful memory and performance management
4. **Licensing**: Ensure compliance with BlackmagicRAW SDK terms
5. **Dependencies**: Managing SDK distribution and dynamic linking

## Future Enhancements

1. **Color Grading**: Basic color grading controls in preview
2. **Proxy Generation**: Create smaller proxy files for better performance
3. **Batch Processing**: Bulk metadata extraction and thumbnail generation
4. **Integration with DaVinci Resolve**: Potential integration or workflow features
5. **Timeline Preview**: Scrubbing through BRAW footage

## Contribution Guidelines

When implementing this feature:

1. Follow Spacedrive's coding standards and guidelines
2. Write comprehensive tests for all new functionality
3. Update documentation for new APIs and features
4. Consider backwards compatibility
5. Ensure error handling is robust and user-friendly
6. Performance test with large BRAW files
7. Submit PRs in logical chunks following the phases above

## Resources for Contributors

- [Spacedrive Contributing Guide](https://github.com/spacedriveapp/spacedrive/blob/main/CONTRIBUTING.md)
- [Rust FFI Guidelines](https://doc.rust-lang.org/nomicon/ffi.html)
- [Video Processing Best Practices](https://github.com/rust-av)
- [BlackmagicRAW Developer Community](https://forum.blackmagicdesign.com/viewforum.php?f=21)

---

*Last Updated: June 14, 2025*
*Status: Planning Phase*
