# dcr.toml overview

`dcr.toml` is the main project file used by DCR.

## Minimal generated template

```toml
[package]
name = "hello"
version = "0.1.0"

[build]
language = "c"
standard = "c11"
compiler = "clang"
kind = "bin"

[dependencies]
```

`dcr new` and `dcr init` generate this structure automatically.

## Required sections

- `[package]`
- `[build]`
- `[dependencies]`

## Required keys

- `package.name`
- `package.version`
- `build.language`
- `build.standard`
- `build.compiler`
- `build.kind`

`build.kind` must be either `bin`, `staticlib`, or `sharedlib`.

## Optional keys

- `build.target`
- `build.cflags`
- `build.ldflags`
- dependency fields (`include`, `lib`, `libs`, `system`)
