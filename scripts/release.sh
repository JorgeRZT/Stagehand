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
