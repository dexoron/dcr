# dcr run

Builds the project and runs the resulting binary.

## Usage

```sh
dcr run
dcr run --debug
dcr run --release
dcr run --release --force
dcr run --debug --clean
```

## Behavior

1. Validates `dcr.toml`.
2. Reads `package.name`, `build.kind`, and optional `build.target`.
3. Runs the same build flow as `dcr build`.
4. Executes the built artifact path for the selected profile.
5. If `[run].cmd` is set, runs that command instead of the built binary.

## Restrictions

- If `build.kind = "staticlib"` or `build.kind = "sharedlib"`, `run` exits with error.
- The profile is selected from the first argument.
- `--force` skips build cache checks and recompiles.
- `--clean` removes `target/<profile>` and `build.clean` paths before building.

## `run.cmd` variables

`run.cmd` supports the same variables as build steps: `{profile}`, `{version}`, `{version_major}`,
`{version_minor}`, `{version_patch}`, `{version_suffix}`, `{version_suffix_dash}`.
The command runs via `sh -c` on Unix and `cmd /C` on Windows.

## On build failure

`dcr run` attempts to launch an existing binary from the same profile if present; otherwise it reports that code errors must be fixed.
