# dcr clean

Removes build artifacts in `target/`.

## Usage

```sh
dcr clean
dcr clean --debug
dcr clean --release
```

## Behavior

- Without arguments: removes entire `target/`.
- With profile: removes only `target/<profile>/`.

## Validation

- `dcr.toml` must exist.
- More than one argument is treated as error.
- Unknown profile is treated as error.
- If `target` or selected profile directory does not exist, command prints a warning.
