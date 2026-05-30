# Target directory

By default, DCR writes final artifacts to `target/<target>/<profile>/` (where `<target>` is `native` or target triple).

You can override this with `build.out_dir`:

```toml
[build]
out_dir = "./dist"
```

## Behavior

- Binary mode:
  - Linux/macOS: `<out_dir>/<name>`
  - Windows: `<out_dir>/<name>.exe`
- Static library mode:
  - Linux/macOS: `<out_dir>/lib<name>.a`
  - Windows: `<out_dir>/<name>.lib`

## Important detail

`build.out_dir` changes only the final artifact location. Object cache and dependency sync directories still use `target/<target>/<profile>/...`.

## Distinction from platform targets

- `build.out_dir` — custom **output directory** path.
- `build.target` / `--target <triple>` CLI flag — **platform target** for cross-compilation (e.g. `x86_64-unknown-linux-gnu`, or short name `linux`).
- `build.targets` (array) — list of **platform targets** to build simultaneously.
- `[build.<target>]` sections — **target-specific config overrides** (e.g. `[build.linux]`).
