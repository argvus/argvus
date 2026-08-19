---
title: Installation
description: Install, update and remove ARGVUS, and customize user configs.
---

Packages are published to the public [argvus/packages](https://github.com/argvus/packages) repository, served at `https://argvus.github.io/packages/`.

The `argvus` package and its dependencies are published to the same repository, so plain `pacman` resolves everything automatically. This includes the vendored packages (`pwvucontrol`, `snappy-switcher` and `wlogout`) and the official component packages `argvus-appearance`, `argvus-calendar`, `argvus-greeter`, `argvus-session` and `argvus-storage`.

## Install

Add the repository and install:

```sh
# Trust the ARGVUS signing key
curl -fsSLo /tmp/argvus.gpg https://argvus.github.io/packages/arch/argvus.gpg
sudo pacman-key --add /tmp/argvus.gpg
ARGVUS_KEY="$(gpg --show-keys --with-colons /tmp/argvus.gpg | grep '^pub:' | head -n1 | cut -d: -f5)"
sudo pacman-key --lsign-key "$ARGVUS_KEY"

# Add repository
curl -fsSL https://argvus.github.io/packages/arch/argvus.conf \
  | sudo tee /etc/pacman.d/argvus.conf
echo "Include = /etc/pacman.d/argvus.conf" \
  | sudo tee -a /etc/pacman.conf

# Install
sudo pacman -Syu argvus
```

## Update

```sh
# Refresh repository configuration
curl -fsSL https://argvus.github.io/packages/arch/argvus.conf \
  | sudo tee /etc/pacman.d/argvus.conf

# Update package
sudo pacman -Syu argvus
```

## Remove

Remove the package:

```sh
sudo pacman -Rns argvus
```

Remove the repository entry:

```sh
sudo sed -i '\|^Include = /etc/pacman.d/argvus.conf$|d' /etc/pacman.conf
sudo rm -f /etc/pacman.d/argvus.conf
sudo pacman -Syy
```

## User configuration

Package installation never writes to `$HOME`. ARGVUS works out of the box by reading packaged defaults from `/usr/share/argvus`.

Runtime configuration uses this priority:

```text
$XDG_CONFIG_HOME/<app>  ->  $XDG_STATE_HOME/argvus/config/<app>  ->  /usr/share/argvus/<app>  ->  upstream defaults
```

If `XDG_STATE_HOME` is unset, the fallback is `~/.local/state`. Theme, accent and spacing tools generate runtime application config under `~/.local/state/argvus/config` instead of copying complete application config trees to `$HOME`.

Small ARGVUS preferences live under `$XDG_CONFIG_HOME/argvus`, and explicit complete-application overrides live under `$XDG_CONFIG_HOME/<app>`.

`argvus-setup` is optional. Use it only when you want to copy packaged defaults into `$XDG_CONFIG_HOME` for customization:

```sh
argvus-setup --copy hypr
argvus-setup --copy waybar
argvus-setup --copy-all
```

Once copied, files become user-owned overrides and package upgrades never replace them.

To refresh an existing config with the current package defaults, run:

```sh
argvus-setup --copy <app> --force
```

The `--force` mode backs up the current directory as `$XDG_CONFIG_HOME/<app>.bak-<timestamp>` before copying the new defaults.
