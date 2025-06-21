# BRAW Support for Spacedrive

## Quick Setup

### Prerequisites
1. Install **Blackmagic RAW SDK** and **Runtime** from [Blackmagic Design](https://www.blackmagicdesign.com/support/family/dv-resolve)
2. Ensure they're installed in `/Applications/Blackmagic RAW/`

### Build & Run

**Quick Start:**
```bash
./start.sh
```

**Manual Steps:**
```bash
# 1. Setup environment (checks SDK installation)
./setup-braw-build.sh

# 2. Build with BRAW support (compiles C++ wrapper automatically)
source .braw-env && cargo build --bin sd-desktop

# 3. Run Spacedrive with BRAW support
source .braw-env && cargo run --bin sd-desktop
```

## What It Does

- **Thumbnails**: Extracts first frame from BRAW files as WebP thumbnails
- **Metadata**: Reads camera settings, resolution, frame rate, etc.
- **Integration**: Works seamlessly with Spacedrive's existing media processing

## Technical Details

### BRAW Processing Pipeline
1. **Detection**: `.braw` files detected via file extension
2. **Decoding**: Direct SDK calls bypass FFmpeg completely
3. **Thumbnails**: Frame extraction → scaling → WebP encoding
4. **Metadata**: Camera data mapped to FFmpeg schema for database storage

### Key Files
- `crates/file-ext/src/extensions.rs` - BRAW extension registration
- `core/crates/heavy-lifting/src/media_processor/helpers/braw_*` - BRAW processing modules
- `setup-braw-build.sh` - Environment setup script

### SDK Dependencies
- BlackmagicRawAPI.dylib
- CoreFoundation, CoreMedia, CoreVideo frameworks
- Metal, MetalKit, AVFoundation frameworks

## Architecture

```
BRAW File → BrawDecoder → [Frame|Metadata] → [Thumbnailer|MediaProcessor] → Database/WebP
```

Unlike other video formats, BRAW files completely bypass FFmpeg and use direct SDK calls for maximum compatibility and performance.

## Troubleshooting

### Build Issues
- Verify SDK installation: `ls /Applications/Blackmagic\ RAW/`
- Run setup script: `./setup-braw-build.sh`
- Check library path: `echo $DYLD_LIBRARY_PATH`

### Runtime Issues
- Source environment: `source .braw-env`
- Check BRAW files are accessible
- Verify SDK version compatibility

---
*Implementation based on standalone brawfile/brawframe examples*