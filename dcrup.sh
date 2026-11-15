#!/usr/bin/env bash
# dcrup needs bash. `curl ... | sh` may start dash/ash, so re-run under bash.
# This block must stay POSIX sh.
if [ -z "${BASH_VERSION:-}" ]; then
  if ! command -v bash >/dev/null 2>&1; then
    echo "[error] dcrup needs bash (install bash and retry)" >&2
    exit 1
  fi
  if [ -f "$0" ] && grep -q 'DCRUP_HOME' "$0" 2>/dev/null; then
    exec bash "$0" "$@"
  fi
  _dcrup_tmp="$(mktemp "${TMPDIR:-/tmp}/dcrup.XXXXXX")" || exit 1
  if ! curl -fsSL --proto '=https' --tlsv1.2 "${DCRUP_INSTALL_URL:-https://sh.dcr-tool.ru}" -o "$_dcrup_tmp"; then
    rm -f "$_dcrup_tmp"
    echo "[error] could not download dcrup to re-run it under bash" >&2
    exit 1
  fi
  exec bash -c 'f=$1; shift; bash "$f" "$@"; r=$?; rm -f "$f"; exit $r' dcrup "$_dcrup_tmp" "$@"
fi
set -Eeuo pipefail

DCRUP_HOME="${DCRUP_HOME:-$HOME/.dcr}"
DCRUP_BIN="${DCRUP_BIN:-$DCRUP_HOME/bin}"
DCRUP_TC="${DCRUP_HOME}/toolchains"
DCRUP_META="${DCRUP_HOME}/meta"
REPO_URL="${DCRUP_REPO:-https://github.com/dexoron/dcr}"
DCRUP_INSTALL_URL="${DCRUP_INSTALL_URL:-https://sh.dcr-tool.ru}"
DCRUP_MIRROR="${DCRUP_MIRROR:-auto}"
API_LATEST="https://api.github.com/repos/dexoron/dcr/releases/latest"
API_ALL="https://api.github.com/repos/dexoron/dcr/releases"
FEATURES="${DCR_FEATURES:-archive}"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

DCRUP_QUIET="${DCRUP_QUIET:-0}"
log() { [[ "$DCRUP_QUIET" == "1" ]] || printf "${CYAN}[dcrup]${NC} %s\n" "$*"; }
ok() { [[ "$DCRUP_QUIET" == "1" ]] || printf "${GREEN}[ok]${NC} %s\n" "$*"; }
# Warnings go to stderr: callers capture stdout with $(...), and a warning
# must never become part of a tag, JSON body, or other captured value.
warn() { printf "${YELLOW}[warn]${NC} %s\n" "$*" >&2; }
die() {
  printf "${RED}[error]${NC} %s\n" "$*" >&2
  exit 1
}

usage() {
  cat <<'EOF'
dcrup — install and switch DCR versions

Usage:
  dcrup install <spec> [--build|--release] [--force] [-q] [--libc gnu|musl|auto] [--mirror auto|cf|gh|sf|ya]
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
  --release   download prebuilt binary (default for stable/dev)
  --build     cargo build --release --features archive
  night       always --build (prebuilt not available)

Linux libc (prebuilt triple only):
  --libc gnu    x86_64-unknown-linux-gnu (default)
  --libc musl   x86_64-unknown-linux-musl
  --libc auto   musl if host ldd/libc is musl, else gnu

Windows (Git Bash / MSYS2 / Cygwin):
  prebuilt assets and bin entries carry the .exe suffix; --libc is ignored.

Mirrors (prebuilt download only):
  --mirror auto   try cf → gh → sf → ya, first successful download wins (default)
  --mirror cf     download from cdn-global.dcr-tool.ru
  --mirror gh     download from github.com/dexoron/dcr
  --mirror sf     download from sourceforge.net/projects/dcr-tool
  --mirror ya     download from cdn-ru.dcr-tool.ru
  DCRUP_MIRROR    env override (default: auto)

Env:
  DCRUP_HOME      default ~/.dcr
  DCRUP_BIN       default $DCRUP_HOME/bin
  DCR_FEATURES    default archive
  DCRUP_REPO      git/GitHub repo URL
  DCRUP_LIBC      gnu|musl|auto (default: gnu)
  DCRUP_MIRROR    auto|cf|gh|sf|ya (default: auto)
  DCRUP_VERSION   spec used when no spec is given (default: stable)
  DCRUP_QUIET=1   only warnings and errors
EOF
}

need_cmd() {
  command -v "$1" >/dev/null 2>&1 || die "required command not found: $1"
}

detect_linux_libc() {
  local pref="${DCRUP_LIBC:-gnu}"
  case "$pref" in
  gnu | glibc) echo "gnu" ;;
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
  Linux:aarch64 | Linux:arm64)
    libc="$(detect_linux_libc)"
    if [[ "$libc" == "musl" ]]; then
      echo "aarch64-unknown-linux-musl"
    else
      echo "aarch64-unknown-linux-gnu"
    fi
    ;;
  Linux:i686 | Linux:i386)
    libc="$(detect_linux_libc)"
    if [[ "$libc" == "musl" ]]; then
      echo "i686-unknown-linux-musl"
    else
      echo "i686-unknown-linux-gnu"
    fi
    ;;
  Linux:armv7l | Linux:armv7)
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
  Darwin:x86_64) echo "x86_64-apple-darwin" ;;
  Darwin:arm64 | Darwin:aarch64) echo "aarch64-apple-darwin" ;;
  FreeBSD:x86_64 | FreeBSD:amd64) echo "x86_64-unknown-freebsd" ;;
  FreeBSD:aarch64 | FreeBSD:arm64) echo "aarch64-unknown-freebsd" ;;
  OpenBSD:x86_64 | OpenBSD:amd64) echo "x86_64-unknown-openbsd" ;;
  NetBSD:x86_64 | NetBSD:amd64) echo "x86_64-unknown-netbsd" ;;
  MINGW* | MSYS* | CYGWIN*)
    case "$arch" in
    x86_64 | amd64) echo "x86_64-pc-windows-gnu" ;;
    aarch64 | arm64) echo "aarch64-pc-windows-msvc" ;;
    i686 | i386) echo "i686-pc-windows-msvc" ;;
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

exe_suffix_for_triple() {
  case "$1" in
  *windows*) echo ".exe" ;;
  *) echo "" ;;
  esac
}

host_exe_suffix() {
  case "$(uname -s)" in
  MINGW* | MSYS* | CYGWIN*) echo ".exe" ;;
  *) echo "" ;;
  esac
}

parse_spec() {
  local raw="$1"
  SPEC_RAW="$raw"
  SPEC_CHANNEL=""
  SPEC_VERSION=""
  SPEC_FLOATING=0

  case "$raw" in
  "") die "missing version/channel spec" ;;
  stable | dev | night)
    SPEC_CHANNEL="$raw"
    SPEC_VERSION=""
    SPEC_FLOATING=1
    ;;
  *@*)
    SPEC_VERSION="$(normalize_version "${raw%@*}")"
    SPEC_CHANNEL="${raw#*@}"
    case "$SPEC_CHANNEL" in
    stable | dev) ;;
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
  if [[ -n "$SPEC_VERSION" ]] && ! [[ "$SPEC_VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+([-.][0-9A-Za-z.-]+)?$ ]]; then
    die "invalid version: $raw (expected e.g. 0.9.5)"
  fi
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

# Returns the newest installed toolchain id for a floating channel.
# Versions are compared numerically (0.10.0 > 0.9.5), night by mtime.
latest_toolchain_for_channel() {
  local channel="$1" d base best="" best_key="" key ver core major minor patch exe
  exe="$(host_exe_suffix)"
  if [[ "$channel" == "night" ]]; then
    while IFS= read -r d; do
      if [[ -n "$d" && -x "$d/dcr$exe" ]]; then
        basename "$d"
        return 0
      fi
    done < <(ls -1dt "$DCRUP_TC"/night-* 2>/dev/null || true)
    return 0
  fi
  for d in "$DCRUP_TC/$channel"-*; do
    [[ -d "$d" && -x "$d/dcr$exe" ]] || continue
    base="$(basename "$d")"
    ver="${base#"$channel"-}"
    core="${ver%%-*}"
    IFS=. read -r major minor patch <<<"$core"
    [[ "$major" =~ ^[0-9]+$ && "$minor" =~ ^[0-9]+$ && "$patch" =~ ^[0-9]+$ ]] || continue
    key="$(printf '%010d%010d%010d' "$major" "$minor" "$patch")"
    if [[ -z "$best" || "$key" > "$best_key" ]]; then
      best="$base"
      best_key="$key"
    fi
  done
  [[ -n "$best" ]] && printf '%s' "$best"
  return 0
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
  local id="$1" exe link
  exe="$(host_exe_suffix)"
  local src="$DCRUP_TC/$id/dcr$exe"
  [[ -x "$src" ]] || die "toolchain not installed: $id"
  # Resolve the source so a relative DCRUP_HOME cannot create a broken link.
  src="$(cd "$(dirname "$src")" && pwd)/$(basename "$src")"
  link="$DCRUP_BIN/dcr$exe"
  # Drop the other naming variant when switching between Unix and Windows bash.
  if [[ "$exe" == ".exe" ]]; then
    rm -f "$DCRUP_BIN/dcr"
  else
    rm -f "$DCRUP_BIN/dcr.exe"
  fi
  if ! ln -sfn "$src" "$link" 2>/dev/null; then
    cp "$src" "$link"
  fi
  write_active "$id"
  ok "default → $id ($link)"
}

fetch_json() {
  need_cmd curl
  curl -fsSL "$1"
}

# One JSON field per line (works on compact and pretty JSON).
json_lines() {
  printf '%s' "$1" | tr ',{}' '\n\n\n' | sed 's/"[[:space:]]*:[[:space:]]*/":/'
}

# First tag_name (for a release list: the newest release).
json_tag() {
  json_lines "$1" | sed -n 's/^[[:space:]]*"tag_name":"\([^"]*\)".*/\1/p' | head -n1
}

# SHA256 that GitHub publishes for an asset ("digest"), or empty.
asset_digest() {
  json_lines "$1" | awk -v want="\"name\":\"$2\"" '
    index($0, want) { f = 1; next }
    f && /"digest":"sha256:/ { sub(/.*sha256:/, ""); sub(/".*/, ""); print; exit }
    f && /"browser_download_url"/ { exit }'
}

file_sha256() {
  if command -v sha256sum >/dev/null 2>&1; then sha256sum "$1" | awk '{print $1}'
  elif command -v shasum >/dev/null 2>&1; then shasum -a 256 "$1" | awk '{print $1}'
  else echo ""; fi
}

set_mirror() {
  case "$1" in
  auto | cf | gh | sf | ya) DCRUP_MIRROR="$1" ;;
  *) die "invalid --mirror: $1 (use auto|cf|gh|sf|ya)" ;;
  esac
}

mirror_url() {
  local mirror="$1" tag="$2" asset="$3"
  case "$mirror" in
  cf) printf 'https://cdn-global.dcr-tool.ru/%s/%s' "$tag" "$asset" ;;
  gh) printf 'https://github.com/dexoron/dcr/releases/download/%s/%s' "$tag" "$asset" ;;
  sf) printf 'https://sourceforge.net/projects/dcr-tool/files/%s/%s/download' "$tag" "$asset" ;;
  ya) printf 'https://cdn-ru.dcr-tool.ru/%s/%s' "$tag" "$asset" ;;
  *) return 1 ;;
  esac
}

mirror_chain() { echo "cf gh sf ya"; }

mirrors_to_try() {
  case "$DCRUP_MIRROR" in
  auto) mirror_chain ;;
  cf | gh | sf | ya) echo "$DCRUP_MIRROR" ;;
  *) die "invalid mirror: $DCRUP_MIRROR (use auto|cf|gh|sf|ya)" ;;
  esac
}

download_from_mirrors() {
  local tag="$1" asset="$2" dest="$3" want_sha="${4:-}" binary="${5:-dcr}"
  local tmp="$dest/dcr.download" m url size got
  USED_MIRROR=""
  for m in $(mirrors_to_try); do
    url="$(mirror_url "$m" "$tag" "$asset")"
    log "trying mirror $m …"
    rm -f "$tmp"
    if curl -fsSL --retry 2 --retry-delay 1 --connect-timeout 10 --max-time 180 \
      -A "dcrup" "$url" -o "$tmp"; then
      size="$(wc -c <"$tmp" | tr -d ' ')"
      if LC_ALL=C head -c 512 "$tmp" | LC_ALL=C grep -aqiE '<(!doctype|html)'; then
        warn "mirror $m returned an HTML page, not a binary"
      elif [[ -n "$want_sha" && "$(file_sha256 "$tmp")" != "$want_sha" && -n "$(file_sha256 "$tmp")" ]]; then
        got="$(file_sha256 "$tmp")"
        warn "mirror $m: SHA256 mismatch (got $got)"
      elif [[ "${size:-0}" -gt 100000 ]]; then
        mv "$tmp" "$dest/$binary"
        USED_MIRROR="$m"
        return 0
      else
        warn "mirror $m returned a too-small file (${size:-0} bytes)"
      fi
    else
      [[ "$DCRUP_QUIET" == "1" ]] || warn "mirror $m failed"
    fi
    rm -f "$tmp"
  done
  return 1
}

latest_stable_json() {
  fetch_json "$API_LATEST"
}

# dev = the newest published release (GitHub lists newest first). An old
# pre-release must never win over a newer stable release.
latest_dev_json() {
  fetch_json "$API_ALL?per_page=10"
}

release_tag_candidates() {
  if [[ "$SPEC_CHANNEL" == "dev" ]]; then
    # Prefer the development tag; fall back to a stable tag for versions that
    # never got a -dev release (keeps `VERSION@dev` usable).
    printf '%s\n' "v${SPEC_VERSION}-dev" "v${SPEC_VERSION}"
  else
    printf '%s\n' "v${SPEC_VERSION}"
  fi
}

# Resolves a pinned tag from the git remote (no GitHub API required).
# For VERSION@dev prefer vVERSION-dev, then fall back to vVERSION.
pinned_git_tag() {
  local tag first="" refs
  while IFS= read -r tag; do
    [[ -n "$first" ]] || first="$tag"
    # Avoid `git ... | grep -q .`: with `pipefail` a SIGPIPE can make a
    # matching tag look missing.
    refs="$(git ls-remote --tags "$REPO_URL" "refs/tags/$tag" 2>/dev/null || true)"
    if [[ -n "$refs" ]]; then
      if [[ "$tag" != "$first" ]]; then
        warn "tag $first not found; falling back to $tag"
      fi
      printf '%s' "$tag"
      return 0
    fi
  done < <(release_tag_candidates)
  printf '%s' "$first"
}

release_json_for_spec() {
  if [[ "$SPEC_FLOATING" -eq 1 ]]; then
    case "$SPEC_CHANNEL" in
    stable) latest_stable_json ;;
    dev) latest_dev_json ;;
    night) die "night has no GitHub release binary" ;;
    esac
    return 0
  fi

  local tag json first="" tried=""
  while IFS= read -r tag; do
    tried="${tried}${tried:+, }$tag"
    if json="$(fetch_json "https://api.github.com/repos/dexoron/dcr/releases/tags/$tag" 2>/dev/null)"; then
      if [[ -n "$first" ]]; then
        warn "release $first not found; falling back to $tag"
      fi
      printf '%s' "$json"
      return 0
    fi
    [[ -n "$first" ]] || first="$tag"
  done < <(release_tag_candidates)
  die "release not found (tried: $tried; see $REPO_URL/releases)"
}

install_prebuilt() {
  local triple asset tag version id dest mirror exe created=0
  need_cmd curl
  triple="$(detect_triple)"
  exe="$(exe_suffix_for_triple "$triple")"
  json="$(release_json_for_spec)"
  tag="$(json_tag "$json")"
  [[ -n "$tag" ]] || die "failed to parse release tag"
  version="$(normalize_version "$tag")"
  asset="dcr-${triple}-${version}${exe}"

  id="$(toolchain_id "$version")"
  dest="$DCRUP_TC/$id"
  if [[ -x "$dest/dcr$exe" && "${FORCE:-0}" != "1" ]]; then
    ok "already installed: $id"
    link_default "$id"
    return 0
  fi

  if [[ ! -x "$dest/dcr$exe" ]]; then
    # A failed earlier download can leave an empty/partial directory. Clear it
    # so `dcrup list/default/uninstall` never see a toolchain without a binary.
    [[ -d "$dest" ]] && rm -rf "$dest"
    mkdir -p "$dest"
    created=1
  fi
  log "downloading $asset (mirror=$DCRUP_MIRROR) …"
  local sha
  sha="$(asset_digest "$json" "$asset")"
  [[ -n "$sha" ]] || warn "release has no published SHA256; checked by file type only"
  if ! download_from_mirrors "$tag" "$asset" "$dest" "$sha" "dcr$exe"; then
    [[ "$created" -eq 1 ]] && rm -rf "$dest"
    die "no mirror reachable for $asset (try: --mirror cf|gh|sf|ya)"
  fi
  mirror="$USED_MIRROR"
  log "mirror → $mirror"
  chmod +x "$dest/dcr$exe"
  if ! "$dest/dcr$exe" --version >/dev/null 2>&1; then
    rm -rf "$dest"
    die "downloaded dcr does not run on this system ($triple); try --libc musl or --build"
  fi
  printf '%s\n' \
    "sha256=${sha:-$(file_sha256 "$dest/dcr$exe")}" \
    "channel=$SPEC_CHANNEL" \
    "version=$version" \
    "tag=$tag" \
    "source=release" \
    "triple=$triple" \
    "mirror=$mirror" \
    "libc=${DCRUP_LIBC:-gnu}" \
    >"$dest/dcrup-meta"
  ok "installed prebuilt $id ($triple via $mirror)"
  link_default "$id"
}

install_build() {
  local tmp id dest sha version exe
  need_cmd git
  need_cmd cargo

  tmp="$(mktemp -d "${TMPDIR:-/tmp}/dcrup.XXXXXX")"
  # `tmp` is local; the EXIT trap can run after the function scope is gone.
  cleanup_tmp() { if [[ -n "${tmp:-}" ]]; then rm -rf "$tmp"; fi; }
  trap cleanup_tmp EXIT

  if [[ "$SPEC_CHANNEL" == "night" ]]; then
    log "cloning $REPO_URL (branch dev) for night …"
    git clone --depth 1 --branch dev "$REPO_URL" "$tmp/src" 2>/dev/null ||
      git clone --depth 1 "$REPO_URL" "$tmp/src" ||
      die "git clone of $REPO_URL failed"
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
    git clone --depth 1 --branch "$tag" "$REPO_URL" "$tmp/src" 2>/dev/null ||
      {
        git clone --depth 1 "$REPO_URL" "$tmp/src" &&
          git -C "$tmp/src" fetch --depth 1 origin "refs/tags/${tag}:refs/tags/${tag}" &&
          git -C "$tmp/src" checkout "$tag"
      } ||
      die "git clone of $tag failed"
    sha="$(git -C "$tmp/src" rev-parse --short HEAD)"
  else
    version="$SPEC_VERSION"
    id="$(toolchain_id)"
    local tag
    tag="$(pinned_git_tag)"
    [[ -n "$tag" ]] || die "failed to resolve pinned tag"
    version="$(normalize_version "$tag")"
    log "cloning $REPO_URL @ $tag …"
    git clone --depth 1 --branch "$tag" "$REPO_URL" "$tmp/src" 2>/dev/null ||
      {
        git clone --depth 1 "$REPO_URL" "$tmp/src" &&
          git -C "$tmp/src" fetch --depth 1 origin "refs/tags/${tag}:refs/tags/${tag}" &&
          git -C "$tmp/src" checkout "$tag"
      } ||
      die "git clone of $tag failed"
    sha="$(git -C "$tmp/src" rev-parse --short HEAD)"
  fi

  exe="$(host_exe_suffix)"
  dest="$DCRUP_TC/$id"
  if [[ -x "$dest/dcr$exe" && "${FORCE:-0}" != "1" ]]; then
    ok "already installed: $id"
    link_default "$id"
    return 0
  fi

  log "building release (features: $FEATURES) …"
  (
    cd "$tmp/src"
    if [[ -n "$FEATURES" ]]; then
      cargo build --release --features "$FEATURES"
    else
      cargo build --release
    fi
  )
  mkdir -p "$dest"
  cp "$tmp/src/target/release/dcr$exe" "$dest/dcr$exe"
  chmod +x "$dest/dcr$exe"
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
    --build)
      mode="build"
      shift
      ;;
    --release)
      mode="release"
      shift
      ;;
    --force)
      FORCE=1
      shift
      ;;
    --libc)
      [[ $# -ge 2 ]] || die "--libc needs gnu|musl|auto"
      libc_cli="$2"
      shift 2
      ;;
    --libc=*)
      libc_cli="${1#--libc=}"
      shift
      ;;
    --mirror)
      [[ $# -ge 2 ]] || die "--mirror needs auto|cf|gh|sf|ya"
      set_mirror "$2"
      shift 2
      ;;
    --mirror=*)
      set_mirror "${1#--mirror=}"
      shift
      ;;
    -q | --quiet)
      DCRUP_QUIET=1
      shift
      ;;
    -y | --yes) shift ;;
    -h | --help)
      usage
      exit 0
      ;;
    -*) die "unknown flag: $1" ;;
    *)
      [[ -z "$spec" ]] || die "unexpected argument: $1"
      spec="$1"
      shift
      ;;
    esac
  done
  spec="${spec:-${DCRUP_VERSION:-stable}}"
  [[ -n "$spec" ]] || die "usage: dcrup install <spec> [--build|--release] [--force] [--libc gnu|musl|auto] [--mirror auto|cf|gh|sf|ya]"
  parse_spec "$spec"
  ensure_dirs

  if [[ -n "$libc_cli" ]]; then
    case "$libc_cli" in
    gnu | glibc | musl | auto) DCRUP_LIBC="$libc_cli" ;;
    *) die "invalid --libc: $libc_cli (use gnu|musl|auto)" ;;
    esac
  fi
  export DCRUP_LIBC="${DCRUP_LIBC:-gnu}"

  if [[ "$SPEC_CHANNEL" == "night" ]]; then
    mode="build"
  fi

  export FORCE
  log "install spec=$SPEC_RAW channel=$SPEC_CHANNEL version=${SPEC_VERSION:-∅} mode=$mode libc=$DCRUP_LIBC mirror=$DCRUP_MIRROR"
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
  local id exe alt
  exe="$(host_exe_suffix)"
  if [[ "$SPEC_CHANNEL" == "night" || "$SPEC_FLOATING" -eq 1 ]]; then
    id="$(latest_toolchain_for_channel "$SPEC_CHANNEL")"
    [[ -n "$id" ]] || die "no installed toolchain for channel $SPEC_CHANNEL (run: dcrup install $SPEC_CHANNEL)"
  else
    id="$(toolchain_id)"
    if [[ ! -x "$DCRUP_TC/$id/dcr$exe" ]]; then
      # `dcrup install stable` stores the same version as stable-<version>;
      # let a plain `dcrup default <version>` find that floating install.
      alt="${SPEC_CHANNEL}-${SPEC_VERSION}"
      if [[ -x "$DCRUP_TC/$alt/dcr$exe" ]]; then
        id="$alt"
      fi
    fi
    [[ -x "$DCRUP_TC/$id/dcr$exe" ]] || die "not installed: $id (run: dcrup install $1)"
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
  stable-* | dev-*)
    local ch="${active%%-*}"
    log "updating floating channel $ch …"
    FORCE=1 cmd_install "$ch" --force
    ;;
  *-stable | *-dev)
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
  local d base exe count=0
  exe="$(host_exe_suffix)"
  for d in "$DCRUP_TC"/*; do
    [[ -d "$d" && -x "$d/dcr$exe" ]] || continue
    count=$((count + 1))
    base="$(basename "$d")"
    if [[ "$base" == "$active" ]]; then
      printf ' * %s\n' "$base"
    else
      printf '   %s\n' "$base"
    fi
  done
  [[ "$count" -gt 0 ]] || warn "no toolchains installed"
}

cmd_show() {
  ensure_dirs
  local active exe
  active="$(read_active)"
  exe="$(host_exe_suffix)"
  echo "DCRUP_HOME=$DCRUP_HOME"
  echo "DCRUP_BIN=$DCRUP_BIN"
  echo "active=${active:-none}"
  if [[ -n "$active" && -f "$DCRUP_TC/$active/dcrup-meta" ]]; then
    echo "--- meta ---"
    cat "$DCRUP_TC/$active/dcrup-meta"
  fi
  if [[ -x "$DCRUP_BIN/dcr$exe" ]]; then
    echo "--- dcr --version ---"
    "$DCRUP_BIN/dcr$exe" --version 2>/dev/null || true
  fi
}

cmd_which() {
  local exe
  exe="$(host_exe_suffix)"
  if [[ -L "$DCRUP_BIN/dcr$exe" || -e "$DCRUP_BIN/dcr$exe" ]]; then
    readlink -f "$DCRUP_BIN/dcr$exe" 2>/dev/null || readlink "$DCRUP_BIN/dcr$exe" 2>/dev/null || echo "$DCRUP_BIN/dcr$exe"
  else
    die "dcr not linked (run: dcrup install stable)"
  fi
}

cmd_uninstall() {
  ensure_dirs
  if [[ "${1:-}" == "--all" ]]; then
    rm -rf "$DCRUP_TC"
    rm -f "$DCRUP_BIN/dcr" "$DCRUP_BIN/dcr.exe" "$(active_file)"
    ok "removed all toolchains"
    return 0
  fi
  [[ $# -eq 1 ]] || die "usage: dcrup uninstall <spec>|--all"
  parse_spec "$1"
  local id exe alt next="" d
  exe="$(host_exe_suffix)"
  if [[ "$SPEC_FLOATING" -eq 1 || "$SPEC_CHANNEL" == "night" ]]; then
    id="$(latest_toolchain_for_channel "$SPEC_CHANNEL")"
  else
    id="$(toolchain_id)"
    if [[ ! -x "$DCRUP_TC/$id/dcr$exe" ]]; then
      alt="${SPEC_CHANNEL}-${SPEC_VERSION}"
      if [[ -x "$DCRUP_TC/$alt/dcr$exe" ]]; then
        id="$alt"
      fi
    fi
  fi
  [[ -n "$id" && -x "$DCRUP_TC/$id/dcr$exe" ]] || die "not installed: $1"
  rm -rf "$DCRUP_TC/$id"
  if [[ "$(read_active)" == "$id" ]]; then
    while IFS= read -r d; do
      if [[ -d "$d" && -x "$d/dcr$exe" ]]; then
        next="$(basename "$d")"
        break
      fi
    done < <(ls -1dt "$DCRUP_TC"/* 2>/dev/null || true)
    if [[ -n "$next" ]]; then
      link_default "$next"
    else
      rm -f "$DCRUP_BIN/dcr" "$DCRUP_BIN/dcr.exe" "$(active_file)"
      warn "removed active toolchain; run: dcrup install stable"
    fi
  fi
  ok "removed $id"
}

check_path() {
  local dir="${1:-$DCRUP_BIN}"
  case ":$PATH:" in
  *":$dir:"*) ;;
  *)
    warn "$dir is not in PATH"
    echo "Add to ~/.bashrc / ~/.zshrc / ~/.profile:"
    echo "  export PATH=\"$dir:\$PATH\""
    ;;
  esac
}

cmd_self_install() {
  local dest="$DCRUP_BIN"
  while [[ $# -gt 0 ]]; do
    case "$1" in
    --to)
      [[ $# -ge 2 ]] || die "--to needs a directory"
      dest="$2"
      shift 2
      ;;
    *) die "usage: dcrup self-install [--to DIR]" ;;
    esac
  done
  ensure_dirs
  mkdir -p "$dest"
  local self target
  self="$(readlink -f "$0" 2>/dev/null || realpath "$0" 2>/dev/null || true)"
  if [[ -z "$self" || ! -f "$self" ]]; then
    # When invoked as `curl ... | sh`, the shell has no script file to copy.
    # Fetch the same trusted installer URL and install that exact script.
    need_cmd curl
    self="$(mktemp "${TMPDIR:-/tmp}/dcrup.XXXXXX")"
    curl -fsSL --proto '=https' --tlsv1.2 "$DCRUP_INSTALL_URL" -o "$self" || {
      rm -f "$self"
      die "unable to download installer for self-install"
    }
    trap 'rm -f "$self"' EXIT
  fi
  target="$dest/dcrup"
  if [[ -e "$target" && "$self" -ef "$target" ]]; then
    ok "dcrup is already installed at $target"
    check_path "$dest"
    return 0
  fi
  cp "$self" "$target"
  chmod +x "$target"
  ok "dcrup installed to $target"
  check_path "$dest"
  echo "Then: dcrup install stable"
}

main() {
  local cmd="${1:-}"
  shift || true
  case "$cmd" in
  "")
    if [[ -f "$0" ]] && grep -q 'DCRUP_HOME' "$0" 2>/dev/null; then usage; else cmd_install; cmd_self_install; fi
    ;;
  -h | --help | help) usage ;;
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
