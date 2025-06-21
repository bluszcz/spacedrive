#!/bin/bash

# Spacedrive BRAW Build & Launch Script
# Like a Viking longship preparing for battle!

set -e

echo "🏴‍☠️ Starting Spacedrive with BRAW support..."

# Check if we're on macOS
if [[ "$OSTYPE" != "darwin"* ]]; then
    echo "❌ This script currently only supports macOS"
    exit 1
fi

# Setup BRAW environment
echo "⚔️ Setting up BRAW environment..."
if [ ! -f ".braw-env" ]; then
    echo "🔧 Running BRAW setup..."
    ./setup-braw-build.sh
fi

# Source environment
echo "🌊 Loading BRAW environment..."
source .braw-env

# Check if binary exists and is recent
BINARY_PATH="target/debug/sd-desktop"
NEEDS_BUILD=false

if [ ! -f "$BINARY_PATH" ]; then
    echo "🔨 Binary not found, building..."
    NEEDS_BUILD=true
else
    # Check if any source files are newer than binary
    if find . -name "*.rs" -newer "$BINARY_PATH" | grep -q .; then
        echo "🔨 Source files changed, rebuilding..."
        NEEDS_BUILD=true
    fi
fi

# Build if needed
if [ "$NEEDS_BUILD" = true ]; then
    echo "⚡ Building Spacedrive with BRAW support..."
    cargo build --bin sd-desktop
    
    if [ $? -ne 0 ]; then
        echo "❌ Build failed!"
        exit 1
    fi
    
    echo "✅ Build completed successfully!"
else
    echo "🚀 Binary is up to date, skipping build..."
fi

# Launch Spacedrive
echo "🎯 Launching Spacedrive..."
echo "📁 BRAW files will be processed with native SDK support!"
echo ""

# Set library path and run
export DYLD_LIBRARY_PATH="/Applications/Blackmagic RAW/Blackmagic RAW SDK/Mac/Libraries:$DYLD_LIBRARY_PATH"
exec "$BINARY_PATH"