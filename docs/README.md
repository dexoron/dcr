# Introduction

DCR (Dexoron Cargo Realization) is a Cargo-style CLI for C/C++ projects.

Current DCR focuses on a simple, predictable workflow:

- create or initialize a project (`dcr new`, `dcr init`)
- build with `debug` or `release` profiles (`dcr build`)
- run binaries (`dcr run`)
- clean artifacts (`dcr clean`)
- update the installed binary (`dcr --update`)

## What is implemented in this version

- Project configuration in `dcr.toml` with `[package]`, `[build]`, `[dependencies]`.
- Recursive source discovery in `src/`:
  - C: `*.c`
  - C++: `*.cpp`, `*.cxx`, `*.cc`
- Compiler selection from `dcr.toml` (`clang`, `gcc`, `cl`, `clang-cl`, etc.).
- ASM support via `language = "asm"` (GCC/Clang, NASM, GAS).
- Three build kinds:
  - `bin` (default)
  - `staticlib`
  - `sharedlib`
- Optional custom output directory via `build.target`.
- Optional `build.platform` to pass architecture hints (`-march` / `/arch`).
- Path dependencies with include/lib resolution and `dcr.lock` generation.

## Important behavior notes

- `dcr.toml` is required for `build`, `run`, and `clean`.
- `run` works only with `build.kind = "bin"`.
- Incremental rebuild currently tracks source file timestamps (not header dependency graphs).
- Object files are always created under `target/<profile>/obj/`.

## Repository

- <https://github.com/dexoron/dcr>
