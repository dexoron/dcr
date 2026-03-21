# Flags & environment

This page summarizes runtime flags and environment variables that affect DCR behavior.

## Build/run flags

Used with `dcr build` and `dcr run`:

- `--debug`: select debug profile.
- `--release`: select release profile.
- `--force`: ignore incremental cache checks and rebuild.
- `--clean`: remove `target/<profile>` and `build.clean` paths before build.

Notes:

- Flags can be passed in any order.
- Duplicate profile flags are rejected.

## Clean flags

Used with `dcr clean`:

- `--debug` / `--release`: clean only selected profile directory.
- `--all`: in workspace root, also clean all members.

## Environment variables

Tool selection overrides:

- `DCR_COMPILER` (all languages)
- `DCR_CC`, `DCR_CXX`, `DCR_AS`, `DCR_AR`, `DCR_LD`

Diagnostics:

- `DCR_DEBUG=1` enables verbose build logging.

## Quick examples

```sh
# Force full release rebuild
dcr build --release --force

# Clean before debug run
dcr run --clean --debug

# Use custom compiler from env
DCR_CC=clang-18 dcr build --debug
```
