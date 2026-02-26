# Target directory

By default, DCR writes final artifacts to `target/<profile>/`.

You can override this with `build.target`:

```toml
[build]
target = "./dist"
```

## Behavior

- Binary mode:
  - Linux/macOS: `<target>/<name>`
  - Windows: `<target>/<name>.exe`
- Static library mode:
  - Linux/macOS: `<target>/lib<name>.a`
  - Windows: `<target>/<name>.lib`

## Important detail

`build.target` changes only the final artifact location. Object cache and dependency sync directories still use `target/<profile>/...`.
