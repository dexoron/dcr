# dcr.toml

`dcr.toml` describes your project, build options, and dependencies.

## Example

```toml
[package]
name = "neoarc-frontent"
version = "0.1.0"

[build]
language = "c"
standard = "c11"
compiler = "gcc"
# Custom compile/link flags (optional)
cflags = ["-D_POSIX_C_SOURCE=200809L", "-Wall", "-Wextra", "-O2", "-Ilib/frecli"]
ldflags = ["-Llib/frecli", "-lfrecli", "-lm"]

[dependencies]
frecli = { path = "./lib/frecli", include = ["."], lib = ["."], libs = ["frecli"] }
```

## Sections

### [package]
- `name`     (string, required)
- `version`  (string, required)

### [build]
- `language` (string, required): `c`, `c++`, `cpp`, or `cxx`
- `standard` (string, required): e.g. `c99`, `c11`, `c++17`
- `compiler` (string, required): `gcc`, `clang`, `cl`(msvc), or `clang-cl`
- `cflags`   (string[], optional): additional compile flags
- `ldflags`  (string[], optional): additional link flags

### [dependencies]
Only `path` dependencies are supported right now.

```toml
[dependencies]
fmt = { path = "./lib/fmt", include = ["include"], lib = ["lib"], libs = ["fmt"] }
```

Rules:
- `path` can be absolute or relative to the project root.
- If `include`/`lib` are missing, DCR looks for `include`, `lib`, or `lib64` under the dependency path.
- If not found, build fails with a message suggesting to add `include`/`lib`.
- `libs` defaults to the dependency name.

## Source Discovery

DCR compiles all `.c`/`.cpp` files under `src/` recursively. The file containing `main()` can be located anywhere in that tree.
