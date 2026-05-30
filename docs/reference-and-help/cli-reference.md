# CLI reference

## Command list

```text
dcr new <name>
dcr init
dcr setup
dcr add <name> <source>(optional)
dcr build [--debug|--release] [--target <triple>]
dcr run [--debug|--release]
dcr clean [--debug|--release] [--all]
dcr tree
dcr test
dcr gen <subcommand>
dcr fmt
dcr setup
dcr --help
dcr --version
dcr --update
```

## Notes on argument parsing

- `build`/`run` parse flags in any order (`--debug|--release`, `--force`, `--clean`, `--verbose`, `--workspace`) and reject duplicates.
- `new` requires exactly one argument.
- `init` and `--update` do not accept extra arguments.
- `clean` accepts `--debug|--release` and optional `--all`.
- `gen` supports `project-info`, `compile-commands`, `vscode`, `clion`.
- `fmt` formats C/C++ sources (`.c`, `.cpp`, `.cxx`, `.cc`, `.h`, `.hpp`, `.hxx`, `.hh`) in `src/` and `tests/` using `clang-format`.
- `setup` initializes DCR registries and local configuration.
- `tree` shows a dependency tree (similar to `cargo tree`).
- `test` / `tests` runs the project's tests.

## Exit behavior overview

- Successful command execution returns `0`.
- Validation/build/runtime failures return non-zero status.
