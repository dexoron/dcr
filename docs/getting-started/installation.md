# Installation

## Option 1: Install via script

### Linux/macOS

```sh
curl -fsSL https://dcr.dexoron.su/install.sh | bash
```

Alternative with `wget`:

```sh
wget -qO- https://dcr.dexoron.su/install.sh | bash
```

### Windows (PowerShell)

```powershell
powershell -ExecutionPolicy ByPass -c "irm https://dcr.dexoron.su/install.ps1 | iex"
```

## Option 2: Build from source

```sh
git clone https://github.com/dexoron/dcr.git
cd dcr
cargo build --release
mkdir -p ~/.local/bin
ln -sf "$PWD/target/release/dcr" ~/.local/bin/dcr
```

## Option 3: Arch Linux (AUR)

```sh
yay -S dcr
```

(You can also use `paru` or other AUR helpers.)

## Verify installation

```sh
dcr --version
```

Expected format:

```text
dcr <version> (<target>)
```

## Update policy

- If DCR was installed from GitHub releases, scripts, or manual build: use `dcr --update`.
- If DCR is managed by `pacman/AUR`: update via package manager. On Linux, `dcr --update` detects pacman-owned binaries and prints a package-manager hint.
