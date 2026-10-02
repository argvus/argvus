---
title: Install ARGVUS
description: Install, update, remove, and configure ARGVUS.
---

ARGVUS is distributed as an Arch Linux package through the official ARGVUS package repository.

## Quick install

On a fresh Arch Linux installation, you can install ARGVUS directly from a TTY without installing another desktop environment first.

Run:

```sh
curl -fsSL https://argvus.github.io/install.sh | sh
```

The installer:

1. Imports and locally signs the ARGVUS repository signing key.
2. Configures the ARGVUS package repository.
3. Updates the package database.
4. Installs the `argvus` package and its dependencies.

After installation, log out and select **ARGVUS** from your display manager.

> **Note:** The quick installer requires `curl`, `sudo`, and the standard Arch Linux package-management tools.

## Configure the repository only

To configure the ARGVUS package repository without installing the desktop, run:

```sh
curl -fsSL https://argvus.github.io/repo-install.sh | sh
```

Install ARGVUS manually afterward:

```sh
sudo pacman -Syu argvus
```

## Manual installation

If you prefer to configure the repository step by step, use the following commands.

### 1. Download the repository signing key

```sh
curl -fsSLo /tmp/argvus.gpg https://argvus.github.io/packages/arch/argvus.gpg
```

### 2. Import the signing key

```sh
sudo pacman-key --add /tmp/argvus.gpg
```

### 3. Locally sign the key

```sh
ARGVUS_KEY="$(gpg --show-keys --with-colons /tmp/argvus.gpg | grep '^pub:' | head -n1 | cut -d: -f5)"
sudo pacman-key --lsign-key "$ARGVUS_KEY"
```

### 4. Configure the ARGVUS repository

```sh
curl -fsSL https://argvus.github.io/packages/arch/argvus.conf \
  | sudo tee /etc/pacman.d/argvus.conf
```

Add the repository configuration to `/etc/pacman.conf`:

```sh
echo "Include = /etc/pacman.d/argvus.conf" \
  | sudo tee -a /etc/pacman.conf
```

The repository configuration enables the ARGVUS repositories, including `argvus` and `argvus-extras`, with `SigLevel = Required TrustedOnly`. Pacman accepts packages only when their signatures are valid and trusted by the local keyring.

### 5. Install ARGVUS

```sh
sudo pacman -Syu argvus
```

After installation, log out and select **ARGVUS** from your display manager.

## Update

Update ARGVUS together with the rest of Arch Linux:

```sh
sudo pacman -Syu
```

To update only the ARGVUS package:

```sh
sudo pacman -S argvus
```

> **Recommendation:** Arch Linux generally recommends a full system upgrade with `sudo pacman -Syu` instead of updating individual packages.

## Remove ARGVUS

Remove ARGVUS while keeping dependencies that are still required by other installed packages:

```sh
sudo pacman -R argvus
```

Remove ARGVUS and dependencies that are no longer required by any installed package:

```sh
sudo pacman -Rns argvus
```

### Remove the ARGVUS repository

If you also want to remove the package repository, remove its configuration:

```sh
sudo rm /etc/pacman.d/argvus.conf
```

Then remove the corresponding line from `/etc/pacman.conf`:

```text
Include = /etc/pacman.d/argvus.conf
```

You can edit the file with:

```sh
sudo nano /etc/pacman.conf
```

### Remove the signing key

To remove the ARGVUS repository signing key from the pacman keyring, first list the keys:

```sh
sudo pacman-key --list
```

Then remove the ARGVUS key with its key ID:

```sh
sudo pacman-key --delete KEY_ID
```

Replace `KEY_ID` with the ARGVUS signing-key ID.

> **Warning:** Remove this key only when you are also removing the ARGVUS repository. Do not remove keys still required by other repositories.

## User configuration

Package installation does not write configuration into `$HOME`. ARGVUS reads packaged defaults from `/usr/share/argvus` and works without copying them into your home directory.

Runtime configuration follows this general priority:

```text
~/.config/argvus/data/generated/<app>  ->  ~/.config/argvus/data/<component>/<app>  ->  $XDG_CONFIG_HOME/<app>  ->  /usr/share/argvus/<app>  ->  upstream defaults
```

`argvus-config` is the only component that writes `~/.config/argvus/data/generated/`, and that projection is the active layer: a managed or native copy only takes effect while the corresponding generated file is absent.

ARGVUS-managed configuration lives under `~/.config/argvus/`. An optional `$XDG_CONFIG_HOME/<app>` override remains available for applications that support a native user configuration.

The `argvus --setup` dispatcher is optional and is for explicit manual
overrides, not recovery. If `~/.config/argvus` is deleted, `SUPER + SHIFT + R`
recreates the canonical profile and generated runtime state through the normal
reload pipeline.

```sh
argvus --setup --copy foot-tui
argvus --setup --copy kitty-tui
argvus --setup --copy-all
```

`--copy-all` is an explicit allowlist of the supported TUI profile files; it
does not clone `/usr/share/argvus` and does not copy scripts, services, assets,
generated files or `config.json`. Run `argvus --setup --help` to see the current
options. Once copied, the files become user-owned overrides and package
upgrades do not replace them.

To replace an existing copied configuration, use `--force`. ARGVUS creates a timestamped backup before replacing it:

```sh
argvus --setup --copy <app> --force
```

## Hyprland user overrides

For the complete Hyprland override guide, including the difference between a full native configuration and incremental ARGVUS fragments, see [Hyprland overrides](/docs/argvus-hyprland/hyprland-overrides/).

```sh
mkdir -p ~/.config/argvus/data/hypr
nano ~/.config/argvus/data/hypr/user.lua
```

### Virtual-machine fallback

Some virtual machines can experience rendering problems with Hyprland, Kitty, or Quickshell. ARGVUS detects virtual machines using the `vmwgfx` graphics driver and enables software rendering when necessary.

If detection does not work on your virtual machine, enable the fallback manually in `user.lua`:

```lua
-- Virtual machine compatibility
hl.env("LIBGL_ALWAYS_SOFTWARE", "1")
```

After creating or modifying `user.lua`, log out of Hyprland and log in again so the variable is applied to applications launched by the session.

Verify it with:

```sh
echo $LIBGL_ALWAYS_SOFTWARE
```

The expected output is:

```text
1
```

> **Note:** `LIBGL_ALWAYS_SOFTWARE=1` forces Mesa/OpenGL applications to use software rendering, usually through `llvmpipe`. It can reduce graphical performance and should be used as a fallback for affected virtual machines.
