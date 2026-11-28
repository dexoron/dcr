# Security Policy

Thank you for helping keep DCR and its users safe.

## Supported Versions

DCR is under active development. Security fixes are released only for the latest stable version.

| Version | Supported |
|---------|-----------|
| 0.9.x (latest stable) | ✅ |
| `dev` / `night` channels | ⚠️ best effort, fixes land in the next release |
| < 0.9.0 | ❌ |

Please update before reporting (`dcr --update` or `dcrup update`) and check that the issue still reproduces on the latest release.

## Reporting a Vulnerability

**Do not open a public issue for security problems.**

Report privately using one of these channels:

1. **GitHub private vulnerability reporting** — the *Security* tab of [dexoron/dcr](https://github.com/dexoron/dcr) → *Report a vulnerability*.
2. **Email** — [main@dexoron.tech](mailto:main@dexoron.tech) with the subject `[SECURITY] dcr: <short summary>`.
3. **Messengers** (only to ask for a secure channel, without technical details) — Telegram `@dexoron`, Discord `dexoron`.

Please include:

- DCR version (`dcr --version`) and, if relevant, the `dcrup` version (`dcrup show`).
- OS, architecture, and toolchain (compiler, linker, `git`, `pkg-config`).
- A clear description of the issue and its impact.
- Step-by-step reproduction, ideally a minimal project or `dcr.toml`.
- Any proof-of-concept, logs, or suggested fix.

## What to Expect

DCR is maintained by a small team, so timelines are best effort:

| Stage | Target |
|-------|--------|
| Acknowledgement of your report | within 7 days |
| Initial assessment and severity | within 14 days |
| Fix or mitigation for confirmed issues | as soon as practical, depending on severity |

We will keep you informed about progress, coordinate a disclosure date with you, and credit you in the release notes and changelog unless you prefer to stay anonymous. Please give us reasonable time to release a fix before any public disclosure.

## Scope

In scope:

- The `dcr` binary (all commands, including `dcr --update`).
- The installers `dcrup.sh` and `dcrup.ps1`, including mirror download, checksum verification, and PATH handling.
- Dependency handling: `path`, `git`, registry, and `pkg-config` sources, `dcr.lock`.
- Generated files (`.dcr/`, `.vscode/`, `.idea/`, `compile_commands.json`).
- Release assets and distribution mirrors (`cf`, `gh`, `sf`, `ya`).

Out of scope:

- **Behavior that is intentional.** `dcr.toml` is trusted input: `build.steps`, `build.post_steps`, `[run].cmd`, `[toolchain]`, and `ldscript` execute commands or tools by design. Running `dcr build` or `dcr run` in an untrusted project is equivalent to running its scripts. Review untrusted projects and dependencies first.
- Vulnerabilities in third-party tools invoked by DCR (compilers, linkers, `git`, `pkg-config`, `clang-tidy`, `clang-format`, `nasm`, Qt tools) or in libraries you build with DCR.
- Issues in projects built with DCR (the GPL applies only to DCR itself).
- Social engineering, denial of service by resource exhaustion on your own machine, and reports without a demonstrable security impact.
- Problems requiring physical access or an already compromised system.

## Safe Usage Tips

- Install DCR only from official sources: the [GitHub releases](https://github.com/dexoron/dcr/releases), `dcrup`, or the package managers listed in the README.
- `dcrup` verifies SHA256 digests when GitHub publishes them for an asset; prefer pinned versions in CI (`dcrup install 0.9.6`).
- Pin git dependencies with `tag` or `rev`; the exact commit is recorded in `dcr.lock`.
- Do not run `dcr build` on code you do not trust without sandboxing it.

## Disclosure Policy

We follow coordinated disclosure. After a fix is released, the issue is described in the [changelog](CHANGELOG.md) and, where appropriate, in a GitHub Security Advisory.
