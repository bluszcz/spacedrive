## BRAW Integration Notes

- ✅ COMPLETED: Integrated BRAW support into Spacedrive's thumbnail and metadata system
- Developed standalone Rust wrappers for BRAW SDK to generate thumbnails and extract metadata  
- Two key standalone apps created to interact with BRAW SDK via native Rust calls to C++ wrapper
- SDK and runtime located in /Applications folder
- Standalone SDK components used as reference:
  - brawfile-standalone (metadata extraction)
  - brawframe-standalone (thumbnail generation)
- IMPORTANT: FFmpeg does not support BRAW format - custom decoder bypasses FFmpeg entirely

## Implementation Details

### Core Components Added:
1. **BRAW Extension Support**: Added `Braw` to `VideoExtension` enum in file-ext crate
2. **BRAW Decoder Module**: Created `braw_decoder.rs` with FFI bindings to BRAW SDK
3. **BRAW Thumbnailer**: Created `braw_thumbnailer.rs` for frame extraction and WebP generation
4. **BRAW Metadata Extractor**: Created `braw_media_data.rs` to convert BRAW metadata to FFmpeg format
5. **Integration Points**: Updated thumbnailer and media processor to handle BRAW files

### Technical Architecture:
- BRAW files bypass FFmpeg completely and use direct SDK calls
- Thumbnails extracted at frame 0, scaled to 1024x1024, saved as WebP at 60% quality
- Metadata mapped to existing FFmpeg schema for database compatibility
- Memory-safe FFI with proper C++ object cleanup
- Async processing with task-based architecture

### Key Files Modified:
- `crates/file-ext/src/extensions.rs` - Added BRAW extension
- `core/crates/heavy-lifting/src/media_processor/helpers/` - New BRAW modules
- `core/crates/heavy-lifting/src/media_processor/helpers/thumbnailer.rs` - BRAW integration
- `core/crates/heavy-lifting/src/media_processor/helpers/ffmpeg_media_data.rs` - BRAW routing

## Design Principles

- Keep minimalisty, dont repeat yourself, dont write code with warnings

## Development Workflow

- Want script start.sh which sets all variables, builds if not build, and runs the app