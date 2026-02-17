# DCR (Dexoron Cargo Realization)

DCR — это утилита для управления проектами на C/C++ в стиле Cargo.

Текущая реализация написана на Rust.

## Зачем DCR
- Единая структура проекта без ручной настройки
- Простые команды для типовых задач
- Прозрачная компиляция и предсказуемые профили сборки

## Возможности
- Создание нового проекта или инициализация текущей директории
- Сборка проекта в профилях `debug` и `release`
- Запуск собранного бинарника
- Очистка результатов сборки
- Генерация минимального шаблона C-проекта
- Обновление бинарника через `dcr --update` (GitHub Releases)

## Поддерживаемые платформы
- Linux: `x86_64-unknown-linux-gnu`
- macOS Intel: `x86_64-apple-darwin`
- macOS Apple Silicon: `aarch64-apple-darwin`
- Windows: `x86_64-pc-windows-msvc`

## Установка

### Из исходников

```sh
git clone https://github.com/dexoron/dcr.git
cd dcr
cargo build --release
mkdir -p ~/.local/bin
ln -sf "$PWD/target/release/dcr" ~/.local/bin/dcr
```

### Через install.sh (Linux/macOS)

```sh
curl -fsSL https://raw.githubusercontent.com/dexoron/dcr/master/install.sh | bash
```

### Через install.ps1 (Windows)

```powershell
irm https://raw.githubusercontent.com/dexoron/dcr/master/install.ps1 | iex
```

Оба скрипта при запуске спрашивают:
- скачать готовый бинарник из GitHub Release
- или собрать проект из `git`

Релизные ассеты:
- `dcr-x86_64-unknown-linux-gnu`
- `dcr-x86_64-apple-darwin`
- `dcr-aarch64-apple-darwin`
- `dcr-x86_64-pc-windows-msvc.exe`

## Быстрый старт

Создать новый проект:
`dcr new hello`

Или инициализировать текущую директорию (директория должна быть пустой):
`dcr init`

Структура проекта:

```txt
hello/
- src/
- - main.c
- dcr.toml
```

Сборка и запуск проекта:
`dcr run` или `dcr run --release`

## Команды

### `dcr new <name>`
Создает проект в текущей директории с указанным именем.

### `dcr init`
Создает проект в текущей директории. Имя проекта будет равно имени директории. Директория должна быть пустой.

### `dcr build [profile]`
Собирает проект. Если профиль не указан, используется `--debug`.

### `dcr run [profile]`
Собирает проект и запускает бинарник. Если профиль не указан, используется `--debug`.

Запуск вручную:
`./target/<profile>/main`

### `dcr clean`
Удаляет папку `target` в корне проекта.

## Профили сборки
Поддерживаются два профиля:
- `--debug` (по умолчанию) — флаги: `-O0 -g -Wall -Wextra -fno-omit-frame-pointer -DDEBUG`
- `--release` — флаги: `-O3 -DNDEBUG -march=native`

## Конфигурация
Основной файл проекта — `dcr.toml`.

Пример `dcr.toml`:

```toml
[package]
name = "hello"
version = "0.1.0"
language = "c"
compiler = "clang"

[dependencies]
```

## Требования
- Rust toolchain (`rustc`, `cargo`) - Для ручной сборки DCR
- C-компилятор (`clang`, `gcc` или другие)

## Релизы
Релизы собираются автоматически через GitHub Actions (`.github/workflows/release.yml`) при пуше тега формата `v*`.

## Лицензия
См. файл `LICENSE`.
