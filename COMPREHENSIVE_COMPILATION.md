# Spacedrive Comprehensive Compilation Guide

## Overview

Spacedrive uses a modular compilation system with feature flags to optimize builds for different platforms and use cases. This guide covers all possible build scenarios including the three critical media processing cases.

## Core Feature Flags

### Main Features (Available in core/Cargo.toml)
- **`ffmpeg`** - Enables FFmpeg-based video processing (required for most video formats)
- **`ai`** - Enables AI-powered features (image labeling, ML models)
- **`mobile`** - Enables mobile-specific functionality and optimizations
- **`heif`** - Enables HEIF/HEIC image format support

### Application-Level Features
- **`ai-models`** - Desktop/server AI functionality (maps to core `ai` feature)
- **`custom-protocol`** - Tauri custom protocol support (desktop default)
- **`devtools`** - Development tools for debugging
- **`assets`** - Static asset serving (server builds)

## Media Processing Build Scenarios

### Case 1: Standard Build (Without BRAW, Without ProRes RAW)
```bash
# Basic build with FFmpeg support only
cargo build --features="ffmpeg,heif"

# For platforms other than macOS, or without BRAW SDK
# Supports: Most video formats via FFmpeg, standard image formats
# Does NOT support: BRAW files, ProRes RAW files
```

### Case 2: BRAW Support Without ProRes RAW
```bash
# macOS with Blackmagic RAW SDK installed
./setup-braw-build.sh
source .braw-env && cargo build --features="ffmpeg,heif"

# Requirements:
# - macOS only
# - Blackmagic RAW SDK installed at /Applications/Blackmagic RAW/
# Supports: BRAW files + standard video/image formats
# Does NOT support: ProRes RAW (would fall back to FFmpeg and fail)
```

### Case 3: ProRes RAW Support Without BRAW
```bash
# macOS build with ProRes RAW support (automatic)
cargo build --features="ffmpeg,heif"

# Requirements:
# - macOS only (uses VideoToolbox framework)
# - No external SDK required
# Supports: ProRes RAW files + standard video/image formats via FFmpeg
# Does NOT support: BRAW files
```

### Case 4: Full Media Support (BRAW + ProRes RAW)
```bash
# macOS with both BRAW SDK and ProRes RAW support
./setup-braw-build.sh
source .braw-env && cargo build --features="ffmpeg,heif"

# Requirements:
# - macOS only
# - Blackmagic RAW SDK installed
# Supports: BRAW files, ProRes RAW files, all FFmpeg formats
```

## Platform-Specific Builds

### macOS (Full Features)
```bash
# Desktop with all media support
cargo build --bin sd-desktop --features="ffmpeg,heif,ai-models"

# With BRAW support
./setup-braw-build.sh && source .braw-env
cargo build --bin sd-desktop --features="ffmpeg,heif,ai-models"
```

### Windows/Linux (No BRAW/ProRes RAW)
```bash
# Standard desktop build
cargo build --bin sd-desktop --features="ffmpeg,heif,ai-models"

# Server build
cargo build --bin sd-server --features="ffmpeg,heif"
```

### Mobile Builds

#### iOS (Limited Media Support)
```bash
# Uses FFmpeg and HEIF, mobile optimizations
cargo build --target aarch64-apple-ios --features="mobile,ffmpeg,heif"
```

#### Android (Minimal Media Support)
```bash
# No FFmpeg support, mobile optimizations only
cargo build --target aarch64-linux-android --features="mobile"
```

## Advanced Codec Support Matrix

| Format | Windows | Linux | macOS | iOS | Android |
|--------|---------|-------|-------|-----|---------|
| Standard Video (H.264, H.265, etc.) | ✅ FFmpeg | ✅ FFmpeg | ✅ FFmpeg | ✅ FFmpeg | ❌ |
| BRAW | ❌ | ❌ | ✅ SDK | ❌ | ❌ |
| ProRes RAW | ❌ | ❌ | ✅ VideoToolbox | ❌ | ❌ |
| Standard Images | ✅ | ✅ | ✅ | ✅ | ✅ |
| HEIF/HEIC | ✅ libheif | ✅ libheif | ✅ Native/libheif | ✅ Native | ❌ |

## Network/Cloud Features (Cannot Be Disabled)

### Always Compiled Components
- **Cloud Services** - P2P networking via `iroh` crate
- **P2P System** - libp2p-based networking (old implementation)
- **Sync System** - Node synchronization functionality

### Dependencies That Cannot Be Removed
- `libp2p` - Peer-to-peer networking
- `iroh` - QUIC-based networking stack  
- `quic-rpc` - RPC over QUIC protocol
- `sd-cloud-schema` - Cloud services schema

**Note**: Currently, there are NO feature flags to disable networking/cloud/P2P functionality. These components are always compiled into all builds.

## Minimal Build Scenarios

### Absolute Minimum (Mobile Android)
```bash
# Smallest possible build
cargo build --target aarch64-linux-android --features="mobile" --no-default-features
```
**Includes**: Basic file operations, networking (cannot be disabled)
**Excludes**: FFmpeg, HEIF, AI, BRAW, ProRes RAW

### Lightweight Server
```bash
# Server without AI
cargo build --bin sd-server --features="ffmpeg" --no-default-features
```
**Includes**: Video processing, networking
**Excludes**: AI models, HEIF, BRAW, ProRes RAW

### Development Build
```bash
# Fast compilation for development
cargo build --features="ffmpeg,heif"
```

## Build Optimization

### Development Profile (Fast Compilation)
```bash
cargo build --profile dev
# - codegen-units = 256 (parallel compilation)
# - debug = 0 (no debug info)
# - opt-level = 0
```

### Release Profile (Optimized)
```bash
cargo build --release
# - lto = true (link-time optimization)
# - opt-level = "s" (optimize for size)
# - codegen-units = 1
```

### Size-Optimized Build
```bash
RUSTFLAGS="-C opt-level=z -C target-cpu=native" cargo build --release
```

## External Dependencies

### Required System Dependencies
- **Rust 1.81+**
- **Basic system libraries** (varies by platform)

### Optional Media Dependencies
- **FFmpeg libraries** (for `ffmpeg` feature)
- **libheif/libheif-sys** (for `heif` feature)
- **Blackmagic RAW SDK** (macOS BRAW support)

### Platform Frameworks (macOS)
- **VideoToolbox** - ProRes RAW support (built-in)
- **CoreFoundation, CoreMedia, CoreGraphics** - Media processing
- **Metal** - GPU acceleration

## Quick Reference Commands

### Desktop Development (macOS with full media)
```bash
./setup-braw-build.sh
source .braw-env && cargo run --bin sd-desktop --features="ffmpeg,heif,ai-models"
```

### Server Deployment (Linux)
```bash
cargo build --release --bin sd-server --features="ffmpeg,heif"
```

### Mobile Development
```bash
# iOS
cargo build --target aarch64-apple-ios --features="mobile,ffmpeg,heif"

# Android
cargo build --target aarch64-linux-android --features="mobile"
```

### Testing Specific Media Support
```bash
# Test BRAW detection
ls /Applications/Blackmagic\ RAW/ && echo "BRAW SDK available"

# Test ProRes RAW support (macOS only)
ffprobe -v quiet -print_format json -show_streams your_prores_raw_file.mov | grep aprn
```

## Architecture Insights

### Media Processing Pipeline
```
File Discovery → Extension Detection → Format-Specific Processing → Thumbnail/Metadata → Database
                                    ↓
                            [FFmpeg | BRAW SDK | VideoToolbox]
```

### Current Limitations
1. **Networking always compiled** - No feature flags to disable P2P/cloud
2. **Platform restrictions** - BRAW/ProRes RAW only on macOS
3. **Mobile limitations** - Android has no FFmpeg support

This modular approach allows optimization for specific platforms and use cases while maintaining compatibility across the Spacedrive ecosystem.