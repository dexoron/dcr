# dcr fmt

Formats all C/C++ source files using `clang-format`.

## Usage

```sh
dcr fmt
```

## Behavior

1. Scans `src/` and `tests/` directories recursively.
2. Formats all C/C++ source and header files (`.c`, `.cpp`, `.cxx`, `.cc`, `.h`, `.hpp`, `.hxx`, `.hh`) using `clang-format`.
3. Uses `.clang-format` if present in the project root, otherwise falls back to LLVM style.

## Requirements

- `clang-format` must be installed and available in PATH.
