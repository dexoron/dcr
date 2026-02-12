#!/usr/bin/env bash

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BASE_DIR="src"

echo "#============#"
echo "#    Ruff    #"
echo "#============#"
(cd "$SCRIPT_DIR" && ruff check "$BASE_DIR")
echo
echo
echo "#============#"
echo "#     Ty     #"
echo "#============#"
(cd "$SCRIPT_DIR" && uvx ty check "$BASE_DIR")
