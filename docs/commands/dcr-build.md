# dcr build

Builds the project in `debug` or `release` profile.

## Usage

```sh
dcr build
dcr build --debug
dcr build --release
```

## What `build` does

1. Checks that `dcr.toml` exists and is valid.
2. Reads build settings from `[build]`.
3. Resolves path dependencies from `[dependencies]`.
4. Creates required output directories.
5. Recursively compiles sources from `src/`.
6. Links final artifact (binary or static library).

## Config values used

- `package.name`
- `build.compiler`
- `build.language`
- `build.standard`
- `build.kind`
- `build.target`
- `build.cflags`
- `build.ldflags`

## Source selection

- `language = "c"` -> `*.c`
- `language = "c++" | "cpp" | "cxx"` -> `*.cpp`, `*.cxx`, `*.cc`

## Notes

- The profile is selected from the first argument only.
- Unknown profile flags return an error.
- Incremental rebuild is based on source/object mtime comparison.
