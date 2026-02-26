# FAQ

## What commands are available?

`new`, `init`, `build`, `run`, `clean`, `--help`, `--version`, `--update`.

## Is `dcr.toml` mandatory?

Yes, for `build`, `run`, and `clean`.

## What source files are compiled?

DCR scans `src/` recursively:

- for C: `*.c`
- for C++: `*.cpp`, `*.cxx`, `*.cc`

## Can I switch compiler?

Yes. Set `build.compiler` in `dcr.toml` (for example `gcc`, `clang`, `cl`, `clang-cl`).

## How does incremental rebuild work?

It compares source file mtime with object file mtime. Header dependency tracking is not implemented yet.

## What does `dcr run` do if build fails?

It tries to run an existing binary from the selected profile; if none exists, it reports an error.

## How do I update DCR?

Use `dcr --update` for non-package-managed installs.

If installed through pacman/AUR, update via package manager.
