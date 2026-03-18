# dcr gen

Generates IDE and build tool integration files for C/C++ development environments.

## Usage

```sh
dcr gen project-info
dcr gen project-info --release
dcr gen compile-commands
dcr gen compile-commands --debug
dcr gen vscode
dcr gen vscode --release
dcr gen clion
dcr gen clion --debug
```

## Subcommands

### `dcr gen project-info`

Outputs project metadata as JSON array.

```sh
dcr gen project-info [--debug | --release]
```

Outputs structured information about the project (root project + all workspace members if present):

```json
[
  {
    "name": "my-project",
    "version": "1.0.0",
    "root": "/path/to/root",
    "profile": "debug",
    "language": "c++",
    "standard": "c++17",
    "compiler": "g++",
    "kind": "bin",
    "sources": ["/path/to/root/src/main.cpp", ...],
    "include_dirs": ["/path/to/dep/include", ...],
    "lib_dirs": ["/path/to/dep/lib", ...],
    "libs": ["somelib", ...],
    "cflags": ["-Wall", ...],
    "ldflags": ["-lm", ...]
  }
]
```

This is useful for integrating DCR projects with custom build tools or scripts.

### `dcr gen compile-commands`

Generates `compile_commands.json` for use by `clangd`, `clang-tidy`, and other tools.

```sh
dcr gen compile-commands [--debug | --release]
```

Writes `./compile_commands.json` containing compilation database entries for all source files. This file is recognized by:
- LSP servers (`clangd`, `clang-tools-extra`)
- Linters (`clang-tidy`)
- Format tools (`clang-format`)

The entries include:
- Full compiler command with flags
- Source file path
- Object file output path
- Project root as working directory

### `dcr gen vscode`

Generates Visual Studio Code integration files.

```sh
dcr gen vscode [--debug | --release]
```

Creates `.vscode/` directory with:

- **`launch.json`**: Debug configurations (one per binary target, debug + release variants)
- **`tasks.json`**: Build/run/clean tasks using `dcr` commands
- **`settings.json`**: Clangd configuration pointing to `compile_commands.json`
- **`extensions.json`**: Recommends `clangd`, disables `cpptools`
- **`compile_commands.json`**: Compilation database (same as `dcr gen compile-commands`)

Usage in VSCode:
- Open the Command Palette (`Ctrl+Shift+P`) and select **"Debug: Start Debugging"** to run debugger
- Press `Ctrl+Shift+B` to run the build task
- Clangd will provide IntelliSense and diagnostics based on `compile_commands.json`

### `dcr gen clion`

Generates JetBrains CLion integration files.

```sh
dcr gen clion [--debug | --release]
```

Creates `.idea/` directory with:

- **`externalTools.xml`**: Registers DCR build/clean/gen commands as external tools
- **`customTargets.xml`**: Defines build targets for debug and release
- **`misc.xml`**: Points CLion to `compile_commands.json` for compilation database
- **`.idea/.gitignore`**: Ignores auto-generated files
- **`runConfigurations/*.xml`**: Debug run configs (one per binary target)
- **`compile_commands.json`**: Compilation database (same as `dcr gen compile-commands`)

Usage in CLion:
- Build/clean via **Tools → DCR → Build Debug/Release/Clean**
- Run/debug via **Run → Run Configurations** (dropdown) → select generated config
- Code completion and diagnostics come from the compilation database

## Config values used

- `package.name`
- `build.compiler`
- `build.language`
- `build.standard`
- `build.kind`
- `build.target`
- `build.platform`
- `build.cflags`
- `build.ldflags`
- `build.debug` / `build.release`
- `build.exclude`
- `build.include`
- `build.pkg_config`
- `build.roots`
- `build.src_disable`
- Workspace members (if `[workspace]` is present)

## Workspace support

When running `dcr gen` in a workspace root, all commands process both the root project and all workspace members:

- `project-info` returns an array with entries for each member + root
- `compile-commands` includes entries from all members
- `vscode` and `clion` generate unified configurations that work across all members

## Troubleshooting

### Clangd not picking up includes

- Ensure `compile_commands.json` exists in the workspace root
- In VSCode settings, verify `clangd.arguments` includes `--compile-commands-dir=<workspace-root>`
- Rebuild via `dcr gen compile-commands` if sources or dependencies changed

### Debug configuration not working

- Verify binary paths in `launch.json` match actual build output (`target/<profile>/binary-name`)
- For workspace projects, each member's debug config uses its own `target/` directory
- Ensure GDB/LLDB is installed: `gdb --version` or `lldb --version`

### CLion not recognizing includes

- Regenerate via `dcr gen clion --debug`
- File → Invalidate Caches → Invalidate and Restart
- Verify `.idea/misc.xml` points to correct `compile_commands.json` path

## Notes

- Profiles default to `debug` if not specified
- Generation does not require the project to be built first
- Generated files are safe to commit; they are generated from `dcr.toml` and thus repeatable
- IDE configurations may be customized after generation, but changes will be overwritten on next `dcr gen` run
- For multi-binary projects, `vscode` and `clion` generate debug configs for all binaries (both debug + release variants)
