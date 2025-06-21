#!/bin/bash

# BRAW SDK Build Setup Script for Spacedrive
# Sets up environment for building with Blackmagic RAW SDK

set -e

echo "🔥 Setting up BRAW SDK environment for Spacedrive..."

# Check if running on macOS
if [[ "$OSTYPE" != "darwin"* ]]; then
    echo "❌ This script currently only supports macOS"
    exit 1
fi

# BRAW SDK Paths
BRAW_SDK_PATH="/Applications/Blackmagic RAW/Blackmagic RAW SDK/Mac/Libraries"
BRAW_RUNTIME_PATH="/Applications/Blackmagic RAW"

echo "🔍 Checking BRAW SDK installation..."

# Check if BRAW SDK exists
if [ ! -d "$BRAW_SDK_PATH" ]; then
    echo "❌ BRAW SDK not found at $BRAW_SDK_PATH"
    echo "Please install Blackmagic RAW SDK from:"
    echo "https://www.blackmagicdesign.com/support/family/dv-resolve"
    exit 1
fi

# Check if runtime exists
if [ ! -d "$BRAW_RUNTIME_PATH" ]; then
    echo "❌ BRAW Runtime not found at $BRAW_RUNTIME_PATH"
    echo "Please install Blackmagic RAW from:"
    echo "https://www.blackmagicdesign.com/support/family/dv-resolve"
    exit 1
fi

echo "✅ BRAW SDK found at: $BRAW_SDK_PATH"
echo "✅ BRAW Runtime found at: $BRAW_RUNTIME_PATH"

# Set environment variables
export BRAW_SDK_PATH="$BRAW_SDK_PATH"
export BRAW_RUNTIME_PATH="$BRAW_RUNTIME_PATH"
export DYLD_LIBRARY_PATH="$BRAW_SDK_PATH:$DYLD_LIBRARY_PATH"

echo "🔧 Environment variables set:"
echo "   BRAW_SDK_PATH=$BRAW_SDK_PATH"
echo "   BRAW_RUNTIME_PATH=$BRAW_RUNTIME_PATH"
echo "   DYLD_LIBRARY_PATH=$DYLD_LIBRARY_PATH"

# Check for required frameworks
echo "🔍 Checking required BRAW frameworks..."
REQUIRED_FRAMEWORKS=(
    "BlackmagicRawAPI.framework"
)

for framework in "${REQUIRED_FRAMEWORKS[@]}"; do
    if [ -d "$BRAW_SDK_PATH/$framework" ]; then
        echo "✅ Found: $framework"
    else
        echo "❌ Missing: $framework"
        exit 1
    fi
done

echo ""
echo "🎯 BRAW SDK setup complete!"
echo ""
echo "To build Spacedrive with BRAW support:"
echo "1. Source this script: source ./setup-braw-build.sh"
echo "2. Run: cargo build"
echo ""
echo "To run Spacedrive with BRAW support:"
echo "1. Export library path: export DYLD_LIBRARY_PATH=\"$BRAW_SDK_PATH:\$DYLD_LIBRARY_PATH\""
echo "2. Run your Spacedrive binary"
echo ""

# Save environment to a file for easy sourcing
cat > .braw-env << EOF
export BRAW_SDK_PATH="$BRAW_SDK_PATH"
export BRAW_RUNTIME_PATH="$BRAW_RUNTIME_PATH"
export DYLD_LIBRARY_PATH="$BRAW_SDK_PATH:\$DYLD_LIBRARY_PATH"
EOF

echo "📝 Environment saved to .braw-env (source it with: source .braw-env)"