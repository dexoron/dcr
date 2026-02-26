# Local dependencies

## Resolution flow during build

For each dependency in `[dependencies]`, DCR:

1. Resolves `path` (absolute or project-relative).
2. Resolves include dirs (`include` or default `include`).
3. Resolves lib dirs (`lib` or defaults `lib`, `lib64`).
4. Resolves link libs (`libs` or dependency name).
5. Synchronizes dependency files to `target/<profile>/deps/<dep-name>/`.
6. Adds include/lib/link options to compile/link commands.

## Failure cases

- Dependency path is missing or not a directory.
- No valid include/lib directories were found.
- Any configured include/lib path does not exist.
- Invalid field types in dependency config.
