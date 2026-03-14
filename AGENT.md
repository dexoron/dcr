# AGENT.md

## Project Overview

**DCR (Dexoron Cargo Realization)** — CLI-утилита для управления C/C++/ASM проектами в стиле Cargo.

- **Язык реализации**: Rust (edition 2024)
- **Лицензия**: MIT
- **Репозиторий**: [github.com/dexoron/dcr](https://github.com/dexoron/dcr)

## Architecture

### Source Tree (`src/`)

```
src/
├── main.rs          # Точка входа, маршрутизация CLI-команд
├── config.rs        # Глобальные константы: профили сборки, шаблон main.c
├── cli/             # Реализация CLI-команд
│   ├── new.rs       # dcr new <name>
│   ├── init.rs      # dcr init
│   ├── build.rs     # dcr build [--debug|--release]
│   ├── run.rs       # dcr run [--debug|--release]
│   ├── clean.rs     # dcr clean [--all]
│   ├── help.rs      # dcr --help
│   └── flag_update.rs  # dcr --update (self-update через GitHub Releases)
├── core/            # Бизнес-логика
│   ├── config.rs    # Парсинг/валидация/форматирование dcr.toml (struct Config)
│   ├── deps.rs      # Разрешение path-зависимостей, генерация dcr.lock
│   ├── runner.rs    # Запуск скомпилированного бинарника
│   ├── workspace.rs # Поддержка [workspace] и упорядочивание сборки
│   └── builder/     # Бэкенды компиляторов/ассемблеров
│       ├── mod.rs   # Общий интерфейс сборки
│       ├── gcc.rs   # GCC backend
│       ├── clang.rs # Clang backend
│       ├── msvc.rs  # MSVC backend
│       ├── nasm.rs  # NASM backend (ASM)
│       └── gas.rs   # GAS backend (ASM)
├── platform/        # Платформенно-зависимая логика
│   ├── mod.rs       # Общий интерфейс платформ
│   ├── linux.rs
│   ├── macos.rs
│   └── windows.rs
└── utils/           # Вспомогательные утилиты
    ├── fs.rs        # Работа с файловой системой
    ├── log.rs       # Логирование (error, warn)
    └── text.rs      # Форматирование текста / цвета
```

### Key Concepts

- **`dcr.toml`** — конфигурационный файл проекта (аналог `Cargo.toml`). Секции: `[package]`, `[build]`, `[toolchain]`, `[dependencies]`, `[workspace]`.
- **Профили сборки**: `debug` (по умолчанию) и `release`. Встроенные флаги зависят от компилятора. Пользовательские флаги через `build.cflags` / `build.ldflags`.
- **Инкрементальная сборка**: объектные файлы кешируются по mtime. Отслеживание заголовков пока не реализовано (см. `TODO.md`).
- **Workspace**: корневой `dcr.toml` с секцией `[workspace]` описывает member-проекты и их зависимости; сборка происходит в топологическом порядке.
- **Path dependencies**: зависимости по путям автоматически добавляют include/lib-пути; генерируется `dcr.lock`.
- **Build kinds**: `bin` (бинарник), `staticlib` (`.a`), `sharedlib` (`.so`/`.dylib`/`.dll`).

## Supported Platforms

| Platform | Target triple |
|---|---|
| Linux | `x86_64-unknown-linux-gnu` |
| macOS Intel | `x86_64-apple-darwin` |
| macOS ARM | `aarch64-apple-darwin` |
| Windows | `x86_64-pc-windows-msvc` |

## Dependencies (Cargo)

| Crate | Purpose |
|---|---|
| `reqwest` | HTTP-запросы для `--update` (blocking, rustls-tls) |
| `serde` | Сериализация/десериализация |
| `toml` | Парсинг `dcr.toml` |
| `sha2` | Хеширование при обновлении |
| `self-replace` | Замена бинарника при `--update` |

## Development

### Build & Run

```bash
cargo build             # debug build
cargo build --release   # release build
cargo run -- new hello  # создать проект "hello"
```

### Code Quality

```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo check
cargo test
```

### CI/CD

- **CI**: `.github/workflows/ci.yml` — проверки при PR/push
- **Release**: `.github/workflows/release.yml` — сборка бинарников при пуше тега `v*`

### Tests

- Интеграционные CLI-тесты: `tests/cli_basic.rs`

## Code Conventions

- Rust stable edition 2024
- Форматирование: `cargo fmt`
- Линтинг: `cargo clippy` без warnings
- Функции — маленькие и сфокусированные
- Имена — понятные и описательные
- Пользовательские сообщения CLI — на английском языке
- Вывод ошибок/предупреждений через `utils::log::{error, warn}`

## TODO

- В файле `TODO.md`

## Useful Links

- Документация проекта: `docs/`
- Changelog: `CHANGELOG.md`
- Contributing: `CONTRIBUTING.md`
- FAQ: `FAQ.md`
- Установка: `install.sh` (Linux/macOS), `install.ps1` (Windows)
