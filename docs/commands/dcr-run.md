# dcr run

Builds the project and runs the resulting binary.

## Usage

```sh
dcr run
dcr run --debug
dcr run --release
```

## Behavior

1. Validates `dcr.toml`.
2. Reads `package.name`, `build.kind`, and optional `build.target`.
3. Runs the same build flow as `dcr build`.
4. Executes the built artifact path for the selected profile.

## Restrictions

- If `build.kind = "staticlib"` or `build.kind = "sharedlib"`, `run` exits with error.
- The profile is selected from the first argument.

## On build failure

`dcr run` attempts to launch an existing binary from the same profile if present; otherwise it reports that code errors must be fixed.
