# Dependencies section

DCR currently supports path dependencies.

## Example

```toml
[dependencies]
frecli = { path = "./lib/frecli", include = ["."], lib = ["."], libs = ["frecli"] }
```

## Supported fields per dependency

- `path` (string, required): absolute path or path relative to project root.
- `include` (string array, optional): include directories inside dependency.
- `lib` (string array, optional): library directories inside dependency.
- `libs` (string array, optional): library names for linker.
- `system` (bool, optional): currently `system = true` is rejected.

## Defaults

- If `include` is omitted, DCR tries `<dep>/include`.
- If `lib` is omitted, DCR tries `<dep>/lib`, then `<dep>/lib64`.
- If `libs` is omitted, dependency key name is used.

## Validation

- Dependency value must be a TOML table.
- `path` is mandatory.
- `include`/`lib`/`libs` must be arrays of strings if provided.
