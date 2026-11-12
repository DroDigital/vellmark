#!/usr/bin/env bash
# Builds Vellmark.app with its packs and the performance-run command, and zips them.
# Run on macOS, by CI. Usage: scripts/package-macos.sh <build number>
set -euo pipefail
cd "$(dirname "$0")/.."
BUILD="${1:-0}"
OUT="build/macos"
APP="$OUT/Vellmark.app"

# The owner's build carries the development tools (the F keys and the console) until
# the Showcase is published; a published build is made without them.
cargo build --release -p vellmark-app --features dev
cargo run --release -p vellmark-bake -- --out build/packs

rm -rf "$OUT"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources/packs"
cp target/release/vellmark "$APP/Contents/MacOS/vellmark"
cp build/packs/*.vpk "$APP/Contents/Resources/packs/"
sed "s/BUILD_NUMBER/$BUILD/" packaging/macos/Info.plist > "$APP/Contents/Info.plist"
cp packaging/macos/perf-run.command packaging/macos/valley.command packaging/macos/READ-ME-FIRST.txt "$OUT/"
chmod +x "$OUT/perf-run.command" "$OUT/valley.command"
# The typefaces' license travels with the game.
mkdir -p "$APP/Contents/Resources/licenses"
cp third_party/fonts/tiny5/OFL.txt "$APP/Contents/Resources/licenses/Tiny5-OFL.txt"
cp third_party/fonts/imfellenglish/OFL.txt "$APP/Contents/Resources/licenses/IMFellEnglish-OFL.txt"
# An ad-hoc signature: Apple Silicon will not run an unsigned binary.
codesign --force --deep --sign - "$APP"

(cd "$OUT" && zip -qry "../vellmark-macos-$BUILD.zip" .)
echo "build/vellmark-macos-$BUILD.zip"
