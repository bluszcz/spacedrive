# Development Guide

## Building Spacedrive

### Standard Build
```bash
cargo build
```

### BRAW Support Build

For BRAW (Blackmagic RAW) file support:

#### Prerequisites
1. Install Blackmagic RAW SDK and Runtime from [Blackmagic Design](https://www.blackmagicdesign.com/support/family/dv-resolve)
2. Ensure installation at `/Applications/Blackmagic RAW/`

#### Build Steps
```bash
# Setup BRAW environment
./setup-braw-build.sh

# Build with BRAW support
source .braw-env && cargo build

# Run with BRAW support
source .braw-env && cargo run
```

The build system automatically:
- Links BRAW SDK framework
- Compiles C++ wrapper for SDK
- Configures library paths

## Dependencies

### Core Dependencies
- Rust 1.70+
- FFmpeg (for video processing)
- WebP libraries

### Optional Dependencies
- Blackmagic RAW SDK (for BRAW support)
- Platform-specific frameworks (macOS: CoreFoundation, Metal, etc.)

## Testing

```bash
cargo test
```

## Architecture

### Media Processing Pipeline
```
File Discovery → Extension Detection → [FFmpeg|BRAW] → Thumbnail/Metadata → Database
```

BRAW files bypass FFmpeg and use direct SDK calls for maximum compatibility.