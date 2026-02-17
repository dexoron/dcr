# Вклад в проект

Спасибо за интерес к проекту.

## Требования

- Использовать Rust stable.
- Перед PR код должен проходить форматирование, линт и сборку.

## Быстрый старт

```bash
git clone https://github.com/dexoron/dcr.git
cd dcr
cargo check
```

## Стиль кода

- Форматируйте код через `cargo fmt`.
- Проверяйте предупреждения через `cargo clippy`.
- Держите функции небольшими и сфокусированными.
- Предпочитайте понятные имена вместо "хитрых".

## Проверки

Перед PR запускайте:

```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo check
```

## CI/CD и релизы

- CI: `.github/workflows/ci.yml`
- Release: `.github/workflows/release.yml`
- Релиз запускается по тегу `v*` и публикует бинарники для:
- `x86_64-unknown-linux-gnu`
- `x86_64-apple-darwin`
- `aarch64-apple-darwin`
- `x86_64-pc-windows-msvc`

## Коммиты и PR

- Делайте коммиты понятными.
- В PR указывайте краткое описание и мотивацию изменений.
- Обновляйте документацию при изменении поведения.

## Вопросы

Если что-то непонятно, открой issue или задайте вопрос в PR.

Также можно задать вопрос напрямую:
- TG: @dexoron
- DS: dexoron
