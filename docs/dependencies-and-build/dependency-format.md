# Dependency format

Dependencies are defined in `dcr.toml` under `[dependencies]`.

## Full example (Path)

```toml
[dependencies]
fmt = { path = "./third_party/fmt", include = ["include"], lib = ["lib"], libs = ["fmt"] }
```

## Full example (Git)

```toml
[dependencies]
json = { git = "https://github.com/nlohmann/json", tag = "v3.11.2" }
```

## Field reference

- `path`: local path to dependency root.
- `git`: URL of Git repository.
- `branch`: Git branch name.
- `tag`: Git tag name.
- `rev`: Git commit SHA.
- `include`: optional include subpaths (absolute or relative to dep root).
- `lib`: optional library subpaths (absolute or relative to dep root).
- `libs`: optional linker library names.
- `features`: optional feature flags for registry dependencies.
- `default-features`: whether to include default features (default `true`).

## Common valid compact forms

```toml
[dependencies]
mylib = { path = "./libs/mylib" }
gitlib = { git = "https://github.com/user/repo" }
# Shortest form for path
simple = "./libs/simple"
# Registry with features
sdl2 = { version = "2.0.0", features = ["vulkan"] }
```

In these forms DCR will try defaults for include/lib dirs and use the dependency name as the linker library name.
