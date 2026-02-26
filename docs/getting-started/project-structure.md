# Project structure

## Minimal project

```text
project/
- dcr.toml
- src/
- - main.c
```

## Typical generated/optional files

```text
project/
- dcr.toml
- dcr.lock
- src/
- - ... source files ...
- target/
- - debug/
- - - obj/
- - - deps/
- - release/
- - - obj/
- - - deps/
```

## Meaning

- `dcr.toml`: main project configuration.
- `dcr.lock`: generated lock file for resolved path dependencies.
- `src/`: recursively scanned source directory.
- `target/<profile>/obj/`: cached object files for incremental rebuild.
- `target/<profile>/deps/`: synchronized copies of path dependencies.
- `target/<profile>/<name>` (or `.exe` on Windows): output for `kind = "bin"`.
- `target/<profile>/lib<name>.a` (or `<name>.lib` on Windows): output for `kind = "staticlib"`.

If `build.target` is set, the final artifact is written to that custom directory, while object cache still stays in `target/<profile>/obj`.
