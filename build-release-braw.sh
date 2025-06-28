#!/bin/bash

# 🔥 Spacedrive Release Build Script with BRAW & ProRes RAW Support
# Like a Viking forging the finest blade for conquest!

set -e

echo "🎯 Building Spacedrive Release with BRAW & ProRes RAW support..."
echo "🔨 Optimizing for battle-ready performance..."

# Setup BRAW environment with release optimizations
echo "⚡ Loading BRAW release environment..."
source .braw-env-release

# Verify BRAW SDK is available
if [ ! -d "$BRAW_SDK_PATH" ]; then
    echo "❌ BRAW SDK not found at $BRAW_SDK_PATH"
    echo "🏃 Run ./setup-braw-build.sh first!"
    exit 1
fi

echo "✅ BRAW SDK found at $BRAW_SDK_PATH"
echo "✅ BRAW Runtime at $BRAW_RUNTIME_PATH"

# Build with maximum optimization
echo "⚔️  Building optimized release with BRAW + ProRes RAW..."
pnpm tauri build --target x86_64-apple-darwin

echo "🏆 Build completed! Your optimized .app bundle awaits in:"
echo "📦 target/release/bundle/dmg/"

# Open the treasure vault
echo "🗂️  Opening distribution folder..."
pnpm --filter @sd/desktop -- dmg

echo "🎉 Victory! Your optimized Spacedrive.app is ready for deployment!"
echo "🦌 Like a swift deer leaping through the forest, your app shall run with grace!"