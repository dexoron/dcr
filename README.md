<h1 align="center">DCR — Dexoron Cargo Realization</h1>

<p align="center"><strong>A Cargo-style build tool for C/C++ projects</strong></p>

<p align="center">
  <a href="https://github.com/dexoron/dcr/actions/workflows/ci.yml"><img alt="CI" src="https://github.com/dexoron/dcr/actions/workflows/ci.yml/badge.svg"></a>
  <a href="https://github.com/dexoron/dcr/releases/latest"><img alt="GitHub Release" src="https://img.shields.io/github/v/release/dexoron/dcr"></a>
  <a href="https://github.com/dexoron/dcr/releases"><img alt="Platform" src="https://img.shields.io/badge/platform-linux%20%7C%20macos%20%7C%20windows-lightgrey"></a>
  <br>
  <a href="https://aur.archlinux.org/packages/dcr"><img alt="AUR" src="https://img.shields.io/aur/version/dcr"></a>
  <a href="https://crates.io/crates/dcr"><img alt="Crates.io" src="https://img.shields.io/crates/v/dcr"></a>
  <a href="https://github.com/dexoron/homebrew-dexoron"><img alt="Homebrew" src="https://img.shields.io/badge/homebrew-dexoron%2Fdexoron-orange"></a>
  <br>
  <a href="https://github.com/dexoron/dcr/stargazers"><img alt="GitHub Stars" src="https://img.shields.io/github/stars/dexoron/dcr?style=flat"></a>
  <a href="LICENSE"><img alt="License: GPL-3.0" src="https://img.shields.io/badge/License-GPL--3.0-blue.svg"></a>
</p>

<!-- Snap badge removed: classic confinement was denied for this project. -->

---

## Why DCR?

- **No boilerplate** — one config file, predictable structure
- **Cargo-like workflow** — `build`, `run`, `clean`, `test`, `add`, `fmt`
- **Cross-compilation** — full target triple support with short names
- **IDE integration** — VS Code, CLion, `compile_commands.json` out of the box
- **Dependencies** — path, git, and registry-based with lock file
- **Man pages** — `man dcr` and per-command pages (`man dcr-build`, `man dcr-new`, etc.)

---

## Installation

<table>
<tr>
<td><b>Linux (Dexoron Packages)</b></td>
<td>

See https://dcr.dexoron.su for settings youre packages manager

**Arch Linux:**

```sh
sudo pacman -Sy dcr
```

**Debian/Ubuntu:**

```sh
sudo apt update && sudo apt install dcr
```

**Fedora/RHEL:**

```sh
sudo dnf install dcr
```

</td>
</tr>
<tr>
<td><b>Arch Linux (AUR)</b></td>
<td>

```sh
yay -S dcr
```

</td>
</tr>
<tr>
<td><b>macOS/Linux(GNU) (Homebrew)</b></td>
<td>

```sh
brew tap dexoron/dexoron
brew install dcr
```

</td>
</tr>
<tr>
<td><b>Nix (flake)</b></td>
<td>

```sh
nix run github:dexoron/dcr
# or install to profile:
nix profile install github:dexoron/dcr
```

</td>
</tr>
<tr>
<td><b>Cargo (crates.io)</b></td>
<td>

```sh
cargo install dcr
```

</td>
</tr>
<tr>
<td><b>Linux / macOS (script)</b></td>
<td>

```sh
curl -fsSL https://dcr.dexoron.su/install.sh | sh
```

</td>
</tr>
<tr>
<td><b>BSD (script)</b></td>
<td>

```sh
curl -fsSL https://dcr.dexoron.su/install_bsd.sh | sh
```

</td>
</tr>
<tr>
<td><b>Snap (Linux)</b></td>
<td>

> Snap Store publishing temporarily disabled — see [#51671](https://forum.snapcraft.io/t/classic-confinement-request-for-dcrup/51671).
> Download the `.snap` from [GitHub Releases](https://github.com/dexoron/dcr/releases/latest) and install manually:

```sh
sudo snap install --classic --dangerous dcrup_*.snap
```

</td>
</tr>
<tr>
<td><b>Windows (PowerShell)</b></td>
<td>

```powershell
irm https://dcr.dexoron.su/install.ps1 | iex
```

</td>
</tr>
<tr>
<td><b>From source</b></td>
<td>

```sh
git clone https://github.com/dexoron/dcr.git
cd dcr && cargo build --release
ln -sf "$PWD/target/release/dcr" ~/.local/bin/dcr
```

</td>
</tr>
</table>

---

## Quick Start

```sh
dcr new hello
cd hello
dcr run
```

**Project structure:**

```
hello/
├── dcr.toml       # project config
└── src/
    └── main.c
```

---

## dcr.toml Example

```toml
[package]
name = "hello"
version = "0.1.0"

[build]
language = "c"
standard = "c11"
compiler = "clang"
kind = "bin"

[dependencies]
```

---

## Base Commands

| Command | Description |
|---|---|
| `dcr new <name>` | Create a new project |
| `dcr init` | Initialize current directory |
| `dcr add <dep>` | Add a dependency |
| `dcr build [--release]` | Build the project |
| `dcr run [--release]` | Build and run |
| `dcr clean` | Remove build artifacts |
| `dcr test` | Run project tests |
| `dcr fmt` | Format C/C++ sources (clang-format) |
| `dcr tree` | View dependency tree |
| `dcr gen <vscode\|clion\|compile-commands>` | Generate IDE integration |
| `dcr setup` | Initialize DCR registry |

other in [docs/](docs/), `dcr --help` and `dcr <command> --help`

---

## Platforms (pre-built binary)

<div align="center">

| OS | x86_64 | aarch64 | armv7 | i686 | riscv64 |
|:---:|:---:|:---:|:---:|:---:|:---:|
| Linux (GNU)     | ✅ | ✅ | ✅ | ✅ | ✅ |
| Linux (Musl)    | ✅ | ✅ | ✅ | ✅ | 🚧 |
| macOS           | ✅ | ✅ | ❌ | ❌ | ❌ |
| Windows (MSVC)  | ✅ | ✅ | ❌ | ✅ | ❌ |
| Windows (MinGW) | ✅ | 🚧 | ❌ | ❌ | ❌ |
| FreeBSD         | ✅ | 🚧 | ❌ | ❌ | ❌ |
| OpenBSD         | ✅ | 🚧 | ❌ | ❌ | ❌ |
| NetBSD          | ✅ | 🚧 | ❌ | ❌ | ❌ |

✅ pre-built binary · 🚧 planned · ❌ not supported by platform or CI

</div>

---

## Documentation

Full docs at **[dcr.dexoron.su](https://dcr.dexoron.su)** or in the [`docs/`](docs/) directory.

---

## Contributors

<div align="center">

| | Name | Role | GitHub |
|:---:|---|---|---|
| 👤 | Dexoron (Bezotechestvo Vladimir) | Maintainer, Creator | [@dexoron](https://github.com/dexoron) |
| 👤 | Kai | Maintainer | [@peoplemiau1](https://github.com/peoplemiau1) |

</div>

---

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). Before a PR:

```sh
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

---

<div align="center">

GPL-3.0 — see [LICENSE](LICENSE)<br/>
DCR is a build tool, not a library. The GPL applies only to DCR's own source code, not to projects built with it.<br/>
Made with ❤️ by [Dexoron](https://github.com/dexoron) and contributors.

</div>
