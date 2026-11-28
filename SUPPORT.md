# Support

Thanks for using DCR! This page explains where to get help.

## Before You Ask

1. **Read the docs** — [dcr-tool.ru/docs](https://dcr-tool.ru/docs) or the [`docs/`](docs/) directory in this repository.
2. **Use built-in help** — `dcr --help`, `dcr <command> --help`, `man dcr`, `man dcr-build`.
3. **Check the [FAQ](FAQ.md)** and the [docs FAQ](docs/faq.mdx).
4. **Search existing [issues](https://github.com/dexoron/dcr/issues)** — your question may already be answered.
5. **Update DCR** — the problem may be fixed in the latest release (`dcr --update` or `dcrup update`). See the [changelog](CHANGELOG.md).

## Where to Get Help

| I want to… | Go to |
|------------|-------|
| Report a bug | [Open an issue](https://github.com/dexoron/dcr/issues/new) |
| Request a feature or share an idea | [Open an issue](https://github.com/dexoron/dcr/issues/new) |
| Ask a usage question | [Open an issue](https://github.com/dexoron/dcr/issues/new) (type: question/help) |
| Report a security vulnerability | **Do not use public issues.** See [SECURITY.md](SECURITY.md) |
| Contribute code, docs, or translations | [CONTRIBUTING.md](CONTRIBUTING.md) |
| Report a problem with the website or docs site | [dexoron/dcr-site](https://github.com/dexoron/dcr-site) issues |
| Contact the maintainer directly | Telegram `@dexoron`, Discord `dexoron`, or [main@dexoron.tech](mailto:main@dexoron.tech) |

Issues can be written in **English or Russian**.

## How to Report a Bug

A good report gets a faster fix. Please include:

1. DCR version — output of `dcr --version`.
2. OS, architecture, and compiler/toolchain versions (for example `clang --version`).
3. How DCR was installed (`dcrup`, AUR, Homebrew, Nix, Cargo, from source).
4. Steps to reproduce.
5. Expected behavior.
6. Actual behavior and full error output (run with `dcr build --verbose` or `DCR_DEBUG=1` if useful; `--output plain` helps with readable logs).
7. A minimal project or `dcr.toml` that reproduces the problem.

## Installation Problems

- Prebuilt binaries are fetched through mirrors in the order **cf → gh → sf → ya**. If downloads fail, try a specific mirror: `dcrup install stable --mirror gh`.
- Behind a corporate proxy on Windows, set `DCRUP_PROXY`; to avoid GitHub API rate limits set `GITHUB_TOKEN`.
- If no prebuilt binary exists for your platform, build from source: `cargo build --release --features archive`.
- See [Installation](docs/getting-started/installation.mdx) and [Environment variables](docs/reference/environment-variables.mdx).

## Supported Versions and Platforms

Support is provided for the **latest stable release**. Older versions are not supported; please update first. Supported platforms are listed in the [README](README.md#platforms-pre-built-binary).

## Project Status

DCR is developed mainly by one maintainer, so replies are best effort and may take a few days. Please be patient and polite. If DCR is useful to you, you can support the project by starring the repository, voting for the AUR package, improving documentation and translations, or via the donation link on the [website](https://dcr-tool.ru).
