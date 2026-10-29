#!/usr/bin/env bash
# Builds the macOS DMG and copies it to dist/<YYYY.MM.DD>/.
# The app version (tauri.conf.json + Cargo.toml) is set to today's date. SemVer forbids leading
# zeros, so the app version is e.g. 2026.3.5 while the dist folder is zero-padded: 2026.03.05.
set -euo pipefail
cd "$(dirname "$0")/.."

FOLDER=$(date +%Y.%m.%d)
VERSION=$(date +%Y.%-m.%-d)

perl -pi -e "s/\"version\": \"[^\"]+\"/\"version\": \"$VERSION\"/ if !\$done++" src-tauri/tauri.conf.json
perl -pi -e "s/^version = \"[^\"]+\"/version = \"$VERSION\"/ if !\$done++" src-tauri/Cargo.toml

cargo tauri build --bundles dmg
mkdir -p "dist/$FOLDER"
cp src-tauri/target/release/bundle/dmg/*_"$VERSION"_*.dmg "dist/$FOLDER/"
echo "Done: dist/$FOLDER"

# Optional: ./scripts/release.sh --publish creates the GitHub Release (tag v$VERSION) with the DMG attached.
# Commit and push first so the tag points at the released code.
if [[ "${1:-}" == "--publish" ]]; then
  gh release create "v$VERSION" "dist/$FOLDER"/*_"$VERSION"_*.dmg \
    --title "Stagehand $VERSION" --notes "macOS (Apple Silicon) build $VERSION. Unsigned: on first launch, right-click the app and choose Open." --target main
fi
