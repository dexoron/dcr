# Dependencies section

DCR supports registry-based dependencies and path dependencies.

## Example

```toml
[dependencies]
# Registry dependency
mylib = "0.1.0"
# Git dependency
json = { git = "https://github.com/nlohmann/json", tag = "v3.11.2" }
# Path dependency
frecli = { path = "./lib/frecli", include = ["."], lib = ["."], libs = ["frecli"] }
```

## Supported dependency types

### Registry dependencies
Defined by name and version string. DCR automatically resolves the package from the configured registry index.
- `version` (string, required): version requirement.
- `features` (string array, optional): enable specific features.

### Git dependencies
Defined by repository URL and optionally branch, tag, or revision.
- `git` (string, required): repository URL.
- `branch`, `tag`, `rev` (string, optional): Git reference.

### Path dependencies
- `path` (string, required): absolute path or path relative to project root.
- `include` (string array, optional): include directories inside dependency.
- `lib` (string array, optional): library directories inside dependency.
- `libs` (string array, optional): library names for linker.

## Defaults for path dependencies

- If `include` is omitted, DCR tries `<dep>/include`.
- If `lib` is omitted, DCR tries `<dep>/lib`, then `<dep>/lib64`.
- If `libs` is omitted, dependency key name is used.

## Validation

- Dependency value must be a string (registry) or a TOML table (path).
- Registry dependencies use the version string to match versions in the registry.
- `path` is mandatory for path dependencies.
- `include`/`lib`/`libs` must be arrays of strings if provided.
