#!/usr/bin/env bash

# ----------------------------------------------------------------------------
#  spacedrive_bluszcz.sh
#  BRAW Development Script - Always builds with full BlackmagicRAW support
# ----------------------------------------------------------------------------
#  • Detects the Blackmagic RAW SDK in its standard macOS location
#  • Sets the BRAW_SDK_PATH env var for the build.rs script
#  • ALWAYS compiles Spacedrive with full BRAW features (braw,with-sdk,native-ffi)
#  • Pass-throughs any extra CLI args to `cargo run` (e.g. --release)
# ----------------------------------------------------------------------------

set -euo pipefail

# Workspace root (directory where this script lives)
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# -----------------------------------------------------------------------------
# Blackmagic RAW SDK path (macOS default)
DEFAULT_SDK="/Applications/Blackmagic RAW/Blackmagic RAW SDK"

if [[ -z "${BRAW_SDK_PATH:-}" ]]; then
  if [[ -d "$DEFAULT_SDK" ]]; then
    export BRAW_SDK_PATH="$DEFAULT_SDK"
  else
    echo "[BRAW DEV] ERROR: Blackmagic RAW SDK not found."
    echo "Install it from https://www.blackmagicdesign.com/ and/or set BRAW_SDK_PATH manually."
    echo "This script requires the full SDK for native BRAW support."
    exit 1
  fi
fi

echo "[BRAW DEV] Using BlackmagicRAW SDK at: $BRAW_SDK_PATH"

# Set runtime framework paths for macOS
if [[ "$(uname)" == "Darwin" ]]; then
  export DYLD_FRAMEWORK_PATH="$BRAW_SDK_PATH/Mac/Libraries:${DYLD_FRAMEWORK_PATH:-}"
  export DYLD_LIBRARY_PATH="$BRAW_SDK_PATH/Mac/Libraries:${DYLD_LIBRARY_PATH:-}"
  echo "[BRAW DEV] Set runtime framework paths for BlackmagicRAW"
fi

# -----------------------------------------------------------------------------
# Generate Prisma client code (both async and sync variants)
# This step is required because the generated modules are git-ignored.
# If they already exist, the generator will exit quickly.

echo "[BRAW DEV] Generating Prisma clients …"

# Try pnpm first; if it fails, fallback to cargo alias.
if command -v pnpm >/dev/null 2>&1; then
  set +e
  pnpm exec prisma generate --schema core/prisma/schema.prisma
  STATUS=$?
  set -e
  if [ $STATUS -ne 0 ]; then
    echo "[BRAW DEV] pnpm prisma generation failed – falling back to cargo alias."
    cargo prisma generate --schema core/prisma/schema.prisma
  fi
else
  echo "[BRAW DEV] pnpm not installed – using cargo alias."
  cargo prisma generate --schema core/prisma/schema.prisma
fi

echo "[BRAW DEV] Prisma client generation complete."

# -----------------------------------------------------------------------------
# macOS dev convenience: create placeholder Spacedrive.framework so that the
# tauri build script doesn't fail in debug mode. Real release builds should
# provide a proper framework via CI pipeline.

if [[ "$(uname)" == "Darwin" ]]; then
  for FRAMEWORK_PATH in \
    ".deps/Spacedrive.framework" \
    "apps/desktop/.deps/Spacedrive.framework" \
    "apps/desktop/src-tauri/.deps/Spacedrive.framework" \
    "apps/desktop/src-tauri/../../.deps/Spacedrive.framework"
  do
    if [[ ! -d "$FRAMEWORK_PATH" ]]; then
      echo "[BRAW DEV] Creating dummy Spacedrive.framework at $FRAMEWORK_PATH (debug build workaround)."
      mkdir -p "$FRAMEWORK_PATH/Versions/A"
      touch "$FRAMEWORK_PATH/Versions/A/Spacedrive" # empty placeholder binary
      ln -sf Versions/A/Spacedrive "$FRAMEWORK_PATH/Spacedrive"
    fi
  done
fi

# -----------------------------------------------------------------------------
# Build the frontend (Vite/React) before running the backend

echo "[BRAW DEV] Building frontend (Vite) …"
cd "$ROOT_DIR/apps/desktop"
if command -v pnpm >/dev/null 2>&1; then
  pnpm install
  pnpm build
else
  npm install
  npm run build
fi
cd "$ROOT_DIR"
echo "[BRAW DEV] Frontend build complete."

# -----------------------------------------------------------------------------
# Build & run with FULL BRAW support (ALWAYS)
cd "$ROOT_DIR"

# BRAW Development Mode: Always compile with full BRAW features
BRAW_FEATURES="braw,with-sdk,native-ffi"

echo "[BRAW DEV] Building Spacedrive with FULL BRAW support (features: $BRAW_FEATURES) …"
echo "[BRAW DEV] This build includes:"
echo "  • BRAW file detection and indexing"
echo "  • Native BlackmagicRAW SDK integration"
echo "  • Real BRAW frame extraction and thumbnails"
echo "  • Full camera metadata support"

# All args (e.g. --release) are passed through to cargo run
cargo run --features "$BRAW_FEATURES" "$@"

# -----------------------------------------------------------------------------
