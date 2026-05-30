# Cross-compilation and Targets

DCR supports cross-compilation to different platforms using the `--target` flag. This allows building your project for architectures and operating systems different from the host.

## Basic Usage

```bash
# Build for a specific target
dcr build --target x86_64-unknown-linux-gnu --release

# Short names for common platforms
dcr build --target linux --release    # x86_64-unknown-linux-gnu
dcr build --target macos --release    # x86_64-apple-darwin
dcr build --target windows --release  # x86_64-pc-windows-msvc

# Run for a specific target
dcr run --target linux --release
```

## Supported Targets

### Short Names
- `linux` → `x86_64-unknown-linux-gnu`
- `macos` → `x86_64-apple-darwin`
- `windows` → `x86_64-pc-windows-msvc`
- `freebsd` → `x86_64-unknown-freebsd`
- `openbsd` → `x86_64-unknown-openbsd`
- `netbsd` → `x86_64-unknown-netbsd`

### Full Target Triples
DCR supports any valid target triple in the format `<arch>-<vendor>-<os>-<abi>`. Common examples:
- `x86_64-unknown-linux-gnu`
- `aarch64-linux-gnu`
- `x86_64-apple-darwin`
- `aarch64-apple-darwin`
- `x86_64-pc-windows-msvc`
- `i686-pc-windows-gnu`
- `x86_64-unknown-freebsd`
- `x86_64-unknown-openbsd`
- `x86_64-unknown-netbsd`

## Configuration

### Multiple Targets

Configure multiple targets to build simultaneously:

```toml
[build]
targets = ["linux", "macos", "windows"]

# Or per profile
[build.release]
targets = ["linux", "macos"]
```

If no `--target` is specified, DCR builds for all targets in `build.targets` (or native if empty).

### Target-Specific Settings

Override settings for specific targets:

```toml
[build.linux]
compiler = "gcc"
cflags = ["-Wall", "-O2"]

[build.macos]
compiler = "clang"
cflags = ["-Wall", "-O2", "-arch", "x86_64"]

[build.windows]
compiler = "x86_64-w64-mingw32-gcc"
ldflags = ["-static"]

[build.freebsd]
compiler = "clang"
cflags = ["-Wall", "-O2"]
```

### Target-Specific Toolchain

Configure toolchain per target:

```toml
[toolchain.linux]
cc = "gcc"
ld = "ld"

[toolchain.macos]
cc = "clang"
ld = "ld"

[toolchain.windows]
cc = "x86_64-w64-mingw32-gcc"
ld = "x86_64-w64-mingw32-ld"
```

### Target-Specific Run Commands

Use custom run commands per target:

```toml
[run.linux]
cmd = "./target/linux/release/myapp"

[run.macos]
cmd = "open ./target/macos/release/myapp.app"

[run.windows]
cmd = "wine ./target/windows/release/myapp.exe"

[run.freebsd]
cmd = "./target/freebsd/release/myapp"
```

## Workspace and Dependencies Inheritance

Workspace and dependencies sections also support target/profile inheritance:

```toml
[workspace.linux]
members = [
  { name = "lib1", path = "libs/lib1" },
]

[dependencies.linux]
sdl2 = { system = true, include = ["/usr/include/SDL2"] }

[dependencies.windows]
sdl2 = { system = true, include = ["C:/SDL2/include"] }
```

Order: `workspace.target.profile` → `workspace.profile.target` → `workspace.target` → `workspace.profile` → `workspace`.

Add `inherit = false` to disable inheritance in any section.

Settings are applied in order:
1. `[build.<target>]`
2. `[build.<profile>]`
3. `[build]`

### Toolchain Configuration

For cross-compilation, configure toolchains:

```toml
[toolchain]
cc = "aarch64-linux-gnu-gcc"
cxx = "aarch64-linux-gnu-g++"
ar = "aarch64-linux-gnu-ar"
ld = "aarch64-linux-gnu-ld"
```

## Output Structure

Build outputs are organized by target:

```
target/
├── native/
│   ├── debug/
│   └── release/
├── x86_64-unknown-linux-gnu/
│   ├── debug/
│   └── release/
├── aarch64-linux-gnu/
│   ├── debug/
│   └── release/
└── x86_64-unknown-freebsd/
    ├── debug/
    └── release/
```

## Requirements

- Cross-compilation requires appropriate toolchains installed (e.g., `aarch64-linux-gnu-gcc` on Arch Linux).
- For Windows targets on Linux, install MinGW: `pacman -S mingw-w64-gcc`.
- For macOS targets on Linux, use `osxcross` or similar.

## Examples

### ARM Linux Build

```bash
# Install toolchain (Arch Linux)
pacman -S aarch64-linux-gnu-gcc

# Configure
echo '[build.aarch64-linux-gnu]
compiler = "aarch64-linux-gnu-gcc"' >> dcr.toml

# Build
dcr build --target aarch64-linux-gnu --release
```

**Note:** If you set `target = "aarch64-linux-gnu"` (or any value) directly under `[build]`, DCR will automatically add `--target=...` to cflags when using clang. You usually don't need to repeat it manually.

### Windows Build on Linux

```bash
# Install toolchain (Arch Linux)
pacman -S mingw-w64-gcc

# Configure
echo '[build.windows]
compiler = "x86_64-w64-mingw32-gcc"' >> dcr.toml

# Build
dcr build --target windows --release
```

## Troubleshooting

- **Toolchain not found**: Ensure the compiler is installed and in PATH, or configure `[toolchain]` in `dcr.toml`. For Arch Linux: `pacman -S aarch64-linux-gnu-gcc` for ARM, `pacman -S mingw-w64-gcc` for Windows cross-compilation.
- **Linking errors**: Check `ldflags` and library paths for cross-compilation.
- **Header issues**: Include paths may differ; use target-specific `cflags`.

For more help, see [Troubleshooting map](../reference-and-help/troubleshooting-map.md).