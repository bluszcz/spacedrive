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
# Build & run
cd "$ROOT_DIR"

# You can pass "--release" or additional cargo flags after the script name.
cargo run --features braw,with-sdk,native-ffi "$@" 