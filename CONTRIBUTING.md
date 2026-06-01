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

## Target System

DCR supports cross-compilation via `--target <triple>` or short names:

- `dcr build --target linux --release` (maps to `x86_64-unknown-linux-gnu`)
- `dcr build --target macos --release` (maps to `x86_64-apple-darwin`)
- `dcr build --target windows --release` (maps to `x86_64-pc-windows-msvc`)
- Full triples: `dcr build --target aarch64-linux-gnu --release`
- Configure multiple targets in `dcr.toml`:

  ```toml
  [build]
  targets = ["linux", "macos"]

  [build.linux]
  compiler = "gcc"

  [build.macos]
  compiler = "clang"
  ```

- If no `--target`, builds for targets in `build.targets` or native if empty.

## Checks

Before PR, run:

```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo check
cargo test
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

## License

This project is licensed under the **GNU General Public License v3.0**. By contributing, you agree that your contributions will be licensed under the same license.

## Questions

If anything is unclear, open an issue or ask in the PR.

You can also reach out directly:

- TG: @dexoron
- DS: dexoron
