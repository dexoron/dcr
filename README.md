# DCR (Dexoron Cargo Realization)

DCR is a utility for managing C/C++ projects in a Cargo-like style.

The current implementation is written in Rust.

## Why DCR
- Unified project structure without manual setup
- Simple commands for common tasks
- Transparent compilation and predictable build profiles

## Features
- Create a new project or initialize the current directory
- Build a project with `debug` and `release` profiles
- Run the compiled binary
- Clean build artifacts
- Generate a minimal C project template
- Update the binary via `dcr --update` (GitHub Releases, not for pacman/AUR installs)

## Supported Platforms
- Linux: `x86_64-unknown-linux-gnu`
- macOS Intel: `x86_64-apple-darwin`
- macOS Apple Silicon: `aarch64-apple-darwin`
- Windows: `x86_64-pc-windows-msvc`

## Installation

### Package Manager

**Arch Linux**
```sh
yay -S dcr # or paru and other AUR package managers
```

### From Source

```sh
git clone https://github.com/dexoron/dcr.git
cd dcr
cargo build --release
mkdir -p ~/.local/bin
ln -sf "$PWD/target/release/dcr" ~/.local/bin/dcr
```

### Via `install.sh` (Linux/macOS)

```sh
curl -fsSL https://dcr.dexoron.su/install.sh | bash
```

### Via `install.ps1` (Windows)

```powershell
irm https://dcr.dexoron.su/install.ps1 | iex
```

When executed, both scripts ask whether to:
- download a prebuilt binary from GitHub Releases
- or build the project from `git`

Release assets:
- `dcr-x86_64-unknown-linux-gnu`
- `dcr-x86_64-apple-darwin`
- `dcr-aarch64-apple-darwin`
- `dcr-x86_64-pc-windows-msvc.exe`

## Update

- If DCR was installed from GitHub release assets, `install.sh`, `install.ps1`, or built manually:
  - use `dcr --update`
- If DCR is installed via `pacman/AUR`:
  - update with your package manager: `paru/yay -Syu dcr` or `sudo pacman -Syu dcr`
  - `dcr --update` detects package-managed installs and asks you to update via package manager

## Quick Start

Create a new project:
`dcr new hello`

Or initialize the current directory (the directory must be empty):
`dcr init`

Project structure:

```txt
hello/
- src/
- - main.c
- dcr.toml
```

Build and run the project:
`dcr run` or `dcr run --release`

## Commands

### `dcr new <name>`
Creates a project with the specified name in the current directory.

### `dcr init`
Creates a project in the current directory. The project name is taken from the directory name. The directory must be empty.

### `dcr build [profile]`
Builds the project. If no profile is specified, `--debug` is used.

### `dcr run [profile]`
Builds the project and runs the binary. If no profile is specified, `--debug` is used.

Run manually:
`./target/<profile>/<name>`

### `dcr clean`
Removes the `target` directory in the project root.

## Build Profiles
Two profiles are supported:
- `--debug` (default) - built-in flags per compiler
- `--release` - built-in flags per compiler

Custom flags can be added in `dcr.toml` via `build.cflags` and `build.ldflags`.
You can also set `build.target` to override the output directory (profile-independent).

## Configuration
The main project file is `dcr.toml`.

Example `dcr.toml`:

```toml
[package]
name = "hello"
version = "0.1.0"

[build]
language = "c"
standard = "c11"
compiler = "clang"
# Optional custom flags
cflags = ["-Wall", "-Wextra"]
ldflags = ["-lm"]

[dependencies]
```

Path dependencies are supported. DCR will resolve them on build and generate `dcr.lock`.

## Requirements
- Rust toolchain (`rustc`, `cargo`) - for building DCR from source
- C compiler (`clang`, `gcc`, or 'cl'(msvc))

## Releases
Releases are built automatically via GitHub Actions (`.github/workflows/release.yml`) when a tag matching `v*` is pushed.

## License
See `LICENSE`.
