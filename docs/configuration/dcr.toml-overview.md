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

Workspace-only root:
- `[workspace]` can be added to a normal root config to define members and dependency order.
- Root config still requires `[package]`, `[build]`, and `[dependencies]`.

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
- `build.exclude`
- `build.include`
- `build.roots`
- `build.src_disable`
- `build.pkg_config`
- `build.steps`
- `build.post_steps`
- `build.generated`
- `build.expect`
- `build.clean`
- `build.debug`
- `build.release`
- dependency fields (`include`, `lib`, `libs`, `system`)
- `[toolchain]` overrides (`cc`, `cxx`, `as`, `ar`, `ld`, `uic`, `moc`, `rcc`)
- `[workspace]` members
- `[run]` overrides (`cmd`)
