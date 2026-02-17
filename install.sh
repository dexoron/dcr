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

trap 'error "Ошибка на строке $LINENO"; exit 1' ERR

check_os() {
    case "$(uname -s)" in
        Linux|Darwin) ;;
        *) error "Поддерживаются только Linux и macOS"; exit 1 ;;
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
        *) error "Неподдерживаемая платформа: $os/$arch"; exit 1 ;;
    esac

    ASSET_NAME="dcr-${TARGET_TRIPLE}"
}

check_common_dependencies() {
    command -v curl >/dev/null 2>&1 || { error "curl не установлен"; exit 1; }
}

check_build_dependencies() {
    command -v git >/dev/null 2>&1 || { error "git не установлен"; exit 1; }
    command -v cargo >/dev/null 2>&1 || { error "cargo не установлен"; exit 1; }
}

prepare_sources() {
    log "Получение исходников..."
    rm -rf "$TMPDIR"
    git clone --depth 1 "$REPO_URL" "$TMPDIR"
    success "Исходники получены"
}

build_binary() {
    log "Сборка release-бинарника..."
    (cd "$TMPDIR" && cargo build --release)
    success "Сборка завершена"
}

fetch_latest_release_json() {
    curl -fsSL "$GITHUB_API"
}

download_binary() {
    log "Получение последнего релиза..."

    local release_json download_url tag
    release_json="$(fetch_latest_release_json)"

    tag="$(printf '%s\n' "$release_json" | sed -n 's/.*"tag_name": "\([^"]*\)".*/\1/p' | head -n1)"
    if [[ -z "$tag" ]]; then
        error "Не удалось определить версию релиза"
        exit 1
    fi

    download_url="$(printf '%s\n' "$release_json" | sed -n "s#.*\"browser_download_url\": \"\([^\"]*${ASSET_NAME}\)\".*#\1#p" | head -n1)"
    if [[ -z "$download_url" ]]; then
        error "Не найден ассет ${ASSET_NAME} в релизе ${tag}"
        exit 1
    fi

    mkdir -p "$INSTALL_PATH"
    curl -fL "$download_url" -o "$INSTALL_PATH/dcr"
    chmod +x "$INSTALL_PATH/dcr"
    success "Скачан бинарник ${ASSET_NAME} (${tag})"
}

install_built_binary() {
    mkdir -p "$INSTALL_PATH"
    cp "$TMPDIR/target/release/dcr" "$INSTALL_PATH/dcr"
    chmod +x "$INSTALL_PATH/dcr"
    success "Установлен бинарник из исходников"
}

install_link() {
    log "Создание симлинка..."
    mkdir -p "$BINPATH"
    ln -sf "$INSTALL_PATH/dcr" "$BINPATH/dcr"
    success "Команда 'dcr' добавлена в $BINPATH"
}

check_path() {
    if ! echo "$PATH" | grep -q "$BINPATH"; then
        warn "Каталог $BINPATH не найден в PATH"
        echo "Добавь в ~/.bashrc или ~/.zshrc:"
        echo "export PATH=\"$BINPATH:\$PATH\""
    fi
}

cleanup() {
    rm -rf "$TMPDIR" 2>/dev/null || true
}

select_install_mode() {
    echo "Выбери способ установки:"
    echo "  1) Скачать готовый бинарник из GitHub Release (рекомендуется)"
    echo "  2) Собрать из git"
    read -r -p "Введите 1 или 2 [1]: " choice

    case "${choice:-1}" in
        1) INSTALL_MODE="release" ;;
        2) INSTALL_MODE="build" ;;
        *) error "Неизвестный вариант"; exit 1 ;;
    esac
}

main() {
    log "Запуск установки DCR"

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

    success "Установка завершена успешно"
    log "Лог сохранён в $LOGFILE"
}

main "$@"
