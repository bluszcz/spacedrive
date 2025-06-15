#!/usr/bin/env bash

# ----------------------------------------------------------------------------
#  spacedrive_bluszcz.sh
#  Helper script to build & run Spacedrive with full Blackmagic-RAW support
# ----------------------------------------------------------------------------
#  • Detects the Blackmagic RAW SDK in its standard macOS location
#  • Sets the BRAW_SDK_PATH env var for the build.rs script
#  • Compiles Spacedrive with native FFI bindings
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
    echo "[spacedrive_bluszcz] ERROR: Blackmagic RAW SDK not found."
    echo "Install it from https://www.blackmagicdesign.com/ and/or set BRAW_SDK_PATH manually."
    exit 1
  fi
fi

echo "[spacedrive_bluszcz] Using SDK at: $BRAW_SDK_PATH"

# -----------------------------------------------------------------------------
# Generate Prisma client code (both async and sync variants)
# This step is required because the generated modules are git-ignored.
# If they already exist, the generator will exit quickly.

echo "[spacedrive_bluszcz] Generating Prisma clients …"

# Try pnpm first; if it fails, fallback to cargo alias.
if command -v pnpm >/dev/null 2>&1; then
  set +e
  pnpm exec prisma generate --schema core/prisma/schema.prisma
  STATUS=$?
  set -e
  if [ $STATUS -ne 0 ]; then
    echo "[spacedrive_bluszcz] pnpm prisma generation failed – falling back to cargo alias."
    cargo prisma generate --schema core/prisma/schema.prisma
  fi
else
  echo "[spacedrive_bluszcz] pnpm not installed – using cargo alias."
  cargo prisma generate --schema core/prisma/schema.prisma
fi

echo "[spacedrive_bluszcz] Prisma client generation complete."

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
      echo "[spacedrive_bluszcz] Creating dummy Spacedrive.framework at $FRAMEWORK_PATH (debug build workaround)."
      mkdir -p "$FRAMEWORK_PATH/Versions/A"
      touch "$FRAMEWORK_PATH/Versions/A/Spacedrive" # empty placeholder binary
      ln -sf Versions/A/Spacedrive "$FRAMEWORK_PATH/Spacedrive"
    fi
  done
fi

# -----------------------------------------------------------------------------
# Build the frontend (Vite/React) before running the backend

echo "[spacedrive_bluszcz] Building frontend (Vite) …"
cd "$ROOT_DIR/apps/desktop"
if command -v pnpm >/dev/null 2>&1; then
  pnpm install
  pnpm build
else
  npm install
  npm run build
fi
cd "$ROOT_DIR"
echo "[spacedrive_bluszcz] Frontend build complete."

# -----------------------------------------------------------------------------
# Build & run
cd "$ROOT_DIR"

# Determine if user wants full native FFI
NATIVE_FFI_FLAG=""
if [[ " $* " == *" --native-ffi "* ]]; then
  NATIVE_FFI_FLAG=",native-ffi"
  # remove the flag from positional args
  set -- ${@/--native-ffi/}
fi

echo "[spacedrive_bluszcz] Building Spacedrive (features: braw,with-sdk${NATIVE_FFI_FLAG}) …"

# Remaining args (e.g. --release) are passed through
cargo run --features "braw,with-sdk${NATIVE_FFI_FLAG}" "$@"

# -----------------------------------------------------------------------------