# CLI reference

## Command list

```text
dcr new <name>
dcr init
dcr build [--debug|--release]
dcr run [--debug|--release]
dcr clean [--debug|--release] [--all]
dcr gen <subcommand>
dcr --help
dcr --version
dcr --update
```

## Notes on argument parsing

- `build`/`run` parse flags in any order (`--debug|--release`, `--force`, `--clean`) and reject duplicates.
- `new` requires exactly one argument.
- `init` and `--update` do not accept extra arguments.
- `clean` accepts `--debug|--release` and optional `--all`.
- `gen` supports `project-info`, `compile-commands`, `vscode`, `clion`.

## Exit behavior overview

- Successful command execution returns `0`.
- Validation/build/runtime failures return non-zero status.
