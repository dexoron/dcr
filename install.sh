#!/usr/bin/env bash
set -Eeuo pipefail

# ===== CONFIG =====
TMPDIR="/tmp/dcr-temp"
INSTALL_PATH="$HOME/.local/share/dcr"
PYTHON_DIR="$INSTALL_PATH/Python"
BINPATH="$HOME/.local/bin"
LOGFILE="$HOME/.cache/dcr-install.log"

PYTHON_DOWNLOAD_URL="https://github.com/astral-sh/python-build-standalone/releases/download/20260203/cpython-3.10.19+20260203-x86_64-unknown-linux-gnu-install_only.tar.gz"
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

check_dependencies() {
    log "Проверка зависимостей..."

    command -v curl >/dev/null 2>&1 || { error "curl не установлен"; exit 1; }
    command -v tar >/dev/null 2>&1 || { error "tar не установлен"; exit 1; }

    success "Зависимости в порядке"
}

check_os() {
    if [[ "$(uname -s)" != "Linux" ]]; then
        error "Поддерживается только Linux"
        exit 1
    fi
}

install_python() {
    log "Python не найден, установка встроенного Python..."

    mkdir -p "$PYTHON_DIR"
    mkdir -p "$TMPDIR"

    curl -fsSL "$PYTHON_DOWNLOAD_URL" -o "$TMPDIR/python.tar.gz"
    tar -xf "$TMPDIR/python.tar.gz" -C "$PYTHON_DIR" --strip-components=1
    rm -f "$TMPDIR/python.tar.gz"

    success "Встроенный Python установлен"
}

install_app() {
    log "Получение последнего релиза..."

    TAG=$(curl -fsSL "$GITHUB_API" | sed -n 's/.*"tag_name": "\(.*\)".*/\1/p')

    if [[ -z "$TAG" ]]; then
        error "Не удалось определить версию релиза"
        exit 1
    fi

    log "Найдена версия: $TAG"

    mkdir -p "$TMPDIR"
    curl -fsSL -o "$TMPDIR/sources.tar.gz" \
        "https://github.com/dexoron/dcr/archive/refs/tags/$TAG.tar.gz"

    mkdir -p "$INSTALL_PATH"
    tar -xf "$TMPDIR/sources.tar.gz" -C "$INSTALL_PATH" --strip-components=1
    rm -f "$TMPDIR/sources.tar.gz"

    success "Приложение установлено"
}

install_wrapper() {
    log "Создание launcher..."

    mkdir -p "$INSTALL_PATH"

    cat > "$INSTALL_PATH/dcr" <<'EOF'
#!/usr/bin/env bash
BASE_DIR="$(cd "$(dirname "$0")" && pwd)"

if command -v python3 >/dev/null 2>&1; then
    PYTHON_BIN="python3"
else
    PYTHON_BIN="$BASE_DIR/Python/bin/python3"
fi

PYTHONPATH="$BASE_DIR/src" exec "$PYTHON_BIN" -m dcr.main "$@"
EOF

    chmod +x "$INSTALL_PATH/dcr"

    success "Launcher создан"
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

main() {
    log "Запуск установки DCR"

    check_os
    check_dependencies

    cleanup

    install_app

    if ! command -v python3 >/dev/null 2>&1; then
        install_python
    else
        success "Найден системный Python"
    fi

    install_wrapper
    install_link
    check_path
    cleanup

    success "Установка завершена успешно"
    log "Лог сохранён в $LOGFILE"
}

main "$@"
