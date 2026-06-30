#!/usr/bin/env bash
set -euo pipefail

# Coalbox Updater
# Builds and installs the latest version from source.
# This script is embedded in the coalbox binary and executed when you run `coalbox update`.

REPO="${COALBOX_GIT_REPO:-https://github.com/SabeeirSharrma/CoalBox.git}"
INSTALL_DIR="${COALBOX_INSTALL_DIR:-$(dirname "$(which coalbox 2>/dev/null || echo /usr/local/bin/coalbox)")}"
BRANCH="${COALBOX_BRANCH:-main}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

info()  { echo -e "${GREEN}▸${NC} $*"; }
warn()  { echo -e "${YELLOW}▸${NC} $*"; }
error() { echo -e "${RED}▸${NC} $*" >&2; }
step()  { echo -e "\n${CYAN}── $* ──${NC}"; }

# Check for Rust
if ! command -v rustc &>/dev/null || ! command -v cargo &>/dev/null; then
    error "Rust is not installed. Install it first:"
    error "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

# Create temp build directory
BUILD_DIR=$(mktemp -d)
trap 'rm -rf "$BUILD_DIR"' EXIT

step "Cloning coalbox"
git clone --depth 1 --branch "$BRANCH" "$REPO" "$BUILD_DIR/coalbox" 2>&1 | tail -3

cd "$BUILD_DIR/coalbox"

step "Building release binary"
cargo build --release 2>&1 | tail -3

step "Installing update"

CLI_BINARY="target/release/coalbox"
WEB_BINARY="target/release/coalbox-web"

if [ ! -f "$CLI_BINARY" ]; then
    error "Build failed — binary not found at $CLI_BINARY"
    exit 1
fi

if [ -w "$INSTALL_DIR" ]; then
    cp "$CLI_BINARY" "${INSTALL_DIR}/coalbox"
    [ -f "$WEB_BINARY" ] && cp "$WEB_BINARY" "${INSTALL_DIR}/coalbox-web"
else
    sudo cp "$CLI_BINARY" "${INSTALL_DIR}/coalbox"
    [ -f "$WEB_BINARY" ] && sudo cp "$WEB_BINARY" "${INSTALL_DIR}/coalbox-web"
fi

chmod +x "${INSTALL_DIR}/coalbox"
[ -f "${INSTALL_DIR}/coalbox-web" ] && chmod +x "${INSTALL_DIR}/coalbox-web"

info "Updated coalbox to $(coalbox --version 2>/dev/null | head -1 | awk '{print $2}' || echo 'latest')"
