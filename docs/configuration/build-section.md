# Build section

## Schema

```toml
[build]
language = "c"
standard = "c11"
compiler = "clang"
kind = "bin"
# optional:
# target = "./dist"
# platform = "x86_64"
# cflags = ["-Wall"]
# ldflags = ["-lm"]
```

## Fields

- `language` (string, required): `c`, `c++`, `cpp`, `cxx`, or `asm`.
- `standard` (string, required): language standard passed to compiler.
- `compiler` (string, required): compiler command (for example `clang`, `gcc`, `cl`).
- `kind` (string, required): `bin`, `staticlib`, or `sharedlib`.
- `target` (string, optional): custom output directory for final artifact.
- `platform` (string, optional): architecture hint for compiler (used for `-march` or `/arch`).
- `cflags` (string array, optional): extra compile flags.
- `ldflags` (string array, optional): extra link flags.

## Behavior notes

- Compiler backend is selected by compiler string (`gcc`, `clang`, `cl`, `clang-cl`).
- Empty or unknown compiler value falls back to clang-like build path.
- `run` is not allowed for library kinds (`staticlib`, `sharedlib`).
