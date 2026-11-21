#!/usr/bin/env bash
# Builds Vellmark from this folder and runs it, on macOS or Linux: checks what the build
# needs, takes the newest code if this is a git checkout, builds outside any folder whose
# path has a space in it (SDL's CMake build fails there), and starts in the valley.
set -uo pipefail
cd "$(dirname "$0")/.." || exit 1

fail() {
  echo
  echo "$1"
  echo
  read -r -p "Press Return to close." _ </dev/tty 2>/dev/null || true
  exit 1
}

# Rust.
if ! command -v cargo >/dev/null 2>&1 && [ -x "$HOME/.cargo/bin/cargo" ]; then
  export PATH="$HOME/.cargo/bin:$PATH"
fi
command -v cargo >/dev/null 2>&1 || fail "Rust is not installed. Install it with:
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
then open a new terminal and run this again."

# The C toolchain and CMake, which build SDL.
if [ "$(uname)" = "Darwin" ]; then
  xcode-select -p >/dev/null 2>&1 || { xcode-select --install; fail "Apple's command line tools are being installed. When that finishes, run this again."; }
  if ! command -v cmake >/dev/null 2>&1; then
    command -v brew >/dev/null 2>&1 || fail "CMake is needed. Install Homebrew from https://brew.sh, then run: brew install cmake"
    echo "Installing CMake with Homebrew..."
    brew install cmake || fail "brew install cmake did not finish."
  fi
else
  command -v cmake >/dev/null 2>&1 || fail "CMake and the build libraries are needed. On Ubuntu or Debian:
  sudo apt-get install -y build-essential cmake ninja-build pkg-config libasound2-dev libpulse-dev libudev-dev libdbus-1-dev libx11-dev libxext-dev libxrandr-dev libxcursor-dev libxi-dev libxfixes-dev libxss-dev libxtst-dev libxinerama-dev libxrender-dev libgl1-mesa-dev libegl1-mesa-dev libdrm-dev libgbm-dev libwayland-dev libxkbcommon-dev wayland-protocols libvulkan1 mesa-vulkan-drivers"
fi

# The newest code, if this folder is a git checkout and nothing here is changed.
if [ -d .git ] && command -v git >/dev/null 2>&1; then
  if [ -z "$(git status --porcelain --untracked-files=no 2>/dev/null)" ]; then
    echo "Taking the newest code..."
    git pull --ff-only --quiet || echo "Could not take the newest code; building what is here."
  fi
fi

# SDL's CMake build fails in a folder whose path has a space in it: build elsewhere then.
case "$PWD" in
  *" "*) export CARGO_TARGET_DIR="$HOME/.vellmark-build"
         echo "This folder's path has a space in it, so the build goes to $CARGO_TARGET_DIR." ;;
esac

if [ "$#" -eq 0 ]; then set -- --scene valley; fi
echo "Building (the first time takes several minutes)..."
cargo run --release -p vellmark-app --features dev -- "$@" || fail "The build or the game stopped with an error, shown above."
