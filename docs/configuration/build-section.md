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
- `standard` (string, required): language standard passed to compiler.
- `compiler` (string, required): compiler command (for example `clang`, `gcc`, `cl`).
- `kind` (string, required): `bin`, `staticlib`, or `sharedlib`.
- `target` (string, optional): custom output directory for final artifact.
- `platform` (string, optional): architecture hint for compiler (used for `-march` or `/arch`).
- `cflags` (string array, optional): extra compile flags.
- `ldflags` (string array, optional): extra link flags.
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

### Build steps

Build steps run shell commands before compilation. Post steps run after linking.

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

```toml
[build.debug]
cflags = ["-g3"]

[build.release]
cflags = ["-O3"]
ldflags = ["-s"]
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
- `run` is not allowed for library kinds (`staticlib`, `sharedlib`).
