# Contributing & Bug reports

## Contribution workflow

1. Fork `dexoron/dcr` (and `dexoron/dcr-site` if docs website changes are needed).
2. Make focused changes.
3. Run checks locally:

```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo check
```

4. Open a PR with short rationale and behavior impact.
5. Update docs when CLI behavior changes.

## Bug report checklist

Include:

1. DCR version (`dcr --version`).
2. OS and compiler/toolchain version.
3. Reproduction steps.
4. Expected result.
5. Actual result and full error output.
6. Minimal reproducible project.

## Where to report

Use GitHub Issues in the relevant repository:

- <https://github.com/dexoron/dcr>
- <https://github.com/dexoron/dcr-site>
