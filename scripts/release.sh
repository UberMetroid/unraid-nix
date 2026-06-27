#!/usr/bin/env bash
# scripts/release.sh — tag-and-release flow for nix-helper.
#
# Prerequisites:
#   * `gh` CLI installed and authenticated (gh auth status).
#   * Clean working tree (`git status --porcelain` is empty).
#   * On the `main` branch, up-to-date with `origin/main`.
#
# Usage:
#   ./scripts/release.sh                # auto-bump patch version
#   ./scripts/release.sh 2026.07.01.1   # explicit plg version
#
# What it does:
#   1. Bumps the version in nix.plg (ENTITY version).
#   2. Builds the release binary into ./nix-helper.
#   3. Creates a git tag matching the plg version (e.g. v2026.07.01.1).
#   4. Pushes the tag (which triggers .github/workflows/release.yml).
#   5. Creates a GitHub Release and uploads nix-helper as an asset.
#
# The .github/workflows/release.yml workflow does its own build, so the
# uploaded binary in step 5 is just a convenience for the WebUI install
# path that references raw.githubusercontent.com URLs.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

if [[ -n "$(git status --porcelain)" ]]; then
    echo "ERROR: working tree is dirty. Commit or stash changes first." >&2
    exit 1
fi

if ! command -v gh >/dev/null; then
    echo "ERROR: gh CLI not found. Install from https://cli.github.com/." >&2
    exit 1
fi

VERSION="${1:-}"
if [[ -z "$VERSION" ]]; then
    # Bump patch: parse the current version from nix.plg ENTITY.
    CURRENT=$(grep -oP '<!ENTITY\s+version\s+"\K[^"]+' nix.plg)
    # Current format is YYYY.MM.DD.NN — bump NN.
    if [[ "$CURRENT" =~ ^([0-9]{4}\.[0-9]{2}\.[0-9]{2})\.([0-9]+)$ ]]; then
        DATE="${BASH_REMATCH[1]}"
        N="${BASH_REMATCH[2]}"
        VERSION="${DATE}.$((N + 1))"
    else
        echo "ERROR: could not parse current version '$CURRENT'." >&2
        exit 1
    fi
fi

echo "Releasing version: $VERSION"

# 1. Update nix.plg ENTITY version.
sed -i "s|<!ENTITY\s\+version\s\+\"[^\"]*\">|<!ENTITY version \"$VERSION\">|" nix.plg

# 2. Build release binary.
"$(dirname "$0")/build.sh" --release

# 3. Commit, tag, push.
git add nix.plg nix-helper
git commit -m "release: $VERSION"
git tag -f "v$VERSION"
git push origin main
git push -f origin "v$VERSION"

# 4. Create the GitHub Release.
gh release create "v$VERSION" \
    --title "v$VERSION" \
    --notes "nix-helper release $VERSION. See nix.plg for the changelog." \
    ./nix-helper

echo ""
echo "Released v$VERSION"
echo "  → https://github.com/$(gh repo view --json nameWithOwner -q .nameWithOwner)/releases/tag/v$VERSION"