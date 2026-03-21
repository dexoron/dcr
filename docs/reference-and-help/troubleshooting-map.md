# Troubleshooting map

Fast mapping from symptom to likely cause and first action.

## `dcr.toml file not found`

Likely cause:
- you are outside project root (or member root).

First action:
```sh
pwd
ls
```
Run DCR from directory that contains `dcr.toml`.

## `Invalid config: missing [package]` / `missing [build]`

Likely cause:
- incomplete config in root project.

First action:
- ensure root `dcr.toml` includes `[package]`, `[build]`, and `[dependencies]`.
- for workspace setups, keep these sections in root and add `[workspace]` additionally.

## `Unknown build flag` or profile-flag errors

Likely cause:
- typo in flag or duplicate profile flag.

First action:
```sh
dcr --help
```
Use only supported flags and at most one of `--debug` / `--release`.

## Dependency include/lib path errors

Likely cause:
- path in `[dependencies]` does not exist.

First action:
- verify `path`, `include`, `lib` values.
- if using profile placeholders, confirm expanded paths exist for selected profile.

## `Cannot run library build`

Likely cause:
- `build.kind` is `staticlib` or `sharedlib`.

First action:
- switch `build.kind` to `bin`, or set `[run].cmd` if you intentionally need custom run command.

## Build succeeds but output is stale

Likely cause:
- stale artifacts or generated files from previous runs.

First action:
```sh
dcr clean --all
dcr build --force
```

## `dcr --update` suggests package manager update

Likely cause:
- DCR binary is managed by pacman/AUR.

First action:
```sh
yay -Syu dcr
# or
paru -Syu dcr
```
