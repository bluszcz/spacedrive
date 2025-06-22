# Spacedrive Compilation Features & Flags Analysis

## Feature Flag Overview

Spacedrive uses a sophisticated compilation feature system to enable/disable functionality based on target platform, optional dependencies, and build requirements. This analysis covers all available compilation features and conditional compilation directives.

## Core Feature Flags

### Main Core Features (`core/Cargo.toml`)
- **`default`**: No features enabled by default - allows maximum customization
- **`mobile`**: Enables mobile-specific functionality and optimizations
- **`ai`**: Enables AI-powered features (image labeling, ML models) - depends on `sd-ai` crate
- **`ffmpeg`**: Enables FFmpeg-based media processing for video thumbnails and metadata
- **`heif`**: Enables HEIF/HEIC image format support through libheif

## Media Processing Features

### Heavy Lifting Crate (`core/crates/heavy-lifting/Cargo.toml`)
- **`ffmpeg`**: Controls FFmpeg functionality in media processor
  - Enables video thumbnail generation
  - Adds video metadata extraction
  - Required for video file support

### Images Crate (`crates/images/Cargo.toml`)
- **`heif`**: HEIF/HEIC format support
  - Enables `libheif-rs` and `libheif-sys` dependencies
  - Supports HEIF, HEIC, AVIF formats
  - Uses pre-compiled headers to avoid bindgen

### Media Metadata Crate (`crates/media-metadata/Cargo.toml`)
- **`ffmpeg`**: FFmpeg-based metadata extraction for video files

## Platform-Specific Features

### Desktop Application (`apps/desktop/src-tauri/Cargo.toml`)
Features:
- **`ai-models`**: Enables AI functionality (`sd-core/ai`)
- **`custom-protocol`**: Tauri custom protocol support (default)
- **`devtools`**: Enables Tauri development tools

Default Features: `["custom-protocol"]`

Core Dependencies: `["ffmpeg", "heif"]` - Desktop always includes media support

### Server Application (`apps/server/Cargo.toml`)
Features:
- **`ai-models`**: Enables AI functionality (`sd-core/ai`)
- **`assets`**: Includes static assets
- **`default`**: No default features

Core Dependencies: `["ffmpeg", "heif"]` - Server includes media support

### Mobile Core
#### iOS (`apps/mobile/modules/sd-core/core/Cargo.toml`)
- Features: `["ffmpeg", "heif", "mobile"]`
- Uses `staticlib` for iOS requirements

#### Android (`apps/mobile/modules/sd-core/core/Cargo.toml`)
- Features: `["mobile"]`
- No FFmpeg support on Android (performance/compatibility)
- Uses `cdylib` for JNI compatibility

## AI/ML Features

### AI Crate (`crates/ai/Cargo.toml`)
Platform-specific ML acceleration:

**Windows:**
- DirectML support
- Features: `["directml", "half", "load-dynamic", "ndarray"]`

**Linux:**
- XNNPACK optimization
- Features: `["half", "ndarray", "xnnpack"]`

**macOS/iOS:**
- CoreML acceleration
- Metal GPU support
- Features: `["coreml", "half", "load-dynamic", "ndarray", "xnnpack"]`

**Android** (commented out):
- Would support QNN, NNAPI, ARM compute library
- Features: `["half", "load-dynamic", "qnn", "nnapi", "xnnpack", "acl", "armnn"]`

## Platform-Specific Build Features

### Target-Specific Dependencies

#### macOS (`target_os = "macos"`)
- **BRAW Support**: Blackmagic RAW SDK integration
  - Requires `/Applications/Blackmagic RAW/Blackmagic RAW SDK`
  - Links against Metal, CoreFoundation, CoreMedia frameworks
  - C++ wrapper compilation with `cc::Build`
- **System Integration**: plist parsing, trash support
- **Swift Integration**: swift-rs for native macOS features

#### Linux (`target_os = "linux"`)
- **File System**: inotify file watching
- **GUI**: GTK 3.24, webkit2gtk, wgpu graphics
- **GPU**: WGPU with no default features
- **Desktop**: dbus integration (non-vendored)

#### Windows (`target_os = "windows"`)
- **System APIs**: Win32 File System, IO, Shell APIs
- **AI**: DirectML acceleration

#### iOS (`target_os = "ios"`)
- **Framework**: icrate for Foundation APIs
- **Linking**: Static library compilation
- **Features**: CoreML, Metal support

#### Android (`target_os = "android"`)
- **Logging**: tracing-android
- **Linking**: Dynamic library for JNI
- **Features**: JNI bindings

## Conditional Compilation Patterns

### Feature-Based Compilation
```rust
#[cfg(feature = "ffmpeg")]
// FFmpeg-specific code

#[cfg(feature = "heif")]
// HEIF format support

#[cfg(feature = "ai")]
// AI/ML functionality
```

### Platform-Based Compilation
```rust
#[cfg(target_os = "macos")]
// macOS-specific code

#[cfg(target_os = "linux")]
// Linux-specific code

#[cfg(target_os = "windows")]
// Windows-specific code
```

### Combined Conditions
```rust
#[cfg(all(feature = "ffmpeg", not(target_os = "android")))]
// FFmpeg support except on Android

#[cfg(any(target_os = "macos", target_os = "ios"))]
// Apple platforms
```

## Build Profile Optimizations

### Development Profiles
- **`dev`**: Fast compilation, minimal optimization
  - `codegen-units = 256` (parallel compilation)
  - `debug = 0` (no debug info for speed)
  - `opt-level = 0`
  - `split-debuginfo = "unpacked"` (macOS optimization)

- **`dev-debug`**: Full debugging support
  - `debug = "full"`
  - All other settings optimized for debugging

### Release Profile
- **`release`**: Maximum optimization
  - `codegen-units = 1` (better optimization)
  - `lto = true` (link-time optimization)
  - `opt-level = "s"` (optimize for size)
  - `strip = true` (remove debug symbols)

### Dependency Overrides
- **Build scripts**: `opt-level = 3` (always optimized)
- **Dependencies**: `opt-level = 3`, `incremental = false`

## External System Dependencies

### BRAW Support (macOS only)
- Requires Blackmagic RAW SDK installation
- Links against Apple frameworks: Metal, CoreFoundation, CoreMedia, etc.
- Custom C++ wrapper compilation

### FFmpeg
- Uses `ffmpeg-sys-next = "7.0"`
- Cross-platform video processing
- Excluded from Android builds

### Image Processing
- **Standard formats**: Built-in image crate support
- **HEIF/HEIC**: Optional libheif integration
- **PDF**: pdfium-render for PDF thumbnails
- **SVG**: resvg for vector graphics

## Memory Allocation
- **mimalloc**: Used across desktop and server builds for better performance
- Platform-optimized memory allocation

## Build Performance Tips

1. **Feature Selection**: Only enable needed features to reduce compilation time
2. **Profile Usage**: Use `dev` profile for development, `release` for production
3. **Platform Targeting**: Build only for target platforms to reduce dependencies
4. **AI Features**: AI models significantly increase build time and binary size
5. **BRAW Support**: Only available on macOS with SDK installed

## Recommended Feature Combinations

### Full Desktop Build
```bash
cargo build --features="ffmpeg,heif,ai-models"
```

### Minimal Server Build
```bash
cargo build --features="ffmpeg"
```

### Mobile-Optimized Build
```bash
# iOS
cargo build --features="ffmpeg,heif,mobile"

# Android  
cargo build --features="mobile"
```

### Development Build
```bash
cargo build --profile=dev-debug --features="ffmpeg,heif"
```

This comprehensive feature system allows Spacedrive to be built for different platforms and use cases while maintaining code reusability and optimal performance.