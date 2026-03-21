# dcr build

Builds the project in `debug` or `release` profile.

## Usage

```sh
dcr build
dcr build --debug
dcr build --release
dcr build --release --force
dcr build --debug --clean
```

## What `build` does

1. Checks that `dcr.toml` exists and is valid.
2. Reads build settings from `[build]`.
3. Resolves path dependencies from `[dependencies]`.
4. Creates required output directories.
5. Recursively compiles sources from `src/`.
6. Links final artifact (binary or library).

## Config values used

- `package.name`
- `build.compiler`
- `build.language`
- `build.standard`
- `build.kind`
- `build.target`
- `build.platform`
- `build.cflags`
- `build.ldflags`
- `build.debug` / `build.release`
- `build.exclude`
- `build.include`
- `build.pkg_config`
- `build.steps`
- `build.post_steps`
- `build.generated`
- `build.expect`
- `build.clean`

## Source selection

- `language = "c"` -> `*.c`
- `language = "c++" | "cpp" | "cxx"` -> `*.cpp`, `*.cxx`, `*.cc`
- `language = "asm"` -> `*.s`, `*.S`, `*.asm`
- Mixed languages are supported with arrays, for example `language = ["c", "asm"]`.
- By default sources are searched in `src/`; use `build.roots` and `build.src_disable` to override.

## Notes

- Profile flag (`--debug` / `--release`) can appear in any argument position (duplicates are rejected).
- Unknown profile flags return an error.
- Incremental rebuild checks source/object mtime and tracked header dependencies.
- `--force` skips build cache checks and recompiles.
- `--clean` removes `target/<profile>` and `build.clean` paths before building.
- For `language = "asm"` with `compiler = "as"`/`"gas"`, use `.s` files (no preprocessing). For `.S`, use `gcc` or `clang`.
- In workspace root, `dcr build` builds all members in dependency order.
- `build.exclude` removes paths from source/header collection; `build.include` re-allows matching paths and has priority over `exclude`.
- `build.steps` run before compilation; `build.post_steps` run after linking.
- `build.generated` is cleaned when `build.steps` need to rerun; `build.expect` is verified after post-steps.
