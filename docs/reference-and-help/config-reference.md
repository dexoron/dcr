# Config reference

## Full schema

```toml
[package]
name = "string"      # required
version = "string"   # required

[build]
language = "c|c++|cpp|cxx|asm"     # required
standard = "string"            # required
compiler = "string"            # required
kind = "bin|staticlib|sharedlib"         # required
target = "string"              # optional
platform = "string"            # optional
cflags = ["string", "..."]     # optional
ldflags = ["string", "..."]    # optional

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

## Generated defaults (`dcr new` / `dcr init`)

- `package.version = "0.1.0"`
- `build.language = "c"`
- `build.standard = "c11"`
- `build.compiler = "clang"`
- `build.kind = "bin"`
