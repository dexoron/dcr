# Build profiles

DCR supports two profiles:

- `debug` (default)
- `release`

## How to select

```sh
dcr build --debug
dcr build --release
dcr run --release
dcr clean --debug
```

## Default profile flags

### GCC/Clang-like toolchains

- `debug`: `-O0 -g -Wall -Wextra -fno-omit-frame-pointer -DDEBUG`
- `release`: `-O3 -DNDEBUG -march=native`

### MSVC toolchain

- `debug`: `/Od /Zi /W4 /DDEBUG /Oy-`
- `release`: `/O2 /DNDEBUG`

## Artifacts

By default:

- `target/debug/<name>` (or `.exe` on Windows)
- `target/release/<name>` (or `.exe` on Windows)

Default compiler flags:

GCC/Clang:

- `debug`: `-O0 -g -Wall -Wextra -fno-omit-frame-pointer -DDEBUG`
- `release`: `-O3 -DNDEBUG -march=native`

MSVC:

- `debug`: `/Od /Zi /W4 /DDEBUG /Oy-`
- `release`: `/O2 /DNDEBUG`

Static library mode (`kind = "staticlib"`):

- Linux/macOS: `lib<name>.a`
- Windows: `<name>.lib`

Shared library mode (`kind = "sharedlib"`):

- Linux: `lib<name>.so`
- macOS: `lib<name>.dylib`
- Windows: `<name>.dll`
