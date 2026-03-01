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
- Each source is compiled to `target/<profile>/obj/<relative-path>.o` (`.obj` on MSVC).
- Recompile happens when source mtime is newer than object mtime.

## Default build flags

These flags are applied automatically based on profile and compiler backend.

GCC/Clang:

- `debug`: `-O0 -g -Wall -Wextra -fno-omit-frame-pointer -DDEBUG`
- `release`: `-O3 -DNDEBUG -march=native`

MSVC:

- `debug`: `/Od /Zi /W4 /DDEBUG /Oy-`
- `release`: `/O2 /DNDEBUG`

## Platform hint

If `build.platform` is set, DCR passes it as an architecture hint:

- GCC/Clang: `-march=<platform>`
- MSVC: maps to `/arch:*` for known values (`x86`, `i386`, `i486`, `i586`, `i686`, `sse2`, `avx`, `avx2`)

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
