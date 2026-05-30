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
- `build.compiler`

## Optional keys

- `build.standard` (default: `c11`)
- `build.kind` (default: `bin`; valid: `bin`, `staticlib`, `sharedlib`, `efi`, `elf`, `none`, `custom`)
- `build.inherit`
- `build.targets`
- `build.target`
- `build.out_dir`
- `build.platform`
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
- `build.<target>` (target-specific overrides)
- dependency fields (`include`, `lib`, `libs`, `system`)
- `[toolchain]` overrides (`cc`, `cxx`, `as`, `ar`, `ld`, `uic`, `moc`, `rcc`)
- `[toolchain.<target>]` (target-specific toolchain)
- `[workspace]` members
- `[workspace.<target>]` (target-specific workspace)
- `[run]` overrides (`cmd`)
- `[run.<target>]` (target-specific run)
