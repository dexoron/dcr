# Create your first project

## Create in a new folder

```sh
dcr new hello
cd hello
```

This creates:

```text
hello/
- dcr.toml
- src/
- - main.c
```

## Or initialize current folder

```sh
mkdir hello
cd hello
dcr init
```

`dcr init` requires the current directory to be empty.

## Next steps

Build and run:

```sh
dcr run
```

Release build/run:

```sh
dcr run --release
```
