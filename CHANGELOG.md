# Changelog

## [0.2.4] - 2026-02-22
Added:
- Custom build flags: `build.cflags` and `build.ldflags`
- Recursive source discovery inside `src/`
- Path dependencies with auto include/lib resolution and `dcr.lock` generation (experimental)
- Add docs/index.md and docs/dcr-toml.md

## [0.2.3] - 2026-02-21
Added:
- Modular builders for `gcc`, `clang`, `msvc`
- Platform-specific binary path generation (Linux/macOS/Windows)

Changed:
- Build configuration moved to `[build]` (`language`, `standard`, `compiler`)
- `dcr.toml` formatting now includes the `[build]` section
- Build uses built-in `debug/release` flags per compiler
- Build compiles all `*.c/*.cpp` files in `src/` into a single binary (no incremental build)
- Updated `dcr.toml` examples in documentation

## [0.2.2] - 2026-02-20
Changed:
- Reworked `dcr.toml` handling: added read, validation, and edit through `core::config`
- `new` and `init` use the new config creation logic
- `build` and `run` now require `dcr.toml` and read `compiler` and `name` from it

## [0.2.1] - 2026-02-18
Changed:
- Translated user-facing CLI messages to English
- Unified error and warning output via `utils::log::{error, warn}`
- Updated `--help` output: translated headers and examples to English, used `printc`, and applied `BOLD_*` styles
- Translated installer script messages in `install.sh` and `install.ps1` to English

## [0.2.0] - 2026-02-17
Changed:
- Project migrated from Python to Rust
- CLI and commands (`new`, `init`, `build`, `run`, `clean`, `--help`, `--version`, `--update`) ported to Rust
- Updated `--update` flag: added support for GNU/Linux, Windows, macOS
- Updated `README.md`, `CONTRIBUTING.md`, and `install.sh` for the Rust implementation
- Added `install.ps1` for Windows
- Updated `install.sh` for GNU/Linux and macOS

Added:
- Added `install.ps1` for Windows
- Support for GNU/Linux, Windows, macOS (x86_64/arm)

Important:
- Code was ported with neural networks; future versions will include bug fixes and logic changes

## [0.1.2] - 2026-02-12
Added:
- Update command
- `install.sh` install script and README instructions
- `--version` flag

Changed:
- Improved CLI output, added colors, and updated `--help`
- Updated project run/build handling in `run.py`/`build.py`

## [0.1.1] - 2026-02-11
Changed:
- Updated `--help`
- `main.py` now runs correctly when executed directly

## [0.1.0] - 2026-02-11
First public release.

Added:
- Base commands `new`, `init`, `build`, `run`, `clean`
- Build profiles `debug` and `release`
- `dcr.toml` and `src/main.c` templates
