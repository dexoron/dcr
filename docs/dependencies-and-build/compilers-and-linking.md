# Compilers & linking

## Compiler backend selection

DCR picks a backend from `build.compiler`:

- contains `clang-cl` -> MSVC backend
- equals `as` or contains `gas` -> GAS backend (standalone assembler)
- contains `gcc` or `g++` -> GCC backend
- contains `clang` or `clang++` -> Clang backend
- equals `cl` or contains `msvc` -> MSVC backend
- contains `nasm` -> NASM backend
- otherwise -> Clang backend fallback

## Compilation model

- Source files are found recursively in `src/` based on `build.language`.
- `build.roots` and `build.src_disable` can replace the default `src/` root.
- Mixed language builds use arrays, for example `build.language = ["c", "asm"]`.
- Each source is compiled to `target/<profile>/obj/<relative-path>.o` (`.obj` on MSVC).
- Recompile happens when source mtime is newer than object mtime.
- `build.exclude` removes paths from source/header collection; `build.include` re-allows matching paths and has priority over `exclude`.

## Default build flags

These flags are applied automatically based on profile and compiler backend.

GCC/Clang:

- `debug`: `-O0 -g -Wall -Wextra -fno-omit-frame-pointer -DDCR_DEBUG`
- `release`: `-O3 -DNDEBUG -march=native`

MSVC:

- `debug`: `/Od /Zi /W4 /DDCR_DEBUG /Oy-`
- `release`: `/O2 /DNDEBUG`

## Platform hint

If `build.platform` is set, DCR passes it as an architecture hint:

- GCC/Clang: `-march=<platform>`
- MSVC: maps to `/arch:*` for known values (`x86`, `i386`, `i486`, `i586`, `i686`, `sse2`, `avx`, `avx2`)

## Toolchain overrides

If `[toolchain]` is set, DCR uses its values to override compiler/linker tools:

- `cc`, `cxx`, `as` override compilers for C/C++/ASM
- `ar` overrides the archiver (used for `staticlib`)
- `ld` overrides the linker (used by GCC/Clang/NASM/GAS backends)
- `uic`, `moc`, `rcc` override Qt tools used in `build.steps`

Environment overrides (highest priority):
- `DCR_COMPILER` (all languages)
- `DCR_CC`, `DCR_CXX`, `DCR_AS`, `DCR_AR`, `DCR_LD`
- `DCR_DEBUG` enables verbose build logging

## Linking

For `kind = "bin"`:

- Object files are linked into executable artifact.
- Dependency lib dirs and library names are passed to linker.
- `build.ldflags` are appended to link command.

For `kind = "staticlib"`:

- Linux/macOS (GCC/Clang): `ar rcs` creates `.a` archive.
- Windows (MSVC): `lib` creates `.lib` archive.

For `kind = "sharedlib"`:

- Linux (GCC/Clang): `-shared` with output `.so`.
- macOS (Clang): `-dynamiclib` with output `.dylib`.
- Windows (MSVC): `/LD` with output `.dll`.
