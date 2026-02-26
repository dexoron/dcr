# dcr init/new

## `dcr new <name>`

Creates a new project directory and initializes it.

```sh
dcr new hello
```

Behavior:

1. Validates arguments (`<name>` is required, only one name is allowed).
2. Creates `<name>/`.
3. Creates `<name>/dcr.toml`.
4. Sets `package.name = "<name>"`.
5. Creates `<name>/src/main.c` with a Hello World template.

Errors:

- Missing project name.
- Extra arguments.
- Target directory already exists.

## `dcr init`

Initializes DCR in the current directory.

```sh
dcr init
```

Behavior:

1. Requires current directory to be empty.
2. Creates `dcr.toml`.
3. Sets `package.name` from the current directory name.
4. Creates `src/main.c`.

Errors:

- Any arguments are treated as invalid.
- Non-empty directory is rejected.
