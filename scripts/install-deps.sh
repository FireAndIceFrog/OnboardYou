#!/usr/bin/env bash
set -euo pipefail

# Install system dependencies needed for OnboardYou (Ubuntu/Debian).
# Run as: sudo ./scripts/install-deps.sh

if [ "$EUID" -ne 0 ]; then
  echo "Please run as root (sudo)."
  exit 1
fi

echo "==> Updating apt packages..."
apt update -y

echo "==> Installing required packages..."
apt install -y \
  build-essential \
  curl \
  git \
  make \
  python3 \
  python3-venv \
  python3-pip \
  libssl-dev \
  pkg-config \
  postgresql-client \
  jq

# Node.js (LTS) via NodeSource
if ! command -v node >/dev/null 2>&1; then
  echo "==> Installing Node.js (LTS)..."
  curl -fsSL https://deb.nodesource.com/setup_lts.x | bash -
  apt install -y nodejs
else
  echo "==> Node.js already installed: $(node --version)"
fi

# Rust toolchain via rustup
if ! command -v rustc >/dev/null 2>&1; then
  echo "==> Installing Rust toolchain (rustup)..."
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
  export PATH="$HOME/.cargo/bin:$PATH"
else
  echo "==> Rust already installed: $(rustc --version)"
fi

# pnpm
if ! command -v pnpm >/dev/null 2>&1; then
  echo "==> Installing pnpm..."
  npm install -g pnpm
else
  echo "==> pnpm already installed: $(pnpm --version)"
fi

# cargo-lambda (Python venv is created in make setup)
if ! command -v cargo-lambda >/dev/null 2>&1; then
  echo "==> cargo-lambda not found. It will be installed by 'make setup' later."
else
  echo "==> cargo-lambda already installed: $(cargo-lambda --version)"
fi

# OpenTofu CLI
if ! command -v tofu >/dev/null 2>&1; then
  echo "==> OpenTofu (tofu) not found. Please install it manually from https://opentofu.org/install/"
else
  echo "==> OpenTofu already installed: $(tofu version)"
fi

echo "\n✅ System dependencies are installed."

echo "Next steps:"
echo "  1) cd $(pwd)"
echo "  2) make setup"
echo "  3) make deploy  # or run individual Make targets as needed"
