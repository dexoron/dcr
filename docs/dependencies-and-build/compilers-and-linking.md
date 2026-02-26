# Compilers & linking

## Compiler backend selection

DCR picks a backend from `build.compiler`:

- contains `clang-cl` -> MSVC backend
- contains `gcc` or `g++` -> GCC backend
- contains `clang` or `clang++` -> Clang backend
- equals `cl` or contains `msvc` -> MSVC backend
- otherwise -> Clang backend fallback

## Compilation model

- Source files are found recursively in `src/` based on `build.language`.
- Each source is compiled to `target/<profile>/obj/<relative-path>.o` (`.obj` on MSVC).
- Recompile happens when source mtime is newer than object mtime.

## Linking

For `kind = "bin"`:

- Object files are linked into executable artifact.
- Dependency lib dirs and library names are passed to linker.
- `build.ldflags` are appended to link command.

For `kind = "staticlib"`:

- Linux/macOS (GCC/Clang): `ar rcs` creates `.a` archive.
- Windows (MSVC): `lib` creates `.lib` archive.
