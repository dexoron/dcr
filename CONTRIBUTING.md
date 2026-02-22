# Contributing

Thanks for your interest in the project.

## Requirements

- Use Rust stable.
- Before opening a PR, the code must pass formatting, linting, and build.

## Quick Start

```bash
git clone https://github.com/dexoron/dcr.git
cd dcr
cargo check
```

## Code Style

- Format code with `cargo fmt`.
- Check warnings with `cargo clippy`.
- Keep functions small and focused.
- Prefer clear names over clever ones.

## Checks

Before PR, run:

```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo check
```

## CI/CD and Releases

- CI: `.github/workflows/ci.yml`
- Release: `.github/workflows/release.yml`
- Releases are triggered by tags `v*` and publish binaries for:
- `x86_64-unknown-linux-gnu`
- `x86_64-apple-darwin`
- `aarch64-apple-darwin`
- `x86_64-pc-windows-msvc`

## Commits and PRs

- Write clear commit messages.
- In PRs, include a short description and rationale.
- Update documentation when behavior changes.

## Questions

If anything is unclear, open an issue or ask in the PR.

You can also reach out directly:
- TG: @dexoron
- DS: dexoron
