# Config reference

## Full schema

```toml
[package]
name = "string"      # required
version = "string"   # required

[build]
language = "c|c++|cpp|cxx|asm"     # required (string or array)
standard = "string"            # required
compiler = "string"            # required
kind = "bin|staticlib|sharedlib"         # required
target = "string"              # optional
platform = "string"            # optional
cflags = ["string", "..."]     # optional
ldflags = ["string", "..."]    # optional
exclude = ["string", "..."]    # optional
include = ["string", "..."]    # optional
roots = ["string", "..."]      # optional
src_disable = false            # optional
pkg_config = ["string", "..."] # optional
generated = ["string", "..."]  # optional
expect = ["string", "..."]     # optional
clean = ["string", "..."]      # optional

[[build.steps]]
name = "string"
in = "glob"
out = "path or glob"
cmd = "string"

[[build.post_steps]]
name = "string"
in = "glob"
out = "path or glob"
cmd = "string"

# Inline array form (equivalent to [[build.steps]] / [[build.post_steps]])
steps = [
  { name = "string", in = "glob", out = "path or glob", cmd = "string" },
]
post_steps = [
    { name = "string", in = "glob", out = "path or glob", cmd = "string" },
]

[build.debug]
cflags = ["string", "..."]     # optional
ldflags = ["string", "..."]    # optional

[build.release]
cflags = ["string", "..."]     # optional
ldflags = ["string", "..."]    # optional

[toolchain]
cc = "string"   # optional
cxx = "string"  # optional
as = "string"   # optional
ar = "string"   # optional
ld = "string"   # optional
uic = "string"  # optional
moc = "string"  # optional
rcc = "string"  # optional

[run]
cmd = "string"  # optional

[workspace]
name = { path = "string", deps = ["string", "..."] }

[dependencies]
name = {
  path = "string",              # required for each dependency
  include = ["string", "..."],  # optional
  lib = ["string", "..."],      # optional
  libs = ["string", "..."],     # optional
}
```

## Validation rules

- `[package]`, `[build]`, `[dependencies]` must exist.
- Required string fields must be non-empty.
- `build.kind` must be `bin`, `staticlib`, or `sharedlib`.
- Dependency fields `include/lib/libs` must be string arrays when provided.
- `build.exclude`/`build.include` must be string arrays when provided.
- `build.roots` must be a string array when provided.
- `build.src_disable` must be boolean when provided.
- `build.clean`/`build.generated`/`build.pkg_config`/`build.expect` must be string arrays when provided.
- `build.debug`/`build.release` must be tables; their `cflags`/`ldflags` must be string arrays.
- `build.steps`/`build.post_steps` entries must include `name`, `in`, `out`, `cmd`.
- `build.language` may be a string or array (for example `["c", "c++", "asm"]`).
- `[build.<profile>]` tables inherit all base values and override any fields they specify.
  Array fields (like `cflags`/`ldflags`) are appended to the base array.

## Generated defaults (`dcr new` / `dcr init`)

- `package.version = "0.1.0"`
- `build.language = "c"`
- `build.standard = "c11"`
- `build.compiler = "clang"`
- `build.kind = "bin"`
