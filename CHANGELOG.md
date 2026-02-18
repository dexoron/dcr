# Changelog

## [0.2.1] - 2026-02-18
Changed:
- Translated user-facing CLI messages to English
- Unified error and warning output via `utils::log::{error, warn}`
- Updated `--help` output: translated headers and examples to English, used `printc`, and applied `BOLD_*` styles
- Translated installer script messages in `install.sh` and `install.ps1` to English

## [0.2.0] - 2026-02-17
Изменено:
- Проект переведен с Python на Rust
- CLI и команды (`new`, `init`, `build`, `run`, `clean`, `--help`, `--version`, `--update`) перенесены на Rust
- Обновлен флаг `--update` - добавлена поддержка GNU\Linux, Windows, MacOS
- Обновлены `README.md`, `CONTRIBUTING.md`, `install.sh` под Rust-реализацию
- Добавлен `install.ps1` для Windows
- Обновлен `install.sh` для GNU/Linux и MacOS

Добавлено:
- Добавлен `install.ps1` для Windows
- Поддержка GNU\Linux, Windows, MacOS(x86_64/arm)

ВАЖНО:
- Код перенесен с помощью нейронных сетей, в будущих версиях будет баг-фикс и изменение логики

## [0.1.2] - 2026-02-12
Добавлено:
- Команда обновления
- Скрипт установки `install.sh` и инструкции по установке в README
- Флаг `--version` для вывода версии

Изменено:
- Улучшен консольный вывод команд, добавлены цвета и обновлена справка `--help`
- Обновлен запуск проекта и обработка сборки в `run.py`/`build.py`

## [0.1.1] - 2026-02-11
Изменено:
- Обновлена справка `--help`
- `main.py` теперь корректно запускается при прямом выполнении

## [0.1.0] - 2026-02-11
Первый публичный релиз.

Добавлено:
- Базовые команды `new`, `init`, `build`, `run`, `clean`
- Профили сборки `debug` и `release`
- Шаблоны `dcr.toml` и `src/main.c`
