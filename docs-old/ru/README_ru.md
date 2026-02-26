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
- Обновление бинарника через `dcr --update` (GitHub Releases, не для pacman-установок)

## Поддерживаемые платформы
- Linux: `x86_64-unknown-linux-gnu`
- macOS Intel: `x86_64-apple-darwin`
- macOS Apple Silicon: `aarch64-apple-darwin`
- Windows: `x86_64-pc-windows-msvc`

## Установка

### Пакетный менеджер

**ArchLinux**
```sh
yay -S dcr # или paru и остальные менеджеры пакетов AUR 
```

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
curl -fsSL https://dcr.dexoron.su/install.sh | bash
```

### Через install.ps1 (Windows)

```powershell
irm https://dcr.dexoron.su/install.ps1 | iex
```

Оба скрипта при запуске спрашивают:
- скачать готовый бинарник из GitHub Release
- или собрать проект из `git`

Релизные ассеты:
- `dcr-x86_64-unknown-linux-gnu`
- `dcr-x86_64-apple-darwin`
- `dcr-aarch64-apple-darwin`
- `dcr-x86_64-pc-windows-msvc.exe`

## Обновление

- Если DCR установлен из GitHub Releases, через `install.sh`, `install.ps1` или собран вручную:
  - используйте `dcr --update`
- Если DCR установлен через `pacman/AUR`:
  - обновляйте через пакетный менеджер: `paru/yay -Syu dcr` или `sudo pacman -Syu dcr`
  - `dcr --update` определяет пакетную установку и подсказывает обновляться через пакетный менеджер

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
`./target/<profile>/<name>`

### `dcr clean`
Удаляет папку `target` в корне проекта.

## Профили сборки
Поддерживаются два профиля:
- `--debug` (по умолчанию) — встроенные флаги для каждого компилятора
- `--release` — встроенные флаги для каждого компилятора

Кастомные флаги можно добавить через `build.cflags` и `build.ldflags` в `dcr.toml`.
Можно указать `build.target`, чтобы задать директорию выхода бинарника (без привязки к профилю).
Используйте `build.kind = "staticlib"`, чтобы собрать статическую библиотеку вместо бинарника.
`dcr run` работает только для `build.kind = "bin"` и завершится ошибкой для `staticlib`.

## Конфигурация
Основной файл проекта — `dcr.toml`.

Пример `dcr.toml`:

```toml
[package]
name = "hello"
version = "0.1.0"

[build]
language = "c"
standard = "c11"
compiler = "clang"
# Опциональные флаги
cflags = ["-Wall", "-Wextra"]
ldflags = ["-lm"]

[dependencies]
```

Поддерживаются path-зависимости. DCR резолвит их при сборке и создаёт `dcr.lock`.

Примечание по инкрементальной сборке: пересборка объектников идёт по изменениям `.c/.cpp`. Учет зависимостей заголовков пока не реализован.

## Требования
- Rust toolchain (`rustc`, `cargo`) - Для ручной сборки DCR
- C-компилятор (`clang`, `gcc` или 'cl'(msvc))

## Релизы
Релизы собираются автоматически через GitHub Actions (`.github/workflows/release.yml`) при пуше тега формата `v*`.

## Лицензия
См. файл `LICENSE`.
