#!/usr/bin/env bash
# Builds Vellmark for Windows or Linux with its packs, its licenses, and the third-party
# notices, and archives it. Run on that platform, by CI. macOS has its own script.
# Usage: scripts/package.sh <windows|linux> <build number>
# The owner's build carries the development tools (the F keys and the console) until the
# Showcase is published; PUBLISH=1 builds without them, for a build the owner publishes.
set -euo pipefail
cd "$(dirname "$0")/.."
PLATFORM="${1:?windows or linux}"
BUILD="${2:-0}"
OUT="build/$PLATFORM/Vellmark"

if [ "${PUBLISH:-0}" = "1" ]; then
  cargo build --release -p vellmark-app --no-default-features
else
  cargo build --release -p vellmark-app --features dev
fi
cargo run --release -p vellmark-bake -- --out build/packs
# The notices cover every platform's crates, so fetch them all, not only this one's.
cargo fetch
cargo run --release -p vellmark-bake --bin notices

rm -rf "build/$PLATFORM"
mkdir -p "$OUT/packs" "$OUT/licenses"
if [ "$PLATFORM" = "windows" ]; then
  cp target/release/vellmark.exe "$OUT/Vellmark.exe"
else
  cp target/release/vellmark "$OUT/vellmark"
  chmod +x "$OUT/vellmark"
fi
cp build/packs/*.vpk "$OUT/packs/"
cp THIRD_PARTY_NOTICES.txt "$OUT/"
cp third_party/fonts/tiny5/OFL.txt "$OUT/licenses/Tiny5-OFL.txt"
cp third_party/fonts/imfellenglish/OFL.txt "$OUT/licenses/IMFellEnglish-OFL.txt"

if [ "$PLATFORM" = "windows" ]; then
  ARCHIVE="build/vellmark-windows-$BUILD.zip"
  (cd "build/$PLATFORM" && tar -a -cf "../vellmark-windows-$BUILD.zip" Vellmark)
else
  ARCHIVE="build/vellmark-linux-$BUILD.tar.gz"
  (cd "build/$PLATFORM" && tar -czf "../vellmark-linux-$BUILD.tar.gz" Vellmark)
fi
echo "$ARCHIVE"
