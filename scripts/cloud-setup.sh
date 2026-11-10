#!/usr/bin/env bash
# Setup for Claude Code cloud sessions. A starting point: task S00-T01 runs it on the
# real machine, fixes what fails, and commits the working version.
# Keep it under about five minutes so the environment can cache it.
set -euo pipefail

# Rust, pinned by rust-toolchain.toml once task S00-T02 has created it.
if ! command -v cargo >/dev/null 2>&1; then
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal
  . "$HOME/.cargo/env"
fi
rustup component add clippy rustfmt || true

# Build tools for SDL3 from source, and a software Vulkan driver for captures.
if command -v apt-get >/dev/null 2>&1; then
  export DEBIAN_FRONTEND=noninteractive
  SUDO=""; [ "$(id -u)" -ne 0 ] && SUDO="sudo"
  $SUDO apt-get update -y || true
  $SUDO apt-get install -y --no-install-recommends \
    build-essential cmake ninja-build pkg-config git \
    libasound2-dev libpulse-dev libudev-dev libdbus-1-dev \
    libx11-dev libxext-dev libxrandr-dev libxcursor-dev libxi-dev libxfixes-dev libxss-dev \
    libwayland-dev libxkbcommon-dev wayland-protocols \
    libvulkan1 mesa-vulkan-drivers vulkan-tools xvfb || true
fi

cargo install --locked cargo-deny || true

echo "Software Vulkan devices:"
vulkaninfo --summary 2>/dev/null | grep -i "deviceName" || echo "  none found: captures will need another route. Record this in docs/DECISIONS.md."
