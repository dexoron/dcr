#!/usr/bin/env bash
set -Eeuo pipefail

TMPDIR="/tmp/dcr-install"
INSTALL_PATH="$HOME/.local/share/dcr"
BINPATH="$HOME/.local/bin"
MANPATH="${XDG_DATA_HOME:-$HOME/.local/share}/man"
LOGFILE="$HOME/.cache/dcr-install.log"
REPO_URL="https://github.com/dexoron/dcr"
GITHUB_API_LATEST="https://api.github.com/repos/dexoron/dcr/releases/latest"
GITHUB_API_ALL="https://api.github.com/repos/dexoron/dcr/releases"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

mkdir -p "$(dirname "$LOGFILE")"
exec > >(tee -a "$LOGFILE") 2>&1

log()     { echo -e "${BLUE}[$(date '+%Y-%m-%d %H:%M:%S')]${NC} $1"; }
success() { echo -e "${GREEN}✔ $1${NC}"; }
warn()    { echo -e "${YELLOW}⚠ $1${NC}"; }
error()   { echo -e "${RED}✖ $1${NC}"; }

trap 'error "Error on line $LINENO"; exit 1' ERR

INSTALL_MODE=""
CHANNEL=""

check_os() {
    case "$(uname -s)" in
        Linux|Darwin|FreeBSD|OpenBSD|NetBSD) ;;
        *) error "Only Linux, macOS, and BSD systems are supported"; exit 1 ;;
    esac
}

detect_target() {
    local os arch
    os="$(uname -s)"
    arch="$(uname -m)"

    case "$os:$arch" in
        Linux:x86_64)                      TARGET_TRIPLE="x86_64-unknown-linux-gnu" ;;
        Linux:aarch64|Linux:arm64)         TARGET_TRIPLE="aarch64-unknown-linux-gnu" ;;
        Linux:i686|Linux:i386)             TARGET_TRIPLE="i686-unknown-linux-gnu" ;;
        Linux:armv7l|Linux:armv7)          TARGET_TRIPLE="armv7-unknown-linux-gnueabihf" ;;
        Linux:riscv64)                     TARGET_TRIPLE="riscv64gc-unknown-linux-gnu" ;;
        Darwin:x86_64)                     TARGET_TRIPLE="x86_64-apple-darwin" ;;
        Darwin:arm64|Darwin:aarch64)       TARGET_TRIPLE="aarch64-apple-darwin" ;;
        FreeBSD:x86_64|FreeBSD:amd64)      TARGET_TRIPLE="x86_64-unknown-freebsd" ;;
        FreeBSD:aarch64|FreeBSD:arm64)     TARGET_TRIPLE="aarch64-unknown-freebsd" ;;
        FreeBSD:i686|FreeBSD:i386)         TARGET_TRIPLE="i686-unknown-freebsd" ;;
        OpenBSD:x86_64|OpenBSD:amd64)      TARGET_TRIPLE="x86_64-unknown-openbsd" ;;
        OpenBSD:aarch64|OpenBSD:arm64)     TARGET_TRIPLE="aarch64-unknown-openbsd" ;;
        OpenBSD:i686|OpenBSD:i386)         TARGET_TRIPLE="i686-unknown-openbsd" ;;
        NetBSD:x86_64|NetBSD:amd64)        TARGET_TRIPLE="x86_64-unknown-netbsd" ;;
        NetBSD:aarch64|NetBSD:arm64)       TARGET_TRIPLE="aarch64-unknown-netbsd" ;;
        NetBSD:i686|NetBSD:i386)           TARGET_TRIPLE="i686-unknown-netbsd" ;;

        *) error "Unsupported platform: $os/$arch"; exit 1 ;;
    esac
}

check_common_dependencies() {
    command -v curl >/dev/null 2>&1 || { error "curl is not installed"; exit 1; }
    if [[ "$CHANNEL" == "dev" ]]; then
        if ! command -v python3 >/dev/null 2>&1 && ! command -v jq >/dev/null 2>&1; then
            error "For dev installations, either 'python3' or 'jq' must be installed to parse GitHub API response."
            exit 1
        fi
    fi
}

check_build_dependencies() {
    command -v git   >/dev/null 2>&1 || { error "git is not installed"; exit 1; }
    command -v cargo >/dev/null 2>&1 || { error "cargo is not installed"; exit 1; }
}

select_channel() {
    echo "Choose channel:"
    echo "  1) Latest stable release (default)"
    echo "  2) Latest dev (pre-release)"
    read -r -p "Enter 1 or 2 [1]: " choice < /dev/tty

    case "${choice:-1}" in
        1) CHANNEL="stable" ;;
        2) CHANNEL="dev"    ;;
        *) error "Unknown option"; exit 1 ;;
    esac
}

select_install_mode() {
    echo "Choose installation mode:"
    echo "  1) Download prebuilt binary from GitHub Release (recommended)"
    echo "  2) Build from git"
    read -r -p "Enter 1 or 2 [1]: " choice < /dev/tty

    case "${choice:-1}" in
        1) INSTALL_MODE="release" ;;
        2) INSTALL_MODE="build"   ;;
        *) error "Unknown option"; exit 1 ;;
    esac
}

# Returns JSON of the requested release to stdout
fetch_release_json() {
    if [[ "$CHANNEL" == "dev" ]]; then
        log "Looking for latest dev (pre-release)..."
        local json result
        json="$(curl -fsSL "$GITHUB_API_ALL")"
        if command -v jq >/dev/null 2>&1; then
            result="$(printf '%s' "$json" | jq -e -c 'map(select(.prerelease == true))[0]')"
        else
            result="$(printf '%s' "$json" | python3 - <<'EOF'
import sys, json
releases = json.load(sys.stdin)
pre = [r for r in releases if r.get("prerelease", False)]
if not pre:
    print("{}", end="")
else:
    print(json.dumps(pre[0]), end="")
EOF
)"
        fi

        if [[ -z "$result" || "$result" == "{}" || "$result" == "null" ]]; then
            error "No dev (pre-release) found on GitHub"
            exit 1
        fi
        printf '%s' "$result"
    else
        curl -fsSL "$GITHUB_API_LATEST"
    fi
}

download_binary() {
    local release_json tag version asset_name download_url

    release_json="$(fetch_release_json)"

    tag="$(printf '%s\n' "$release_json" | \
        sed -n 's/.*"tag_name": "\([^"]*\)".*/\1/p' | head -n1)"
    if [[ -z "$tag" ]]; then
        error "Failed to determine release version"
        exit 1
    fi

    # Version without leading 'v'
    version="${tag#v}"
    # Binary name: dcr-<triple>-<version>
    asset_name="dcr-${TARGET_TRIPLE}-${version}"

    log "Fetching release ${tag} (channel: ${CHANNEL})..."

    download_url="$(printf '%s\n' "$release_json" | \
        sed -n "s#.*\"browser_download_url\": \"\([^\"]*/${asset_name}\)\".*#\1#p" | head -n1)"

    if [[ -z "$download_url" ]]; then
        error "Asset ${asset_name} not found in release ${tag}"
        exit 1
    fi

    mkdir -p "$INSTALL_PATH"
    curl -fL "$download_url" -o "$INSTALL_PATH/dcr"
    chmod +x "$INSTALL_PATH/dcr"
    success "Binary ${asset_name} downloaded (${tag})"
}

prepare_sources() {
    log "Fetching sources..."
    rm -rf "$TMPDIR"
    git clone --depth 1 "$REPO_URL" "$TMPDIR"
    if [[ "$CHANNEL" == "dev" ]]; then
        # Clone dev branch if it exists
        git clone --depth 1 --branch dev "$REPO_URL" "$TMPDIR" 2>/dev/null || \
        git clone --depth 1 "$REPO_URL" "$TMPDIR"
    fi
    success "Sources fetched"
}

build_binary() {
    log "Building release binary..."
    (cd "$TMPDIR" && cargo build --release)
    success "Build completed"
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

install_man() {
    log "Installing man pages..."
    local man_src="$TMPDIR/man/man1"
    if [[ -d "$man_src" ]]; then
        mkdir -p "$MANPATH/man1"
        cp "$man_src"/*.1 "$MANPATH/man1/"
        success "Man pages installed to $MANPATH/man1"
    fi
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

main() {
    log "Starting DCR installation"

    check_os
    detect_target
    select_channel
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
    install_man
    check_path
    cleanup

    success "Installation completed successfully"
    log "Log saved to $LOGFILE"
}

main "$@"
