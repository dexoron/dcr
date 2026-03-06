# CLI reference

## Command list

```text
dcr new <name>
dcr init
dcr build [--debug|--release]
dcr run [--debug|--release]
dcr clean [--debug|--release] [--all]
dcr --help
dcr --version
dcr --update
```

## Notes on argument parsing

- Most commands parse profile from the first argument.
- `new` requires exactly one argument.
- `init` and `--update` do not accept extra arguments.
- `clean` accepts `--debug|--release` and optional `--all`.

## Exit behavior overview

- Successful command execution returns `0`.
- Validation/build/runtime errors return non-zero status.
