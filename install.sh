#!/bin/bash

set -e

TMPDIR="./dcr-temp"
INSTALL_PATH="$PWD/dcr"
PYTHON_DIR="$INSTALL_PATH/Python"
BINPATH="$HOME/.local/bin"

install_python() {
    DOWNLOAD_URL="https://github.com/astral-sh/python-build-standalone/releases/download/20260203/cpython-3.10.19+20260203-x86_64-unknown-linux-gnu-install_only.tar.gz"

    curl -L -o "$TMPDIR/python.tar.gz" "$DOWNLOAD_URL"
    mkdir -p "$PYTHON_DIR"
    tar -xf "$TMPDIR/python.tar.gz" -C "$PYTHON_DIR" --strip-components=1
    rm -f "$TMPDIR/python.tar.gz"

    cat > "$INSTALL_PATH/dcr" <<'EOF'
#!/bin/bash
BASE_DIR="$(dirname "$(readlink -f "$0")")"
SRC_DIR="$BASE_DIR/src"
PYTHON_BIN="$BASE_DIR/Python/bin/python3"
PYTHONPATH="$SRC_DIR" exec "$PYTHON_BIN" -m dcr.main "$@"
EOF

    chmod +x "$INSTALL_PATH/dcr"
}

install_app() {
    TAG=$(curl -s https://api.github.com/repos/dexoron/dcr/releases/latest | grep '"tag_name":' | cut -d'"' -f4)

    curl -L -o "$TMPDIR/sources.tar.gz" "https://github.com/dexoron/dcr/archive/refs/tags/$TAG.tar.gz"

    mkdir -p "$INSTALL_PATH"
    tar -xf "$TMPDIR/sources.tar.gz" -C "$INSTALL_PATH" --strip-components=1
    rm -f "$TMPDIR/sources.tar.gz"
}

install_link() {
    mkdir -p "$BINPATH"
    ln -sf "$INSTALL_PATH/dcr" "$BINPATH/dcr"
}

main() {
    command -v curl >/dev/null 2>&1 || { echo "curl не установлен"; exit 1; }

    mkdir -p "$TMPDIR"
    rm -rf "$TMPDIR"/*

    install_app
    command -v curl >/dev/null 2>&1 || { echo "Python не установлен\nУстанавливаем Python..."; install_python; }
    install_link

    rm -rf "$TMPDIR"

    echo "Установка завершена"

}

main
