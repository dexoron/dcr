# Config reference

## Full schema

```toml
[package]
name        = "string"            # required
version     = "string"            # required
type        = "string"            # optional: "app", "lib", "none"
description = "string"            # optional
author      = "string"            # optional
authors     = ["string", "..."]   # optional
license     = "string"            # optional
homepage    = "string"            # optional
repository  = "string"            # optional
readme      = "string"            # optional
keywords    = ["string", "..."]   # optional
categories  = ["string", "..."]   # optional

[build]
language     = "c|c++|cpp|cxx|asm"               # required (string or array)
standard     = "string"                          # optional (default "c11")
cxx_standard = "string"                          # optional (e.g. "c++17", "gnu++17")
compiler     = "string"                          # required (e.g. "clang", "gcc")
kind         = "bin|staticlib|sharedlib|efi|elf|none|custom" # optional (default "bin")
target       = "string"                          # optional (cross-compilation target triple)
out_dir      = "string"                          # optional (custom output directory)
platform     = "string"                          # optional (architecture hint, -march)

cflags       = ["string", "..."]                 # optional (supports {version} etc.)
ldflags      = ["string", "..."]                 # optional (supports {version} etc.)
ldscript     = "string"                          # optional (linker script path)
exclude      = ["string", "..."]                 # optional (glob patterns)
include      = ["string", "..."]                 # optional (allowlist + -I dirs)
roots        = ["string", "..."]                 # optional (source roots)
src_disable  = false                             # optional (disable default src/)
pkg_config   = ["string", "..."]                 # optional (pkg-config packages)
generated    = ["string", "..."]                 # optional (generated file patterns)
expect       = ["string", "..."]                 # optional (expected artifact patterns)
clean        = ["string", "..."]                 # optional (extra clean paths)
targets      = ["string", "..."]                 # optional (build targets)
filename     = "string"                          # optional (custom output name)
extension    = "string"                          # optional (custom extension)
inherit      = true                              # optional (inherit base settings)

[[build.steps]]
name = "string"    # required
in   = "glob"      # required
out  = "path"      # required
cmd  = "string"    # required

# Inline steps (equivalent to [[build.steps]])
steps = [
  { name = "string", in = "glob", out = "path", cmd = "string" },
]

[[build.post_steps]]
name = "string"    # required
in   = "glob"      # required (or "target/...")
out  = "path"      # required
cmd  = "string"    # required

# Inline post_steps (equivalent to [[build.post_steps]])
post_steps = [
  { name = "string", in = "glob", out = "path", cmd = "string" },
]

# Profile overrides (inherit all from [build])
[build.debug]
cflags  = ["string", "..."]  # optional
ldflags = ["string", "..."]  # optional

[build.release]
cflags  = ["string", "..."]  # optional
ldflags = ["string", "..."]  # optional

# Target-specific overrides
[build.linux]
compiler = "string"
cflags   = ["string", "..."]

[build.windows]
compiler = "string"

# Target + profile override
[build.linux.release]
cflags = ["string", "..."]

[toolchain]
cc  = "string"  # optional (C compiler override)
cxx = "string"  # optional (C++ compiler override)
as  = "string"  # optional (assembler override)
ar  = "string"  # optional (archiver override)
ld  = "string"  # optional (linker override)
uic = "string"  # optional (Qt uic)
moc = "string"  # optional (Qt moc)
rcc = "string"  # optional (Qt rcc)

[run]
cmd = "string"  # required for `dcr run`

# Debug run variant
[run.debug]
cmd = "string"

[workspace]
[workspace.member_name]
path = "string"             # required
deps = ["string", "..."]    # optional

[dependencies]
[dependencies.dep_name]
path    = "string"          # local path
# or
version = "string"          # registry version
# or
git     = "string"          # git URL
branch  = "string"          # optional (git branch)
tag     = "string"          # optional (git tag)
rev     = "string"          # optional (git commit)
# common fields:
include = ["string", "..."] # optional (include dirs)
lib     = ["string", "..."] # optional (lib dirs)
libs    = ["string", "..."] # optional (link libs)
features        = ["string", "..."] # optional (registry features)
default-features = true             # optional (include default features)
```

## Validation rules

- `[package]`, `[build]`, `[dependencies]` must exist.
- Required string fields must be non-empty.
- `build.kind` must be `bin`, `staticlib`, `sharedlib`, `efi`, `elf`, `none`, or `custom`.
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
