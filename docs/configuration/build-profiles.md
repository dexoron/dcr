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

You can also add `--force` or `--clean` to `build`/`run`.

## Default profile flags

### GCC/Clang-like toolchains

- `debug`: `-O0 -g -Wall -Wextra -fno-omit-frame-pointer -DDCR_DEBUG`
- `release`: `-O3 -DNDEBUG`

### MSVC toolchain

- `debug`: `/Od /Zi /W4 /DDCR_DEBUG /Oy-`
- `release`: `/O2 /DNDEBUG`

## Artifacts

By default (for `kind = "bin"`):

- `target/<target>/debug/<name>` (or `.exe` on Windows)
- `target/<target>/release/<name>` (or `.exe` on Windows)

Where `<target>` is `native` or a target triple.  
Use `build.out_dir` to override the output directory.

Default compiler flags:

GCC/Clang:

- `debug`: `-O0 -g -Wall -Wextra -fno-omit-frame-pointer -DDCR_DEBUG`
- `release`: `-O3 -DNDEBUG`

MSVC:

- `debug`: `/Od /Zi /W4 /DDCR_DEBUG /Oy-`
- `release`: `/O2 /DNDEBUG`

Static library mode (`kind = "staticlib"`):

- Linux/macOS: `lib<name>.a`
- Windows: `<name>.lib`

Shared library mode (`kind = "sharedlib"`):

- Linux: `lib<name>.so`
- macOS: `lib<name>.dylib`
- Windows: `<name>.dll`

UEFI executable mode (`kind = "efi"`):

- All platforms: `<name>.efi`

Bare-metal ELF mode (`kind = "elf"`):

- All platforms: `<name>` (no extension)

## Profile overrides

Use `[build.debug]` and `[build.release]` to override build settings per profile.
All values inherit from `[build]`. If a field is present in the profile table, it replaces the base value.
For arrays (like `cflags`/`ldflags`), the profile value is appended to the base array.

```toml
[build.debug]
cflags = ["-g3"]

[build.release]
cflags = ["-O3"]
ldflags = ["-s"]
```
