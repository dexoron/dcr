#!/usr/bin/env bash
set -Eeuo pipefail

DCRUP_HOME="${DCRUP_HOME:-$HOME/.dcr}"
DCRUP_BIN="${DCRUP_BIN:-$DCRUP_HOME/bin}"
DCRUP_TC="${DCRUP_HOME}/toolchains"
DCRUP_META="${DCRUP_HOME}/meta"
REPO_URL="${DCRUP_REPO:-https://github.com/dexoron/dcr}"
API_LATEST="https://api.github.com/repos/dexoron/dcr/releases/latest"
API_ALL="https://api.github.com/repos/dexoron/dcr/releases"
FEATURES="${DCR_FEATURES:-archive}"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

log()  { printf "${CYAN}[dcrup]${NC} %s\n" "$*"; }
ok()   { printf "${GREEN}[ok]${NC} %s\n" "$*"; }
warn() { printf "${YELLOW}[warn]${NC} %s\n" "$*"; }
die()  { printf "${RED}[error]${NC} %s\n" "$*" >&2; exit 1; }

usage() {
    cat <<'EOF'
dcrup — install and switch DCR versions

Usage:
  dcrup install <spec> [--build|--release] [--force] [--libc gnu|musl|auto]
  dcrup default <spec>
  dcrup update
  dcrup list
  dcrup show
  dcrup which
  dcrup uninstall [<spec>|--all]
  dcrup self-install [--to DIR]
  dcrup help

Spec:
  stable | dev | night
  VERSION              → VERSION@stable  (e.g. 0.8.2)
  VERSION@stable
  VERSION@dev
  night                → always build from branch "dev" HEAD

Modes:
  --release   download GitHub Release binary (default for stable/dev)
  --build     cargo build --release --features archive
  night       always --build (prebuilt not available)

Linux libc (prebuilt triple only):
  --libc gnu    x86_64-unknown-linux-gnu (default)
  --libc musl   x86_64-unknown-linux-musl
  --libc auto   musl if host ldd/libc is musl, else gnu

Env:
  DCRUP_HOME      default ~/.dcr
  DCRUP_BIN       default $DCRUP_HOME/bin
  DCR_FEATURES    default archive
  DCRUP_REPO      git/GitHub repo URL
  DCRUP_LIBC      gnu|musl|auto (default: gnu)
EOF
}

need_cmd() {
    command -v "$1" >/dev/null 2>&1 || die "required command not found: $1"
}

detect_linux_libc() {
    local pref="${DCRUP_LIBC:-gnu}"
    case "$pref" in
        gnu|glibc) echo "gnu" ;;
        musl) echo "musl" ;;
        auto)
            if command -v ldd >/dev/null 2>&1; then
                if ldd --version 2>&1 | grep -qi musl; then
                    echo "musl"
                    return 0
                fi
            fi
            if [[ -e /lib/ld-musl-x86_64.so.1 || -e /lib/ld-musl-aarch64.so.1 ]]; then
                echo "musl"
                return 0
            fi
            if command -v getconf >/dev/null 2>&1 && getconf GNU_LIBC_VERSION >/dev/null 2>&1; then
                echo "gnu"
                return 0
            fi
            echo "gnu"
            ;;
        *) die "invalid DCRUP_LIBC / --libc: $pref (use gnu|musl|auto)" ;;
    esac
}

detect_triple() {
    local os arch libc
    os="$(uname -s)"
    arch="$(uname -m)"
    case "$os:$arch" in
        Linux:x86_64)
            libc="$(detect_linux_libc)"
            if [[ "$libc" == "musl" ]]; then
                echo "x86_64-unknown-linux-musl"
            else
                echo "x86_64-unknown-linux-gnu"
            fi
            ;;
        Linux:aarch64|Linux:arm64)
            libc="$(detect_linux_libc)"
            if [[ "$libc" == "musl" ]]; then
                echo "aarch64-unknown-linux-musl"
            else
                echo "aarch64-unknown-linux-gnu"
            fi
            ;;
        Linux:i686|Linux:i386)
            libc="$(detect_linux_libc)"
            if [[ "$libc" == "musl" ]]; then
                echo "i686-unknown-linux-musl"
            else
                echo "i686-unknown-linux-gnu"
            fi
            ;;
        Linux:armv7l|Linux:armv7)
            libc="$(detect_linux_libc)"
            if [[ "$libc" == "musl" ]]; then
                echo "armv7-unknown-linux-musleabihf"
            else
                echo "armv7-unknown-linux-gnueabihf"
            fi
            ;;
        Linux:riscv64)
            libc="$(detect_linux_libc)"
            if [[ "$libc" == "musl" ]]; then
                echo "riscv64gc-unknown-linux-musl"
            else
                echo "riscv64gc-unknown-linux-gnu"
            fi
            ;;
        Darwin:x86_64)               echo "x86_64-apple-darwin" ;;
        Darwin:arm64|Darwin:aarch64) echo "aarch64-apple-darwin" ;;
        FreeBSD:x86_64|FreeBSD:amd64) echo "x86_64-unknown-freebsd" ;;
        FreeBSD:aarch64|FreeBSD:arm64) echo "aarch64-unknown-freebsd" ;;
        OpenBSD:x86_64|OpenBSD:amd64) echo "x86_64-unknown-openbsd" ;;
        NetBSD:x86_64|NetBSD:amd64)  echo "x86_64-unknown-netbsd" ;;
        MINGW*|MSYS*|CYGWIN*)
            case "$arch" in
                x86_64|amd64) echo "x86_64-pc-windows-gnu" ;;
                aarch64|arm64) echo "aarch64-pc-windows-gnullvm" ;;
                i686|i386) echo "i686-pc-windows-gnu" ;;
                *) die "unsupported Windows arch under MSYS: $arch" ;;
            esac
            ;;
        *) die "unsupported platform: $os/$arch" ;;
    esac
}

normalize_version() {
    local v="$1"
    v="${v#v}"
    printf '%s' "$v"
}

parse_spec() {
    local raw="$1"
    SPEC_RAW="$raw"
    SPEC_CHANNEL=""
    SPEC_VERSION=""
    SPEC_FLOATING=0

    case "$raw" in
        "") die "missing version/channel spec" ;;
        stable|dev|night)
            SPEC_CHANNEL="$raw"
            SPEC_VERSION=""
            SPEC_FLOATING=1
            ;;
        *@*)
            SPEC_VERSION="$(normalize_version "${raw%@*}")"
            SPEC_CHANNEL="${raw#*@}"
            case "$SPEC_CHANNEL" in
                stable|dev) ;;
                night) die "cannot pin a version on night (use: dcrup install night)" ;;
                *) die "unknown channel in spec: $SPEC_CHANNEL (use stable|dev)" ;;
            esac
            [[ -n "$SPEC_VERSION" ]] || die "empty version in spec: $raw"
            ;;
        *)
            SPEC_VERSION="$(normalize_version "$raw")"
            SPEC_CHANNEL="stable"
            [[ -n "$SPEC_VERSION" ]] || die "invalid spec: $raw"
            ;;
    esac
}

toolchain_id() {
    if [[ "$SPEC_CHANNEL" == "night" ]]; then
        local sha="${1:-unknown}"
        echo "night-${sha}"
    elif [[ "$SPEC_FLOATING" -eq 1 ]]; then
        local ver="${1:-latest}"
        echo "${SPEC_CHANNEL}-${ver}"
    else
        echo "${SPEC_VERSION}-${SPEC_CHANNEL}"
    fi
}

ensure_dirs() {
    mkdir -p "$DCRUP_TC" "$DCRUP_BIN" "$DCRUP_META"
}

active_file() { echo "$DCRUP_META/active"; }

read_active() {
    if [[ -f "$(active_file)" ]]; then
        cat "$(active_file)"
    fi
}

write_active() {
    printf '%s\n' "$1" >"$(active_file)"
}

link_default() {
    local id="$1"
    local src="$DCRUP_TC/$id/dcr"
    [[ -x "$src" ]] || die "toolchain not installed: $id"
    ln -sfn "$src" "$DCRUP_BIN/dcr"
    write_active "$id"
    ok "default → $id ($DCRUP_BIN/dcr)"
}

fetch_json() {
    need_cmd curl
    curl -fsSL "$1"
}

json_tag() {
    printf '%s' "$1" | sed -n 's/.*"tag_name"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' | head -n1
}

json_asset_url() {
    local json="$1" asset="$2"
    printf '%s' "$json" | sed -n "s#.*\"browser_download_url\"[[:space:]]*:[[:space:]]*\"\([^\"]*/${asset}\)\"#\1#p" | head -n1
}

latest_stable_json() {
    fetch_json "$API_LATEST"
}

latest_dev_json() {
    need_cmd curl
    local json result
    json="$(fetch_json "$API_ALL")"
    if command -v jq >/dev/null 2>&1; then
        result="$(printf '%s' "$json" | jq -e -c '[.[] | select(.prerelease == true)][0]')" || true
    elif command -v python3 >/dev/null 2>&1; then
        result="$(printf '%s' "$json" | python3 -c '
import sys, json
rels = json.load(sys.stdin)
pre = [r for r in rels if r.get("prerelease")]
print(json.dumps(pre[0]) if pre else "", end="")
')"
    else
        die "dev channel needs jq or python3 to parse GitHub API"
    fi
    [[ -n "$result" && "$result" != "null" ]] || die "no prerelease (dev) found"
    printf '%s' "$result"
}

release_json_for_spec() {
    if [[ "$SPEC_FLOATING" -eq 1 ]]; then
        case "$SPEC_CHANNEL" in
            stable) latest_stable_json ;;
            dev)    latest_dev_json ;;
            night)  die "night has no GitHub release binary" ;;
        esac
    else
        fetch_json "https://api.github.com/repos/dexoron/dcr/releases/tags/v${SPEC_VERSION}"
    fi
}

install_prebuilt() {
    local triple asset tag version url json id dest
    need_cmd curl
    triple="$(detect_triple)"
    json="$(release_json_for_spec)"
    tag="$(json_tag "$json")"
    [[ -n "$tag" ]] || die "failed to parse release tag"
    version="$(normalize_version "$tag")"
    asset="dcr-${triple}-${version}"
    case "$triple" in
        *windows*) asset="${asset}.exe" ;;
    esac
    url="$(json_asset_url "$json" "$asset")"
    [[ -n "$url" ]] || die "asset not found: $asset (tag $tag)"

    id="$(toolchain_id "$version")"
    dest="$DCRUP_TC/$id"
    if [[ -x "$dest/dcr" && "${FORCE:-0}" != "1" ]]; then
        ok "already installed: $id"
        link_default "$id"
        return 0
    fi

    log "downloading $asset …"
    mkdir -p "$dest"
    curl -fL "$url" -o "$dest/dcr"
    chmod +x "$dest/dcr"
    printf '%s\n' \
        "channel=$SPEC_CHANNEL" \
        "version=$version" \
        "tag=$tag" \
        "source=release" \
        "triple=$triple" \
        "libc=${DCRUP_LIBC:-gnu}" \
        >"$dest/dcrup-meta"
    ok "installed prebuilt $id ($triple)"
    link_default "$id"
}

install_build() {
    local tmp id dest sha version branch_or_tag
    need_cmd git
    need_cmd cargo

    tmp="$(mktemp -d "${TMPDIR:-/tmp}/dcrup.XXXXXX")"
    cleanup_tmp() { rm -rf "$tmp"; }
    trap cleanup_tmp EXIT

    if [[ "$SPEC_CHANNEL" == "night" ]]; then
        log "cloning $REPO_URL (branch dev) for night …"
        git clone --depth 1 --branch dev "$REPO_URL" "$tmp/src" 2>/dev/null \
            || git clone --depth 1 "$REPO_URL" "$tmp/src"
        sha="$(git -C "$tmp/src" rev-parse --short HEAD)"
        version="night"
        id="$(toolchain_id "$sha")"
    elif [[ "$SPEC_FLOATING" -eq 1 ]]; then
        local json tag
        json="$(release_json_for_spec)"
        tag="$(json_tag "$json")"
        [[ -n "$tag" ]] || die "failed to resolve floating channel tag"
        version="$(normalize_version "$tag")"
        id="$(toolchain_id "$version")"
        log "cloning $REPO_URL @ $tag …"
        git clone --depth 1 --branch "$tag" "$REPO_URL" "$tmp/src" 2>/dev/null \
            || {
                git clone --depth 1 "$REPO_URL" "$tmp/src"
                git -C "$tmp/src" fetch --depth 1 origin "refs/tags/${tag}:refs/tags/${tag}"
                git -C "$tmp/src" checkout "$tag"
            }
        sha="$(git -C "$tmp/src" rev-parse --short HEAD)"
    else
        version="$SPEC_VERSION"
        id="$(toolchain_id)"
        local tag="v${SPEC_VERSION}"
        log "cloning $REPO_URL @ $tag …"
        git clone --depth 1 --branch "$tag" "$REPO_URL" "$tmp/src" 2>/dev/null \
            || {
                git clone --depth 1 "$REPO_URL" "$tmp/src"
                git -C "$tmp/src" fetch --depth 1 origin "refs/tags/${tag}:refs/tags/${tag}"
                git -C "$tmp/src" checkout "$tag"
            }
        sha="$(git -C "$tmp/src" rev-parse --short HEAD)"
    fi

    dest="$DCRUP_TC/$id"
    if [[ -x "$dest/dcr" && "${FORCE:-0}" != "1" ]]; then
        ok "already installed: $id"
        link_default "$id"
        return 0
    fi

    log "building release (features: $FEATURES) …"
    (
        cd "$tmp/src"
        # shellcheck disable=SC2086
        cargo build --release --features $FEATURES
    )
    mkdir -p "$dest"
    cp "$tmp/src/target/release/dcr" "$dest/dcr"
    chmod +x "$dest/dcr"
    printf '%s\n' "channel=$SPEC_CHANNEL" "version=$version" "git=$sha" "source=build" "features=$FEATURES" >"$dest/dcrup-meta"
    ok "installed build $id"
    link_default "$id"
    trap - EXIT
    cleanup_tmp
}

cmd_install() {
    local mode="release" FORCE=0
    local spec=""
    local libc_cli=""
    while [[ $# -gt 0 ]]; do
        case "$1" in
            --build) mode="build"; shift ;;
            --release) mode="release"; shift ;;
            --force) FORCE=1; shift ;;
            --libc)
                [[ $# -ge 2 ]] || die "--libc needs gnu|musl|auto"
                libc_cli="$2"
                shift 2
                ;;
            --libc=*)
                libc_cli="${1#--libc=}"
                shift
                ;;
            -h|--help) usage; exit 0 ;;
            -*) die "unknown flag: $1" ;;
            *)
                [[ -z "$spec" ]] || die "unexpected argument: $1"
                spec="$1"
                shift
                ;;
        esac
    done
    [[ -n "$spec" ]] || die "usage: dcrup install <spec> [--build|--release] [--force] [--libc gnu|musl|auto]"
    parse_spec "$spec"
    ensure_dirs

    if [[ -n "$libc_cli" ]]; then
        case "$libc_cli" in
            gnu|glibc|musl|auto) DCRUP_LIBC="$libc_cli" ;;
            *) die "invalid --libc: $libc_cli (use gnu|musl|auto)" ;;
        esac
    fi
    export DCRUP_LIBC="${DCRUP_LIBC:-gnu}"

    if [[ "$SPEC_CHANNEL" == "night" ]]; then
        mode="build"
    fi

    export FORCE
    log "install spec=$SPEC_RAW channel=$SPEC_CHANNEL version=${SPEC_VERSION:-∅} mode=$mode libc=$DCRUP_LIBC"
    if [[ "$mode" == "build" ]]; then
        install_build
    else
        install_prebuilt
    fi
    check_path
}

cmd_default() {
    [[ $# -eq 1 ]] || die "usage: dcrup default <spec>"
    parse_spec "$1"
    ensure_dirs
    local id
    if [[ "$SPEC_CHANNEL" == "night" || "$SPEC_FLOATING" -eq 1 ]]; then
        id="$(ls -1 "$DCRUP_TC" 2>/dev/null | grep -E "^${SPEC_CHANNEL}-" | sort | tail -n1 || true)"
        [[ -n "$id" ]] || die "no installed toolchain for channel $SPEC_CHANNEL (run: dcrup install $SPEC_CHANNEL)"
    else
        id="$(toolchain_id)"
        [[ -d "$DCRUP_TC/$id" ]] || die "not installed: $id (run: dcrup install $1)"
    fi
    link_default "$id"
}

cmd_update() {
    ensure_dirs
    local active
    active="$(read_active)"
    [[ -n "$active" ]] || die "no active toolchain (run: dcrup install stable)"
    case "$active" in
        night-*)
            log "updating night (rebuild HEAD) …"
            FORCE=1 cmd_install night --build --force
            ;;
        stable-*|dev-*)
            local ch="${active%%-*}"
            log "updating floating channel $ch …"
            FORCE=1 cmd_install "$ch" --force
            ;;
        *-stable|*-dev)
            ok "pinned toolchain $active — not floating; install a newer version explicitly"
            ;;
        *)
            warn "unknown active id $active; try: dcrup install stable"
            ;;
    esac
}

cmd_list() {
    ensure_dirs
    local active
    active="$(read_active)"
    if [[ ! -d "$DCRUP_TC" ]] || [[ -z "$(ls -A "$DCRUP_TC" 2>/dev/null || true)" ]]; then
        warn "no toolchains installed"
        return 0
    fi
    local d base
    for d in "$DCRUP_TC"/*; do
        [[ -d "$d" ]] || continue
        base="$(basename "$d")"
        if [[ "$base" == "$active" ]]; then
            printf ' * %s\n' "$base"
        else
            printf '   %s\n' "$base"
        fi
    done
}

cmd_show() {
    ensure_dirs
    local active
    active="$(read_active)"
    echo "DCRUP_HOME=$DCRUP_HOME"
    echo "DCRUP_BIN=$DCRUP_BIN"
    echo "active=${active:-none}"
    if [[ -n "$active" && -f "$DCRUP_TC/$active/dcrup-meta" ]]; then
        echo "--- meta ---"
        cat "$DCRUP_TC/$active/dcrup-meta"
    fi
    if [[ -x "$DCRUP_BIN/dcr" ]]; then
        echo "--- dcr --version ---"
        "$DCRUP_BIN/dcr" --version 2>/dev/null || true
    fi
}

cmd_which() {
    if [[ -L "$DCRUP_BIN/dcr" || -e "$DCRUP_BIN/dcr" ]]; then
        readlink -f "$DCRUP_BIN/dcr" 2>/dev/null || readlink "$DCRUP_BIN/dcr" 2>/dev/null || echo "$DCRUP_BIN/dcr"
    else
        die "dcr not linked (run: dcrup install stable)"
    fi
}

cmd_uninstall() {
    ensure_dirs
    if [[ "${1:-}" == "--all" ]]; then
        rm -rf "$DCRUP_TC"
        rm -f "$DCRUP_BIN/dcr" "$(active_file)"
        ok "removed all toolchains"
        return 0
    fi
    [[ $# -eq 1 ]] || die "usage: dcrup uninstall <spec>|--all"
    parse_spec "$1"
    local id
    if [[ "$SPEC_FLOATING" -eq 1 || "$SPEC_CHANNEL" == "night" ]]; then
        id="$(ls -1 "$DCRUP_TC" 2>/dev/null | grep -E "^${SPEC_CHANNEL}-" | sort | tail -n1 || true)"
    else
        id="$(toolchain_id)"
    fi
    [[ -n "$id" && -d "$DCRUP_TC/$id" ]] || die "not installed: $1"
    rm -rf "$DCRUP_TC/$id"
    if [[ "$(read_active)" == "$id" ]]; then
        rm -f "$DCRUP_BIN/dcr" "$(active_file)"
        warn "removed active toolchain; run: dcrup install stable"
    fi
    ok "removed $id"
}

check_path() {
    case ":$PATH:" in
        *":$DCRUP_BIN:"*) ;;
        *)
            warn "$DCRUP_BIN is not in PATH"
            echo "Add to ~/.bashrc / ~/.zshrc / ~/.profile:"
            echo "  export PATH=\"$DCRUP_BIN:\$PATH\""
            ;;
    esac
}

cmd_self_install() {
    local dest="$DCRUP_BIN"
    while [[ $# -gt 0 ]]; do
        case "$1" in
            --to) dest="$2"; shift 2 ;;
            *) die "usage: dcrup self-install [--to DIR]" ;;
        esac
    done
    ensure_dirs
    mkdir -p "$dest"
    local self
    self="$(readlink -f "$0" 2>/dev/null || realpath "$0" 2>/dev/null || echo "$0")"
    cp "$self" "$dest/dcrup"
    chmod +x "$dest/dcrup"
    ok "dcrup installed to $dest/dcrup"
    check_path
    echo "Then: dcrup install stable"
}

main() {
    local cmd="${1:-}"
    shift || true
    case "$cmd" in
        ""|-h|--help|help) usage ;;
        install) cmd_install "$@" ;;
        default) cmd_default "$@" ;;
        update) cmd_update "$@" ;;
        list) cmd_list "$@" ;;
        show) cmd_show "$@" ;;
        which) cmd_which "$@" ;;
        uninstall) cmd_uninstall "$@" ;;
        self-install) cmd_self_install "$@" ;;
        *) die "unknown command: $cmd (try: dcrup help)" ;;
    esac
}

main "$@"
