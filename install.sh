#!/usr/bin/env bash
set -Eeuo pipefail

TMPDIR="/tmp/dcr-install"
INSTALL_PATH="$HOME/.local/share/dcr"
BINPATH="$HOME/.local/bin"
LOGFILE="$HOME/.cache/dcr-install.log"
REPO_URL="https://github.com/dexoron/dcr"
GITHUB_API="https://api.github.com/repos/dexoron/dcr/releases/latest"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

mkdir -p "$(dirname "$LOGFILE")"
exec > >(tee -a "$LOGFILE") 2>&1

log() { echo -e "${BLUE}[$(date '+%Y-%m-%d %H:%M:%S')]${NC} $1"; }
success() { echo -e "${GREEN}✔ $1${NC}"; }
warn() { echo -e "${YELLOW}⚠ $1${NC}"; }
error() { echo -e "${RED}✖ $1${NC}"; }

trap 'error "Error on line $LINENO"; exit 1' ERR

check_os() {
    case "$(uname -s)" in
        Linux|Darwin) ;;
        *) error "Only Linux and macOS are supported"; exit 1 ;;
    esac
}

detect_target() {
    local os arch
    os="$(uname -s)"
    arch="$(uname -m)"

    case "$os:$arch" in
        Linux:x86_64) TARGET_TRIPLE="x86_64-unknown-linux-gnu" ;;
        Darwin:x86_64) TARGET_TRIPLE="x86_64-apple-darwin" ;;
        Darwin:arm64|Darwin:aarch64) TARGET_TRIPLE="aarch64-apple-darwin" ;;
        *) error "Unsupported platform: $os/$arch"; exit 1 ;;
    esac

    ASSET_NAME="dcr-${TARGET_TRIPLE}"
}

check_common_dependencies() {
    command -v curl >/dev/null 2>&1 || { error "curl is not installed"; exit 1; }
}

check_build_dependencies() {
    command -v git >/dev/null 2>&1 || { error "git is not installed"; exit 1; }
    command -v cargo >/dev/null 2>&1 || { error "cargo is not installed"; exit 1; }
}

prepare_sources() {
    log "Fetching sources..."
    rm -rf "$TMPDIR"
    git clone --depth 1 "$REPO_URL" "$TMPDIR"
    success "Sources fetched"
}

build_binary() {
    log "Building release binary..."
    (cd "$TMPDIR" && cargo build --release)
    success "Build completed"
}

fetch_latest_release_json() {
    curl -fsSL "$GITHUB_API"
}

download_binary() {
    log "Fetching latest release..."

    local release_json download_url tag
    release_json="$(fetch_latest_release_json)"

    tag="$(printf '%s\n' "$release_json" | sed -n 's/.*"tag_name": "\([^"]*\)".*/\1/p' | head -n1)"
    if [[ -z "$tag" ]]; then
        error "Failed to determine release version"
        exit 1
    fi

    download_url="$(printf '%s\n' "$release_json" | sed -n "s#.*\"browser_download_url\": \"\([^\"]*${ASSET_NAME}\)\".*#\1#p" | head -n1)"
    if [[ -z "$download_url" ]]; then
        error "Asset ${ASSET_NAME} not found in release ${tag}"
        exit 1
    fi

    mkdir -p "$INSTALL_PATH"
    curl -fL "$download_url" -o "$INSTALL_PATH/dcr"
    chmod +x "$INSTALL_PATH/dcr"
    success "Binary ${ASSET_NAME} downloaded (${tag})"
}

install_built_binary() {
    mkdir -p "$INSTALL_PATH"
    cp "$TMPDIR/target/release/dcr" "$INSTALL_PATH/dcr"
    chmod +x "$INSTALL_PATH/dcr"
    success "Binary installed from source"
}

install_link() {
    log "Creating symlink..."
    mkdir -p "$BINPATH"
    ln -sf "$INSTALL_PATH/dcr" "$BINPATH/dcr"
    success "Command 'dcr' added to $BINPATH"
}

check_path() {
    if ! echo "$PATH" | grep -q "$BINPATH"; then
        warn "Directory $BINPATH not found in PATH"
        echo "Add this to ~/.bashrc or ~/.zshrc:"
        echo "export PATH=\"$BINPATH:\$PATH\""
    fi
}

cleanup() {
    rm -rf "$TMPDIR" 2>/dev/null || true
}

select_install_mode() {
    echo "Choose installation mode:"
    echo "  1) Download prebuilt binary from GitHub Release (recommended)"
    echo "  2) Build from git"
    read -r -p "Enter 1 or 2 [1]: " choice

    case "${choice:-1}" in
        1) INSTALL_MODE="release" ;;
        2) INSTALL_MODE="build" ;;
        *) error "Unknown option"; exit 1 ;;
    esac
}

main() {
    log "Starting DCR installation"

    check_os
    detect_target
    check_common_dependencies
    select_install_mode
    cleanup

    if [[ "$INSTALL_MODE" == "build" ]]; then
        check_build_dependencies
        prepare_sources
        build_binary
        install_built_binary
    else
        download_binary
    fi

    install_link
    check_path
    cleanup

    success "Installation completed successfully"
    log "Log saved to $LOGFILE"
}

main "$@"
