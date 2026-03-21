# Workspace guide

`[workspace]` lets you build related DCR projects in dependency order from one root.

## Important rule

Workspace root still needs a valid root project config:

- `[package]`
- `[build]`
- `[dependencies]`

`[workspace]` augments that root config; it does not replace required root sections.

## Minimal example

```toml
[package]
name = "root-app"
version = "0.1.0"

[build]
language = "c"
standard = "c11"
compiler = "clang"
kind = "bin"

[dependencies]

[workspace]
core = { path = "./modules/core", deps = [] }
net = { path = "./modules/net", deps = ["core"] }
app = { path = "./modules/app", deps = ["core", "net"] }
```

Each member path must point to a directory containing its own `dcr.toml`.

## Build behavior

- Running `dcr build` in workspace root builds members in topological dependency order, then builds the root project.

- Running `dcr clean --all` in workspace root cleans all members too.

- `dcr gen` in workspace root includes root + members in generated outputs.

## Common errors

- `workspace.<name>.path does not contain dcr.toml`:
  member path is wrong or config is missing.
- `workspace dependency '<name>' not found`:
  `deps` references unknown member.
- `workspace dependency cycle at <name>`:
  cyclic dependencies in member graph.
