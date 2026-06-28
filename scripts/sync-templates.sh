#!/bin/bash
# Pulls and extracts the latest preset templates from the unraid-nix-templates repo.
set -euo pipefail
TMPZIP="/tmp/nix-templates.zip"
DEST_USR="/usr/local/emhttp/plugins/nix"
DEST_BOOT="/boot/config/plugins/nix"
EXTRACTED="/tmp/unraid-nix-templates-main"

# Best-effort cleanup if the script is interrupted or exits early.
trap 'rm -f "$TMPZIP" "$EXTRACTED" 2>/dev/null || true' EXIT INT TERM

curl -sSf -L -o "$TMPZIP" "https://github.com/UberMetroid/unraid-nix-templates/archive/refs/heads/main.zip"
unzip -q -o "$TMPZIP" -d /tmp
mkdir -p "$DEST_USR/presets" "$DEST_USR/presets_composed" "$DEST_BOOT/presets" "$DEST_BOOT/presets_composed"
cp -rf "$EXTRACTED/presets/"* "$DEST_USR/presets/" 2>/dev/null || true
cp -rf "$EXTRACTED/presets_composed/"* "$DEST_USR/presets_composed/" 2>/dev/null || true
cp -rf "$EXTRACTED/presets/"* "$DEST_BOOT/presets/" 2>/dev/null || true
cp -rf "$EXTRACTED/presets_composed/"* "$DEST_BOOT/presets_composed/" 2>/dev/null || true
rm -rf "$EXTRACTED" "$TMPZIP"
echo "Templates successfully synced and updated."
