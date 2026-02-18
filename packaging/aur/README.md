# AUR publishing

This repository includes an automated AUR publish workflow:
- package files: `packaging/aur/dcr/PKGBUILD`, `packaging/aur/dcr/.SRCINFO`
- workflow: `.github/workflows/aur.yml`

## One-time setup

1. Create the `dcr` package on AUR and initialize its git repository.
2. Create an SSH key pair for CI.
3. Add the public key to your AUR account.
4. Add the private key to GitHub Actions secret `AUR_SSH_PRIVATE_KEY`.

## Release flow

1. Bump project version in `Cargo.toml`.
2. Push tag `v<version>` (example: `v0.2.2`).
3. GitHub Actions will:
- compute source tarball SHA256
- render `PKGBUILD` and `.SRCINFO`
- push updates to `ssh://aur@aur.archlinux.org/dcr.git`
