# DCR TODO.md

Priority list of tasks to turn DCR into a true Cargo killer for C/C++ developers.

**Legend:** `[ ]` — not implemented | `[-]` — partially implemented | `[x]` — implemented

## 🔥 High Priority

- [ ] **CLI Output Refactoring & Cleanup**
  - [ ] Move all terminal strings to a dedicated module (`src/ui/messages.rs`)
  - [ ] Standardize logging formats (`error:`, `warning:`, `status:`)
  - [x] Add build execution time statistics to the final output (e.g., `Finished in 4.23s`)
- [ ] **Internationalization (i18n) / Localization**
  - [ ] Integrate a lightweight localization engine (`fluent` or `gettext` pure-Rust port)
  - [ ] Create initial translation catalogs (`en.ftl`, `ru.ftl`)
- [x] **Self-Update Stability Validation**
  - [x] Self-update implemented via `self-replace` + `ureq` in `flag_update.rs`
- [-] **Freestanding / Embedded Automation**
  - *`-ffreestanding` injected at compile time, `-nostdlib -static` at link time for bare-metal targets or `build.freestanding = true`. Config option `build.freestanding` (bool) added.*
- [ ] **Conan and vcpkg integration** Automatic download, installation and linking of packages

## ✅ Medium Priority

- [ ] **Static analysis** (`dcr check`)  
  Integration with clang-tidy, cppcheck, include-what-you-use
- [x] **Linter + Formatter** (`dcr fmt`, `dcr lint`)  
  *`dcr fmt` (clang-format) and `dcr lint` (clang-tidy) are implemented*
- [ ] **Hot reload / Live reload** for applications (raylib, SFML, SDL, etc.)
- [-] **Workspaces improvements** Per-member include directories, selective builds, better dependency graph  
  *Basic workspaces with dep ordering + topological sort work; advanced features missing*
- [-] **Build-time code generation** Support for protobuf, flatbuffers, Qt moc, custom generators  
  *Generic `build.steps` machinery + Qt toolchain (uic/moc/rcc) work; no dedicated protobuf/flatbuffers generators*
- [-] **dcr publish + official package registry** *Registry consumption (fetch, resolve, link) works; `dcr publish` command / local packing missing*

## 📌 Low Priority / Nice to Have

- [ ] Support for Meson and Ninja as alternative backends
- [ ] Dockerfile generation (`dcr docker init`)
- [ ] First-class Windows MSVC support (currently works but rough)
- [ ] Documentation generation (`dcr doc`)
- [ ] Sanitizers integration (`dcr sanitize`)
- [ ] Benchmarks and comparison with CMake/Meson
- [ ] C++20 modules support (when it becomes stable)

## 🧠 Future Ideas

- Official DCR package registry (dcrhub)
- GUI wrapper (dcr-gui)
- Plugin system
- Toolchain manager (like rustup)
- Zig as alternative compiler
- deb/rpm/flatpak/AppImage packaging support

---

**Currently in progress:**
- UI/UX terminal output unification and benchmarking metrics.

**Completed in recent releases:**
- Extreme binary size optimization (-50% weight reduction via `ureq` + `native-certs`)
- Complete removal of `git2` and `openssl` in favor of system Git execution
- Workspaces & Cross-compilation (`--target <triple>`)
- IDE config generation (VS Code, CLion, compile_commands.json)
- EFI support & Linker script support (`build.ldscript`)
- Mixed C/C++/ASM compilation with custom `std::thread::scope` work-stealing engine