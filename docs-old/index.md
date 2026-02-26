# DCR Documentation

DCR is a lightweight build tool for C/C++ projects with a Cargo-like workflow.

## Quick Start

```sh
dcr new hello
cd hello
dcr run
```

## Configuration

Project configuration lives in `dcr.toml`.
See `docs/dcr-toml.md` for the full schema and examples.

## Build Flags

You can add custom compile and link flags in `[build]`:

```toml
[build]
compiler = "gcc"
standard = "c99"
kind = "bin"
target = "./dist"
cflags = ["-D_POSIX_C_SOURCE=200809L", "-Wall", "-Wextra", "-O2", "-Ilib/frecli"]
ldflags = ["-Llib/frecli", "-lfrecli", "-lm"]
```

## Dependencies

Path dependencies can be declared in `[dependencies]` and are resolved on build.

```toml
[dependencies]
frecli = { path = "./lib/frecli", include = ["."], lib = ["."], libs = ["frecli"] }
```

## Source Discovery

DCR compiles all `.c`/`.cpp` files under `src/` recursively. The file containing `main()` can be in any subdirectory.

## Notes

- `dcr run` works only for `kind = "bin"`.
- Incremental rebuild currently tracks source file changes only (`.c/.cpp`).
