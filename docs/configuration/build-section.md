# Build section

## Schema

```toml
[build]
language = "c"
standard = "c11"
compiler = "clang"
kind = "bin"
# filename = "KERNEL"
# extension = "BIN"
# optional:
# inherit = true  # Inherit from base (default true)
# targets = ["linux", "macos"]  # Multiple targets to build
# out_dir = "./dist"
# target = "x86_64-unknown-linux-gnu"
# platform = "x86_64"
# cflags = ["-Wall"]
# ldflags = ["-lm"]
# exclude = ["src/vendor", "src/legacy/**"]
# include = ["src/legacy/boot/**"]
# roots = ["kernel", "drivers"]
# src_disable = true
# pkg_config = ["Qt6Core", "Qt6Widgets"]
# generated = ["src/ui/*.h"]
# expect = ["./target/{profile}/boot/*.bin"]
# clean = ["./target/{profile}/boot", "/tmp/image/*.img"]
```

## Fields

- `language` (string or array, required): `c`, `c++`, `cpp`, `cxx`, or `asm`.
  Mixed languages can be specified as an array, for example `["c", "c++", "asm"]`.
- `standard` (string, required): C language standard passed to compiler (e.g. `c11`, `c17`, `gnu11`).
- `cxx_standard` (string, optional): C++ language standard passed to compiler for `.cpp`/`.cxx`/`.cc` files
  (e.g. `c++17`, `gnu++17`, `c++20`). When set, overrides `standard` for C++ sources.
- `compiler` (string, required): compiler command (for example `clang`, `gcc`, `cl`).
- `kind` (string, required): `bin`, `staticlib`, `sharedlib`, `efi`, or `elf`.
- `filename` (string, optional): custom output filename (without extension). Overrides `package.name` for the final artifact.
- `extension` (string, optional): custom file extension without the leading dot (e.g. `BIN`, `efi`, `EFI`). Combined with `filename`.
- `inherit` (bool, optional): inherit settings from base `[build]` (default `true`). If `false`, only use target/profile specific settings.
- `targets` (string array, optional): list of targets to build simultaneously. Supports short names (`linux`, `macos`, `windows`).
- `target` (string, optional): platform target triple (e.g. `x86_64-unknown-linux-gnu`) or short name (`linux`, `macos`, `windows`). Same as `--target` CLI flag.
- `out_dir` (string, optional): custom output directory for final artifact (overrides default `target/<target>/<profile>/`).
- `platform` (string, optional): architecture hint for compiler (used for `-march` or `/arch`).
- `cflags` (string array, optional): extra compile flags.
  Supports variable substitution — see [Variable substitution](#variable-substitution) below.
- `ldflags` (string array, optional): extra link flags.
  Supports variable substitution — see [Variable substitution](#variable-substitution) below.
- `ldscript` (string, optional): linker script path. Passed as `-T <path>` to the linker. Useful for bare-metal/embedded/freestanding targets.
- `exclude` (string array, optional): paths or glob patterns to skip during source/header collection.
- `include` (string array, optional): allowlist paths or globs that override `exclude`.
- `include` directory entries (non-glob) are also passed to the compiler as `-I` include paths.
- `roots` (string array, optional): additional source roots to scan instead of `src/`.
- `src_disable` (bool, optional): disables default `src/` root when true.
- `pkg_config` (string array, optional): packages used to append `pkg-config --cflags/--libs`.
- `generated` (string array, optional): generated files to delete when build steps rerun.
- `expect` (string array, optional): artifacts that must exist after build/post-steps.
- `clean` (string array, optional): additional paths removed by `dcr clean` and `--clean`.

Notes for `exclude`/`include`:
- Supports `*` and `**` glob patterns.
- If `include` is set, it has priority over `exclude`.
- Use `include` to re-allow nested paths, for example exclude `src/boot` but allow `src/boot/arch/**`.
When using glob patterns in `exclude`, DCR converts them internally, so `exclude` globs still work as expected.

### Variable substitution

`cflags`, `ldflags`, and `include` directory entries support the same `{placeholder}` variables
as build/post steps:

| Variable | Description |
|---|---|
| `{profile}` | Build profile (`debug` / `release`) |
| `{name}` | Package name |
| `{version}` | Full package version (e.g. `0.2.1`) |
| `{version_major}` | Major version number |
| `{version_minor}` | Minor version number |
| `{version_patch}` | Patch version number |
| `{version_suffix}` | Version suffix (e.g. `-rc1`) |
| `{version_suffix_dash}` | Suffix with leading dash (or empty) |

Example:

```toml
[build]
cflags = ['-DPROJECT_VERSION="{version}"', '-DVERSION_MAJOR={version_major}']
```

## Build steps

```toml
[[build.steps]]
name = "uic"
in = "src/ui/*.ui"
out = "src/ui/{stem}.h"
cmd = "{uic} {in} -o {out}"

[[build.post_steps]]
name = "image"
in = "target/{profile}/boot/*.bin"
out = "/tmp/image/os-{version}.img"
cmd = "dd if=/dev/zero of={out} bs=1M count=8"
```

Step notes:
- `in` supports glob patterns.
- If `in` expands to multiple files, `out` must include `{stem}`.
- Steps run only when inputs are newer than outputs.
- Variables: `{in}`, `{out}`, `{stem}`, `{cflags}`, `{uic}`, `{moc}`, `{rcc}`, `{profile}`,
  `{version}`, `{version_major}`, `{version_minor}`, `{version_patch}`, `{version_suffix}`, `{version_suffix_dash}`.

`build.clean` supports `{profile}` and version placeholders (`{version}`, `{version_major}`, ...).

### Profile overrides

Profile tables inherit all values from `[build]`. Any field present in the profile table replaces the base value.
Array fields (like `cflags`/`ldflags`) are appended to the base array.

Target tables override profile and base settings for specific targets. Full inheritance order:
1. `[build.<target>.<profile>]`
2. `[build.<profile>.<target>]`
3. `[build.<target>]`
4. `[build.<profile>]`
5. `[build]`

If custom `cflags` are set (non-empty), default compiler flags are disabled.

```toml
[build.debug]
cflags = ["-g3"]

[build.release]
cflags = ["-O3"]
ldflags = ["-s"]

# Target-specific overrides (applied after profile)
[build.linux]
compiler = "gcc"
cflags = ["-march=x86_64"]

[build.macos]
compiler = "clang"
cflags = ["-arch", "x86_64"]

# Full inheritance example
[build.linux.release]
cflags = ["-O3", "-march=native"]  # Overrides profile and base

[build.release.linux]
ldflags = ["-static"]  # Alternative order

[build.windows.debug]
compiler = "x86_64-w64-mingw32-gcc"
cflags = []  # Empty to disable defaults

[build.release]
inherit = false  # No inheritance, only explicit settings
```

## Toolchain section

```toml
[toolchain]
cc = "clang"
cxx = "clang++"
as = "as"
ar = "ar"
ld = "ld"
uic = "uic"
moc = "moc"
rcc = "rcc"
```

## Behavior notes

- Compiler backend is selected by compiler string (`gcc`, `clang`, `cl`, `clang-cl`).
- Empty or unknown compiler value falls back to clang-like build path.
- `run` is not allowed for library kinds (`staticlib`, `sharedlib`), `efi`, or `elf`.
