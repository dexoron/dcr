# dcr clean

Removes build artifacts in `target/`.

## Usage

```sh
dcr clean
dcr clean --debug
dcr clean --release
dcr clean --all
dcr clean --release --all
```

## Behavior

- Without arguments: removes entire `target/`.
- With profile: removes only `target/<profile>/`.
- With `--all`: also cleans all workspace members.

## Validation

- `dcr.toml` must exist.
- Unknown flags are treated as error.
- If `target` or selected profile directory does not exist, command prints a warning.
