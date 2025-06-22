# Spacedrive Compilation Features Analysis

## Network/Cloud/P2P Features That Can Be Disabled

### Core Features (from core/Cargo.toml)
- **`ffmpeg`** - Controls FFmpeg-based media processing functionality
- **`ai`** - Controls AI model functionality (depends on `sd-ai` crate)
- **`mobile`** - Special mobile compilation mode that may affect networking features
- **`heif`** - HEIF image format support

### Desktop App Features (from apps/desktop/src-tauri/Cargo.toml)
- **`ai-models`** - Enables AI model functionality (maps to `sd-core/ai`)
- **`custom-protocol`** - Tauri custom protocol support (default enabled)
- **`devtools`** - Development tools support

### Server App Features (from apps/server/Cargo.toml)
- **`ai-models`** - Enables AI model functionality (maps to `sd-core/ai`)
- **`assets`** - Asset serving functionality

### Mobile Compilation (apps/mobile/modules/sd-core/core/Cargo.toml)
Mobile builds use `default-features = false` and only enable:
- `mobile`
- `ffmpeg` 
- `heif` (iOS only)

## Networking/Cloud Components (Always Compiled)

### Cloud Services (core/crates/cloud-services/)
- **Always compiled** - No feature flags found
- Contains P2P networking functionality via `iroh` crate
- Includes sync, client, and P2P runner modules
- Uses QUIC/RPC transport layer

### P2P System (crates/old-p2p/)
- **Always compiled** - No optional compilation found
- Uses libp2p with features: autonat, dcutr, macros, noise, quic, relay, serde, tokio, yamux
- Includes block transfer, protocol definitions, and tunneling

### Sync System (crates/sync/ and core/crates/sync/)
- **Always compiled** - No feature flags to disable
- Core synchronization functionality between nodes

## Key Dependencies That Cannot Be Disabled

1. **libp2p** - Peer-to-peer networking library (always included)
2. **iroh** - QUIC-based networking stack (always included in cloud services)
3. **quic-rpc** - RPC over QUIC protocol (always included)
4. **sd-cloud-schema** - Cloud services schema (workspace dependency)

## Minimal Build Considerations

For truly minimal builds wanting to exclude networking:
- Mobile builds already use `default-features = false` but still include cloud services
- No current feature flags exist to completely disable P2P or cloud functionality
- The Node structure hardcodes `cloud_services` and `p2p` managers
- All networking dependencies are workspace-level dependencies

## Current Architecture Limitations

The current architecture does not support completely network-free builds because:
1. Core Node struct requires CloudServices and P2PManager instances
2. No conditional compilation around networking modules
3. Workspace dependencies include networking crates unconditionally

To create truly offline/minimal builds, significant refactoring would be needed to make networking components optional at the architectural level.

## Media Processing Build Scenarios

### Three Critical Build Cases

#### Case 1: Standard Build (No BRAW, No ProRes RAW)
```bash
# Basic build with FFmpeg support only
cargo build --features="ffmpeg,heif"
# Supports: Standard video formats via FFmpeg, images
# Missing: BRAW files, ProRes RAW files will fail with FFmpeg errors
```

#### Case 2: BRAW Support (macOS with SDK)
```bash
# Requires Blackmagic RAW SDK at /Applications/Blackmagic RAW/
./setup-braw-build.sh
source .braw-env && cargo build --features="ffmpeg,heif"
# Supports: BRAW files + standard formats
# Note: ProRes RAW still fails without VideoToolbox integration
```

#### Case 3: ProRes RAW Support (macOS VideoToolbox)
```bash
# Uses native VideoToolbox APIs (no external SDK needed)
cargo build --features="ffmpeg,heif"  # ProRes RAW automatically enabled on macOS
# Supports: ProRes RAW files + standard formats via VideoToolbox
# Note: BRAW files not supported without SDK
```

#### Case 4: Full Media Support (Both BRAW + ProRes RAW)
```bash
# macOS with both BRAW SDK and ProRes RAW support
./setup-braw-build.sh
source .braw-env && cargo build --features="ffmpeg,heif"
# Supports: BRAW files, ProRes RAW files, all FFmpeg formats
```

## ProRes RAW Integration Details

### Technical Implementation
- **Detection**: Uses `ffprobe` to identify `aprn` codec tag
- **Processing**: Bypasses FFmpeg entirely, uses `AVAssetImageGenerator`
- **Thumbnails**: Extracts frames at `kCMTimeZero`, converts to WebP
- **Fallback**: Falls back to FFmpeg for non-ProRes RAW MOV files

### Files Modified for ProRes RAW Support
- `core/crates/heavy-lifting/wrapper_prores_raw.cpp` - VideoToolbox C++ wrapper
- `core/crates/heavy-lifting/src/media_processor/helpers/prores_raw_decoder.rs` - Rust FFI
- `core/crates/heavy-lifting/src/media_processor/helpers/prores_raw_thumbnailer.rs` - Thumbnail generation
- `core/crates/heavy-lifting/src/media_processor/helpers/thumbnailer.rs` - Integration logic
- `core/crates/heavy-lifting/build.rs` - VideoToolbox framework linking