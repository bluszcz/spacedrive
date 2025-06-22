# Spacedrive Compilation Guide

## Overview

Spacedrive uses a modular compilation system with feature flags to optimize builds for different platforms and use cases. Like a Viking longship with removable parts, you can enable/disable features based on your needs.

## Core Features

### Media Processing Features

#### FFmpeg Support
```bash
# Enable FFmpeg for video processing (default for desktop)
cargo build --features ffmpeg

# Disable FFmpeg for lighter builds
cargo build
```

#### HEIF Image Support
```bash
# Enable modern HEIF/HEIC image format support
cargo build --features heif
```

#### BRAW Support (macOS only)
```bash
# Requires Blackmagic RAW SDK installation
./setup-braw-build.sh
source .braw-env && cargo build
```

### AI/ML Features
```bash
# Enable AI features for object recognition, etc.
cargo build --features ai
```

### Platform-Specific Builds

#### Desktop (Full Features)
```bash
# macOS/Windows/Linux with all features
cargo build --bin sd-desktop --features "ffmpeg,heif"
```

#### Server (Headless)
```bash
# Minimal server build without GUI dependencies
cargo build --bin sd-server
```

#### Mobile
```bash
# iOS build (requires Xcode)
cargo build --target aarch64-apple-ios --features mobile

# Android build (requires NDK)
cargo build --target aarch64-linux-android --features mobile
```

## Build Profiles

### Development (Fast compilation)
```bash
cargo build
```

### Release (Optimized)
```bash
cargo build --release
```

### Size-Optimized
```bash
RUSTFLAGS="-C opt-level=z" cargo build --release
```

## Platform-Specific Configuration

### macOS
- **Metal GPU acceleration**: Automatically enabled
- **BRAW support**: Requires SDK installation
- **HEIF support**: Native via frameworks

### Windows
- **DirectML AI**: Automatically enabled when available
- **Windows-specific file operations**: Enabled via conditional compilation

### Linux
- **XNNPACK AI**: Used for AI inference
- **System integration**: Varies by distribution

## Feature Combinations

### Minimal Build (Server)
```bash
cargo build --bin sd-server --no-default-features
```

### Full Desktop Build
```bash
cargo build --bin sd-desktop --features "ffmpeg,heif,ai"
```

### Development Build with BRAW
```bash
source .braw-env && cargo build --features "ffmpeg,heif"
```

## Cross-Compilation

### iOS
```bash
rustup target add aarch64-apple-ios
cargo build --target aarch64-apple-ios --features mobile
```

### Android
```bash
rustup target add aarch64-linux-android
cargo build --target aarch64-linux-android --features mobile
```

### ARM64 Linux
```bash
rustup target add aarch64-unknown-linux-gnu
cargo build --target aarch64-unknown-linux-gnu
```

## Build Dependencies

### Core Dependencies (Always Required)
- Rust 1.81+
- Basic system libraries

### Optional Dependencies
- **FFmpeg**: For video processing (`ffmpeg` feature)
- **HEIF libraries**: For HEIF support (`heif` feature)  
- **Blackmagic RAW SDK**: For BRAW support (macOS)
- **GPU libraries**: Metal (macOS), DirectML (Windows), Vulkan (Linux)

## Optimization Flags

### Link-Time Optimization
```bash
RUSTFLAGS="-C lto=fat" cargo build --release
```

### CPU-Specific Optimization
```bash
RUSTFLAGS="-C target-cpu=native" cargo build --release
```

### Debug Information
```bash
# Include debug info in release
cargo build --release --config profile.release.debug=true
```

## Troubleshooting

### Build Failures
1. **Missing dependencies**: Install platform-specific libraries
2. **Feature conflicts**: Check feature combinations
3. **Outdated Rust**: Update to latest stable

### Performance Issues
1. **Use release builds**: `cargo build --release`
2. **Enable LTO**: Add link-time optimization
3. **Platform optimization**: Use native CPU features

### Platform-Specific Issues
- **macOS**: Ensure Xcode command line tools installed
- **Windows**: Install Visual Studio Build Tools
- **Linux**: Install development packages (libssl-dev, etc.)

## Quick Start Scripts

### Desktop Development
```bash
./start.sh  # Builds and runs with all desktop features
```

### Server Deployment
```bash
cargo build --release --bin sd-server
```

### Cross-Platform Build
```bash
# Build for all platforms
cargo build --release --bin sd-desktop --target x86_64-pc-windows-gnu
cargo build --release --bin sd-desktop --target x86_64-apple-darwin
cargo build --release --bin sd-desktop --target x86_64-unknown-linux-gnu
```

## Advanced Configuration

### Custom Feature Sets
Create `.cargo/config.toml`:
```toml
[build]
target-dir = "target"

[profile.release]
opt-level = 3
lto = true
codegen-units = 1
panic = "abort"
```

### Environment Variables
```bash
# Optimize for size
export RUSTFLAGS="-C opt-level=z -C target-cpu=native"

# Link statically
export RUSTFLAGS="-C target-feature=+crt-static"
```

This modular approach ensures you only compile what you need, keeping builds fast and binaries lean!