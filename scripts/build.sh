#!/usr/bin/env bash
# scripts/build.sh — build the nix-helper binary.
#
# Usage:
#   ./scripts/build.sh           # debug build
#   ./scripts/build.sh --release # release build (stripped, LTO, ~1 MB)
#
# The output binary is placed at the repo root as `nix-helper`, which is
# what `nix.plg` references during install. The path matches the
# `.gitignore` entry, so the binary is never committed.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

if [[ "${1:-}" == "--release" ]]; then
    echo "Building release binary (stripped + LTO)..."
    cargo build --release
    cp target/release/nix-helper ./nix-helper
    echo "Built $(ls -la ./nix-helper | awk '{print $5}') bytes → ./nix-helper"
else
    echo "Building debug binary..."
    cargo build
    cp target/debug/nix-helper ./nix-helper
    echo "Built $(ls -la ./nix-helper | awk '{print $5}') bytes → ./nix-helper"
fi