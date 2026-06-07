# Changelog

## [0.7.1] - 2026-06-02 "Мультиархитектурное расширение / The Multi-Arch Expansion"

### RU

**Добавлено:**
- Вывод `--help` для всех команд с цветным форматированием.
- man-страницы (12 штук в `man/man1/`) и их установка во всех пакетах.
- Валидация имен проектов в `dcr init` и `dcr new` (только ASCII буквы, цифры, `_` и `-`).
- Метаданные `documentation` и `homepage` в `Cargo.toml`.
- Оптимизация сборки: `opt-level = "z"`, LTO, `codegen-units = 1`, `panic = "abort"`, `strip = true`.
- Поддержка новых архитектур (i686, armv7, riscv64gc, aarch64) в рабочих процессах и установочных скриптах.
- Сборка пакетов `deb` и `rpm` для всех 5 архитектур Linux через `cargo-zigbuild`.
- Пакеты AUR (`dcr-dev`, `dcr-dev-bin`) и сборка Snap в CI.

**Изменено:**
- Функция `validate_package_name` стала публичной.
- Команды `gen`, `tree`, `fmt`, `setup` больше не игнорируют аргументы и используют цветной вывод.
- Инструкция в README изменена с `| bash` на `| sh` для POSIX.
- Отключена архитектура Windows 32-bit (GNU) из-за проблем со сборкой.

**Исправлено:**

- **`dcr new <invalid-name>` создаёт директорию, затем падает** — валидация теперь происходит до операций с файлами.
- **`dcr init` создаёт dcr.toml, затем падает при невалидном имени директории** — то же исправление.
- **Man-страницы отсутствовали после установки** — теперь устанавливаются через скрипты установки, пакетные менеджеры и Homebrew.
- **Ложное срабатывание на букву диска в Windows** — в парсинге файлов зависимостей.
- **CRLF `\r\n` ломает парсер зависимостей** — теперь обрабатываются смешанные окончания строк.
- **Новая строка после обратного слеша не поглощалась** — в обработке escape-последовательностей `parse_d_file`.

### EN

**Added:**
- `--help` for all commands with styled headers and usage lines.
- Man pages (12 troff man pages in `man/man1/`) and their installation across all packaging formats.
- Project name validation in `dcr init` and `dcr new` (ASCII letters, digits, `_`, `-` only).
- `documentation` and `homepage` metadata in `Cargo.toml`.
- Release profile optimizations (`opt-level = "z"`, LTO, `codegen-units = 1`, `panic = "abort"`, `strip = true`).
- Linux i686, armv7, riscv64gc, aarch64 architecture support in release workflows and installation scripts.
- Multi-arch `deb` and `rpm` package builds for all 5 Linux architectures via `cargo-zigbuild`.
- AUR packages (`dcr-dev`, `dcr-dev-bin`) and Snap build in CI.

**Changed:**
- `validate_package_name` made public and callable from CLI commands directly.
- `gen.rs`, `tree.rs`, `fmt.rs`, `setup.rs` no longer ignore arguments and use styled output.
- README install instructions changed from `| bash` to `| sh` for POSIX compatibility.
- Disabled Windows 32-bit (GNU) target due to build issues.

Fixed:

- **`dcr new <invalid-name>` creates directory then fails** — validation now happens
  before any file operations.
- **`dcr init` creates dcr.toml then fails on invalid directory name** — same fix.
- **Man pages missing after installation** — now installed via install scripts,
  package managers, and Homebrew.
- **Windows drive letter false positive** — in dependency file parsing.
- **CRLF `\r\n` breaks dependency parser** — now handles mixed line endings.
- **Newline after backslash not consumed** — in `parse_d_file` escape handling.

## [0.7.0] - 2026-06-02

Added:

- **OpenBSD and NetBSD target support** — full platform routing with dynamically generated target triples using `std::env::consts::ARCH` and `std::env::consts::OS`. Affects `build`, `run`, `clean`, and the platform module.
- **`src/platform/bsd.rs`** — new dedicated BSD platform module shared by FreeBSD, OpenBSD, NetBSD, providing `bin_path`, `lib_path`, `elf_path`, `efi_path`, and `shared_lib_path`.
- **`build.out_dir` configuration option** — custom output directory that overrides the default `target/<triple>/<profile>` path for final artifacts. Supported in `build`, `run`, and config validation.
- **`dcr fmt` command** — new CLI command that formats all C/C++ source files (`*.c`, `*.cpp`, `*.h`, `*.hpp`) in `src/` and `tests/` using `clang-format`.
- **Incremental linking** — `needs_link()` function in `common.rs` that checks whether any object file is newer than the linked output, skipping unnecessary re-linking. Implemented for all backends (`unix_cc`, `msvc`, `gas`, `nasm`).
- **`build.kind = "none"` and `"custom"`** — two new project kind types for special build scenarios where no standard artifact is produced.
- **`install_bsd.sh`** — dedicated POSIX-compliant install script for BSD systems (FreeBSD, OpenBSD, NetBSD) with binary download and source build modes.
- **Linux ARM64 support in `install.sh`** — added `Linux:aarch64|Linux:arm64` target triple detection for pre-built binary downloads.
- **BSD OS detection in `install.sh`** — FreeBSD, OpenBSD, NetBSD detection and target triple resolution.
- **`rust-toolchain.toml`** — explicit `stable` channel toolchain pinning.
- **Nix support** — `flake.nix` for Nix package manager. Run via `nix run github:dexoron/dcr` or install with `nix profile install github:dexoron/dcr`. Contributed by community contributor.
- **Snapcraft publish** — `dcrup` published on Snapcraft. Install with `sudo snap install dcrup`.
- **Integration tests** — `build_with_target_config` (verifies `build.target = "linux"`) and `build_with_out_dir` (verifies custom output directory).
- **`get_build_string_with_profile` made `pub`** — so `run.rs` can resolve custom output directory configuration.

Changed:

- **`build.target` semantics changed** — now strictly holds the target triple (e.g. `x86_64-unknown-linux-gnu`) or a short name (`linux`, `macos`, `windows`). No longer doubles as a custom output directory — that functionality moved to `build.out_dir`.
- **`build.standard` made optional** — changed from `String` to `Option<String>`. Validation only enforces non-empty for non-ASM languages. Skipped in `dcr.toml` output when empty.
- **`dcr run` with `out_dir`** — now resolves target directory respecting `build.out_dir` when configured, via `get_build_string_with_profile()` from `build.rs`.
- **`collect_sources()` returns empty vec** instead of error when no source files are found, allowing `kind = "none"` or `kind = "custom"` projects to have no source files.
- **CI/CD release workflow refactored** — `git2` made target-specific (no vendored-openssl on Windows), Zig-based cross-compilation for non-x86_64 Linux targets, Arch Linux package version sanitization (dashes → dots), NetBSD `gmake` symlink.
- **README compatibility table** — FreeBSD, OpenBSD, NetBSD build/runtime status upgraded from community/best-effort to officially supported.
- **Documentation rewritten** — available at [dcr.dexoron.su/docs](https://dcr.dexoron.su/docs) or in the `docs/` directory. Installation guides, commands, and reference updated to reflect current CLI and config format.
- **Site rewritten** — `dcr-site` migrated to Docusaurus with internationalization (i18n) support, improved navigation, and updated content across all sections.
- **Dev channel dependency check in `install.sh`** — checks for `python3` or `jq` before attempting dev channel installations.

Fixed:

- **`dcr run` stdout/stderr not inherited** — child process output was captured and manually printed, breaking interactive programs. Fixed by switching to `Command::status()`.
- **Release CI race condition** — build matrix jobs could upload assets to a release that did not yet exist. Fixed by adding a dedicated `create-release` job.
- **GHA release — stable toolchain override** — resolved Rust toolchain override issues in CI.
- **Arch Linux package version sanitization** — version strings with dashes (e.g. `0.7.0-dev`) are invalid for Arch Linux `pkgver`. Fixed by replacing dashes with dots.
- **GPG permissions after Docker** — Docker operations changed GPG directory ownership. Fixed by running `chown` after Docker commands.
- **RPM package artifact paths** — RPM artifacts placed under `rpm/x86_64/` instead of `fedora/x86_64/`. Corrected in Dexoron Packages Index workflow.
- **`install.sh` JSON parsing robustness** — added `jq` as primary parser with `python3` fallback for dev channel release lookups.

Removed:

- **`format_roots()` helper** — removed from `common.rs`. Was only used by the old error path in `collect_sources()`.
- **Individual per-distro artifact download steps** — three separate `actions/download-artifact` steps replaced with unified `gh release download --clobber`.

## [0.6.9] - 2026-05-28

Added:

- **Dexoron Packages Index** — new standalone CI workflow (`.github/workflows/dexoron-packages-index.yml`)
  that automatically updates the `dexoron/packages` repository indexes after a release.
  Supports Arch Linux (`repo-add`), Debian (`dpkg-scanpackages`), and Fedora (`createrepo_c`).
  Triggered by `workflow_dispatch` or `workflow_run` on the Release workflow.

- **Arch Linux native package** (`.pkg.tar.zst`) build inside the release workflow —
  builds from source in an `archlinux:latest` Docker container using a dynamically
  generated PKGBUILD.

- **AUR publish support** — new `run_aur` workflow dispatch input and `publish-aur` /
  `publish-aur-bin` jobs that push PKGBUILD / `.SRCINFO` to `aur.archlinux.org`.

Changed:

- **Release workflow refactored** — Linux packages (deb, rpm, snap, arch) and BSD
  builds now use `strategy.matrix` instead of separate jobs. A new
  `publish-dexoron-packages` job collects all package artifacts and uploads them
  to the GitHub Release in one step. Removed standalone `build-arch-package`,
  `build-snap`, `build-deb-rpm` jobs. The old `run_arch_native` input is now
  `run_linux_packages`.

- **Dexoron Packages Index workflow** extracted from `release.yml` into its own
  file with support for Debian and RPM repository indexes in addition to Arch.

- **git2 vendoring** — enabled `vendored-libgit2` and `vendored-openssl` features
  for reproducible CI builds without system libraries.

- **cargo-generate-rpm metadata** — updated to the current `package.metadata.rpm`
  format (uses `files` table instead of `assets` array).

Fixed:

- **`--target=<triple>` no longer passed to GCC** — the flag is now only injected into
  cflags when the resolved compiler name contains `"clang"`. GCC rejects this flag
  with `unrecognized command-line option`.

- **Workspace clean test** — removed assertions for member-local `target/`
  directories. Workspace members share the root `target/` directory, so their
  individual `src/<name>/target/` is never created.

- **Release build test path** — corrected expected directory from `target/release`
  to `target/x86_64-unknown-linux-gnu/release`.

## [0.6.8] - 2026-05-27

Added:

- `build.cxx_standard` — separate C++ language standard for `.cpp`/`.cxx`/`.cc` files.
  When set, overrides `build.standard` for C++ sources. Example:

  ```toml
  [build]
  language = ["c", "c++"]
  standard = "gnu11"
  cxx_standard = "gnu++17"
  ```

- Variable substitution for `cflags`, `ldflags`, and `include` directory entries.
  All `{version}`, `{version_major}`, `{name}`, `{profile}` and other step variables
  are now available. Example:

  ```toml
  [build]
  cflags = ['-DPROJECT_VERSION="{version}"']
  ```

- FreeBSD build support in CI — new `run_bsd` workflow input and `build-bsd` job
  that cross-compiles `x86_64-unknown-freebsd` via `cargo-zigbuild`.

Changed:

- License changed from MIT to **GPL-3.0-or-later**. All source files now carry
  the GPL-3.0 header. Default license in `dcr new` / `dcr init` changed to
  `"GPL-3.0-or-later"`. README, FAQ, and CONTRIBUTING.md updated accordingly.
- `get_list_with_profile_and_target` no longer duplicates list values when
  `inherit = true` and the profile/target value is identical to the base value.

Fixed:

- Multi-language `language` array (e.g. `["c", "c++", "asm"]`) no longer drops `c++`
  during internal parsing — `.cxx`/`.cpp`/`.cc` source files are correctly discovered
  and compiled.
- `-std=` flag is no longer passed to C++ files when only `standard` (C standard) is
  configured — uses `cxx_standard` when available, otherwise skips `-std=` for C++
  sources, avoiding clang error `invalid argument '-std=c11' not allowed with 'C++'`.
- Object file collision: source files with the same stem but different extensions
  (e.g. `gdt.cxx` and `gdt.S`) no longer overwrite each other. Object filenames
  now include the original source extension (e.g. `gdt.cxx.o`, `gdt.S.o`).
- MSVC backend compile step now detects language per source file extension instead
  of using the global `language` field, fixing mixed C/C++ compilation with MSVC.
- Assembler `-x` flag in `unix_cc.rs` and `gen.rs` is now placed **before** the
  source file, as required by clang/gcc.
- `{version}` variables in `build.clean` are now correctly substituted.

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
- - `dcr gen project-info`: Output project metadata as JSON
- - `dcr gen compile-commands`: Generate `compile_commands.json` for clangd and clang-tidy
- - `dcr gen vscode`: Generate VS Code `.vscode/` integration (launch.json, tasks.json, settings.json)
- - `dcr gen clion`: Generate JetBrains CLion `.idea/` integration (run configurations, build targets)
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
