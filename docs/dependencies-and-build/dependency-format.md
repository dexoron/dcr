# Dependency format

Dependencies are defined in `dcr.toml` under `[dependencies]`.

## Full example

```toml
[dependencies]
fmt = { path = "./third_party/fmt", include = ["include"], lib = ["lib"], libs = ["fmt"] }
```

## Field reference

- `path`: required dependency root path.
- `include`: optional include subpaths (absolute or relative to dep root).
- `lib`: optional library subpaths (absolute or relative to dep root).
- `libs`: optional linker library names.

## Common valid compact form

```toml
[dependencies]
mylib = { path = "./libs/mylib" }
```

In this form DCR will try defaults for include/lib dirs and use `mylib` as the linker library name.
