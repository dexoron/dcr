#!/usr/bin/env bash

set -Eeuo pipefail

TMPDIR="/tmp/dcr-temp"
INSTALL_PATH="$HOME/.local/share/dcr"
PYTHON_DIR="$INSTALL_PATH/Python"
BINPATH="$HOME/.local/bin"
LOGFILE="$PWD/dcr-install.log"

PYTHON_DOWNLOAD_URL="https://github.com/astral-sh/python-build-standalone/releases/download/20260203/cpython-3.10.19+20260203-x86_64-unknown-linux-gnu-install_only.tar.gz"
GITHUB_API="https://api.github.com/repos/dexoron/dcr/releases/latest"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

exec > >(tee -a "$LOGFILE") 2>&1

log() {
    echo -e "${BLUE}[$(date '+%Y-%m-%d %H:%M:%S')]${NC} $1"
}

success() {
    echo -e "${GREEN}✔ $1${NC}"
}

warn() {
    echo -e "${YELLOW}⚠ $1${NC}"
}

error() {
    echo -e "${RED}✖ $1${NC}"
}

trap 'error "Ошибка на строке $LINENO"; exit 1' ERR

check_dependencies() {
    log "Проверка зависимостей..."

    command -v curl >/dev/null 2>&1 || {
        error "curl не установлен"
        exit 1
    }

    command -v tar >/dev/null 2>&1 || {
        error "tar не установлен"
        exit 1
    }

    success "Зависимости в порядке"
}

install_python() {
    log "Установка встроенного Python..."

    mkdir -p "$PYTHON_DIR"

    curl -L "$PYTHON_DOWNLOAD_URL" -o "$TMPDIR/python.tar.gz"
    tar -xf "$TMPDIR/python.tar.gz" -C "$PYTHON_DIR" --strip-components=1
    rm -f "$TMPDIR/python.tar.gz"

    cat > "$INSTALL_PATH/dcr" <<'EOF'
#!/usr/bin/env bash
BASE_DIR="$(dirname "$(readlink -f "$0")")"
SRC_DIR="$BASE_DIR/src"
PYTHON_BIN="$BASE_DIR/Python/bin/python3"
PYTHONPATH="$SRC_DIR" exec "$PYTHON_BIN" -m dcr.main "$@"
EOF

    chmod +x "$INSTALL_PATH/dcr"

    success "Python установлен"
}

install_app() {
    log "Получение последнего релиза..."

    TAG=$(curl -s "$GITHUB_API" | grep '"tag_name":' | cut -d'"' -f4)

    if [[ -z "$TAG" ]]; then
        error "Не удалось определить версию релиза"
        exit 1
    fi

    log "Найдена версия: $TAG"

    curl -L -o "$TMPDIR/sources.tar.gz" \
        "https://github.com/dexoron/dcr/archive/refs/tags/$TAG.tar.gz"

    mkdir -p "$INSTALL_PATH"
    tar -xf "$TMPDIR/sources.tar.gz" -C "$INSTALL_PATH" --strip-components=1
    rm -f "$TMPDIR/sources.tar.gz"

    success "Приложение установлено"
}

install_link() {
    log "Создание симлинка..."

    mkdir -p "$BINPATH"
    ln -sf "$INSTALL_PATH/dcr" "$BINPATH/dcr"

    success "Команда 'dcr' добавлена в $BINPATH"
}

main() {
    log "Запуск установки DCR"

    check_dependencies

    mkdir -p "$TMPDIR"
    rm -rf "$TMPDIR"/*

    install_app
    log "проверка наличия Python"
    command -v curl >/dev/null 2>&1 || {
        error "Python не установлен"
        install_python
    }
    install_link

    rm -rf "$TMPDIR"

    success "Установка завершена успешно"
    log "Лог сохранён в $LOGFILE"
}

main "$@"
