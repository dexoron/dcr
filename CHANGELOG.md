# Changelog

## [0.6.7] - 2026-05-26

Added:

- `build.kind = "efi"` — UEFI PE32+ executable support. Links with `-shared -nostdlib -Wl,-dll -Wl,--subsystem,10`, output via `efi_path()` with `.efi` extension, rejected by `dcr run`.
- `build.kind = "elf"` — bare-metal ELF executable support. Uses `elf_path()` output path, rejected by `dcr run`, no extra linker flags.
- `build.filename` and `build.extension` in `dcr.toml` — complete control over the final artifact name without relying on `package.name`.
  Example:
  ```toml
  [build]
  filename = "KERNEL"
  extension = "EFI"
  ```
  Produces `KERNEL.EFI` (works for `bin`, `staticlib`, and `sharedlib`).
- Automatic injection of `--target=<build.target>` into compiler flags when `build.target` is set in `dcr.toml`. This greatly simplifies clang-based cross-compilation (especially for bare-metal targets like `aarch64-none-elf`).
- Bare-metal targets (containing `none`, `-elf`, `eabi`, `baremetal`) no longer receive DCR's internal default flags (`-g`, `-Wall`, `-Wextra`, `-fno-omit-frame-pointer`, `-DDCR_DEBUG`, etc.). Prevents unwanted sections (`.comment`, debug info, etc.) that break custom linker scripts when `inherit = true`.
- `build.ldscript` in `dcr.toml` — linker script path passed as `-T <path>` to the linker. Essential for bare-metal/embedded/freestanding targets.
- `dcr build --verbose` — prints compiler/linker command lines (also works with `DCR_DEBUG` env var).
- `dcr add <name>` without a source argument — if a registry is configured, DCR looks up the package by name and adds it as a version dependency.

Fixed:

- `build.target` declared in `dcr.toml` is now correctly used as the default target even when `--target` is not passed on the command line (previously the host target was always forced for config resolution).
- `dcr run` with `--target` or `build.target` now correctly finds the binary in the arch-specific target directory.
- `dcr run` checks profile-specific `build.{profile}.kind` before rejecting library builds (previously only checked `build.kind`).
- `dcr clean` now reads `build.target` from `dcr.toml` when no `--target` flag is given.
- Linux: target directory is now consistent — `target/<arch>-unknown-linux-gnu/<profile>/` (no stale `target/<profile>/`).
- `dcr.lock` is now populated with all resolved dependencies (registry, path, git) instead of being always empty.
- Git dependencies are now properly recognized and recorded in `dcr.lock`.
- `is_registry_dep` correctly identifies version-based registry dependency tables (`{ version = "..." }`) and rejects tables with unknown keys.
- `dcr test` accepts `--debug`/`--release` flags (defaults to `debug` profile instead of always `release`).
- Build fingerprint cache (`build_cache_path`) now includes `target_dir` — different targets no longer share a single cache entry.
- `object_path` no longer hardcodes the `src/` prefix — works correctly with custom `build.roots` (e.g. `roots = ["lib"]`).
- `dcr update` now generates `.exe` candidates for all Windows targets, not just `x86_64-pc-windows-msvc`.
- Compiler existence is verified before starting the build (clear error if compiler is not found in PATH).
- Consolidated duplicate `OUTPUT_MUTEX` definitions — MSVC backend now uses the shared mutex from `common.rs`.
- Registry cache root directory is created if missing (prevents cryptic errors when `~/.dcr/` does not exist).

Changed:

- `openssl` dependency is now conditional (`cfg(not(windows))`) — no longer pulled in on Windows, fixing Windows CI builds that lack Perl `Locale::Maketext::Simple`. On Linux (including musl cross-compilation) the vendored OpenSSL build is unchanged.

## [0.6.6] - 2026-05-18

Fixed:

- Path dependencies now work correctly: `dcr add` stores them as `{ path =
"./..." }` tables, `is_registry_dep` properly distinguishes registry strings
from path/git strings, and `deps/mod.rs` resolves both table-form and
legacy string-form path dependencies
- Registry dependency paths no longer hardcoded to `project_root/dcr-index`;
now use `package_root_from_registry_info()` which resolves relative to the
registry cache root (`~/.dcr/`)
- Registry dependencies are now actually built: `build_project_at()` is
called when `include_dir` or `lib_dir` is missing

Added:

- `SIGINT`/`Ctrl+C` handler via `ctrlc` crate — `dcr build` now checks
`BUILD_INTERRUPTED` flag at key points and aborts cleanly
- `utils/build.rs` — extracted shared utilities: `parse_version_info`,
`normalize_target_os`, `resolve_compiler`, `primary_language`,
`resolve_pkg_config_flags` and config helpers. Eliminates code duplication
between `cli/build.rs`, `cli/run.rs`, `cli/clean.rs`, `cli/gen.rs`
- `utils/fs.rs::with_dir` — extracted common directory-scoped execution
- `run_command_sync_output` in `builder/common.rs` with global `OUTPUT_MUTEX`
— synchronized compiler output across all backends (unix_cc, gas, nasm, msvc)
- New helper functions in `deps/register.rs`: `get_registry_cache_root`,
`package_root_from_registry_info`, `registry_include_dir`, `registry_lib_dir`,
`path_from_string_dep`
- Unit tests for `register.rs` (`is_registry_dep`, `package_root_from_registry_info`),
  `deps/mod.rs` (`path_dep_path`, `push_default_lib_dirs`),
  `utils/build.rs` (`normalize_target_os`, `parse_version_info`)
- `dcr tree` command — visual dependency tree viewer (similar to `cargo tree`)

Removed:

- Removed unused dependencies `indicatif` and `console` from `Cargo.toml`

## [0.6.5] - 2026-05-13

Added:

- New dependency registry system
- Package type support (`lib`, `app`, `none`) in `dcr.toml`
- Library packaging functionality: automatically generates `include` and `lib` directories in `target/` for `lib` type projects
- Expanded CI build targets:
  - Linux: `aarch64-unknown-linux-gnu`, `x86_64-unknown-linux-musl`, `aarch64-unknown-linux-musl`
  - Windows: `aarch64-pc-windows-msvc`, `x86_64-pc-windows-gnu`, `aarch64-pc-windows-gnullvm`

Changed:

- `src/core/deps/` modular architecture overhaul
- Native platform pathing defaults for Linux (now dynamic architecture detection)
- Improved `run` command output handling

## [0.6.0] - 2026-05-12

Added:

- Full Git dependency support in `dcr.toml`
  - Support for `git`, `branch`, `tag`, and `rev` fields
  - Automatic cloning and checkout via `git2`
  - Dependencies are stored in `target/<profile>/deps/git/`
- New `dcr add` command for easy dependency management
  - Short syntax: `github:user/repo`, `gitlab:user/repo`, `path:/to/lib`
  - Automatic GitHub resolution for `git:user/repo`
- Visual feedback for dependency fetching with in-place status updates ("Fetching" → "Fetched")
- Modular dependency management architecture in `src/core/deps/`

## [0.5.1] - 2026-04-12

Fixed:

- Windows compatibility for target-specific config without --target flag

Added:

- Default host target detection for automatic target overrides
- Unit tests for target normalization

Improved:

- Config value lookup optimization
- Documentation for Windows cross-compilation

## [0.5.0] - 2026-04-10

Added:

- Full inheritance system for all config sections: `build`, `toolchain`, `run`, `workspace`, `dependencies`
  - Order: `section.target.profile` → `section.profile.target` → `section.target` → `section.profile` → `section`
  - `inherit = false` to disable inheritance completely
  - Target-specific overrides: `[build.linux]`, `[toolchain.windows]`, `[run.macos]`, etc.
- Cross-compilation support with `--target <triple>`
  - Full target triples and short names (`linux`, `macos`, `windows`)
  - Target-specific config sections: `[build.<target>]`, `[toolchain.<target>]`, `[run.<target>]`
- Workspace and dependencies support target/profile inheritance
- Multiple targets build: `[build.targets]` to build for multiple targets simultaneously
- Target directory structure: `target/<target>/<profile>/` (native or triple)
- Updated documentation with cross-compilation guide and Arch Linux examples

## [0.4.0] - 2026-04-10

Added:

- Test command: `dcr test` and `dcr tests`
  - Automatic compilation and linking for tests without manual configuration
  - `dcr test --init` creates test template and header
  - Test framework with EXPECT/SKIP macros in `dcr_test.h`
- Improved test integration: added test CLI module and basic test suite
- Documentation updates for testing functionality

## [0.3.0] - 2026-03-18

Added:

- `dcr gen` command for IDE integration and build tool support
    - `dcr gen project-info`: Output project metadata as JSON
    - `dcr gen compile-commands`: Generate `compile_commands.json` for clangd and clang-tidy
    - `dcr gen vscode`: Generate VS Code `.vscode/` integration (launch.json, tasks.json, settings.json)
    - `dcr gen clion`: Generate JetBrains CLion `.idea/` integration (run configurations, build targets)
- Full workspace support for all `gen` subcommands

## [0.2.10] - 2026-03-16

Added:

- `build.steps` for pre-build generators (e.g., `moc`/`uic`/`rcc`)
- `build.pkg_config` for `pkg-config`-driven cflags/ldflags resolution
- `[toolchain]` support for `uic`, `moc`, `rcc`
- Step command variables (`{uic}`, `{moc}`, `{rcc}`, `{cflags}`, `{stem}`, `{in}`, `{out}`)
- `{profile}` placeholder support for dependency paths

## [0.2.9] - 2026-03-14

Added:

- Header dependency tracking for fine-grained incremental builds (rebuilds when `#include` files change)
- Complete internal rewrite of the builder module: merged duplicate compilation backends

Changed:

- `cflags` and `default_flags` no longer incorrectly leak to the linker stage in GCC/Clang
- Reduced builder codebase size by over 20% while increasing reliability
- Greatly expanded test suite with 34 new tests covering configuration, CLI errors, and path operations

## [0.2.8] - 2026-03-06

Added:

- Workspace support (`[workspace]` with deps ordering)
- `clean --all` for workspace members
  Changed:
- Project root discovery for `build/run/clean` (searches parent dirs)
- Workspace paths are excluded from root source scan
- Build output now uses a Cargo-style `Compiling <name> v<version>` line
- Build time now reports total time for the whole build stage (workspace included)
- Cache hits are silent (no compile line when nothing is rebuilt)
- Safer build cache: fingerprints include headers and resolved library files

## [0.2.7] - 2026-03-03

Added:

- Toolchain overrides via `[toolchain]` and env (`DCR_*`)
- ASM `.S` preprocessing via GCC/Clang (`-x assembler-with-cpp`)

## [0.2.6] - 2026-03-01

Added:

- `sharedlib` build kind (shared library output)
- ASM projects (`build.language = "asm"`)
- NASM and GAS backends
- `build.platform` architecture hint for `-march` / `/arch`
- Basic CLI integration tests

## [0.2.5] - 2026-02-26

Added:

- Incremental builds (object caching by mtime)
- `build.kind` with `staticlib` support
- Custom output directory via `build.target`

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
