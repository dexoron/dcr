# Global options

## `dcr --help`

```sh
dcr --help
```

Prints top-level CLI usage, commands, flags, and examples.

## `dcr --version`

```sh
dcr --version
```

Prints:

```text
dcr <version> (<target>)
```

## `dcr --update`

```sh
dcr --update
```

Update flow:

1. Requests latest release from GitHub API.
2. Finds asset matching current `DCR_TARGET`.
3. Downloads replacement binary.
4. Replaces current executable via `self-replace`.

Limitations and behavior:

- No extra arguments are accepted.
- Requires network access to GitHub API/assets.
- On Linux, if binary is owned by pacman, DCR prints package-manager update instructions instead of self-updating.

## Running without command

Running plain `dcr` (without arguments) shows help output.
