# Build and run

## Build

Debug (default):

```sh
dcr build
# or
dcr build --debug
```

Release:

```sh
dcr build --release
```

## Run

```sh
dcr run
# or
dcr run --release
```

`dcr run` first builds, then launches the produced binary.

## Clean artifacts

```sh
dcr clean
```

Profile-only cleanup:

```sh
dcr clean --debug
dcr clean --release
```

## Notes

- `run` is unavailable for `build.kind = "staticlib"`.
- Build profile flag is parsed from the first command argument.
- Unknown profile flags produce an error.
