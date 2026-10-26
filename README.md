<!-- markdownlint-disable MD033 -->
<!-- markdownlint-disable MD041 -->

<div align="center">
  <img src="https://raw.githubusercontent.com/argvus/argvus-logo/refs/heads/main/svg/ARGVUS-banner.svg" width="540">
</div>

<div align="center">

[![CI](https://github.com/argvus/argvus/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/argvus/argvus/actions/workflows/ci.yml)
[![License: GPL-3.0-only](https://img.shields.io/github/license/argvus/argvus)](./LICENSE)
[![Rust 1.95+](https://img.shields.io/badge/Rust-1.95%2B-orange?logo=rust)](https://www.rust-lang.org/)
[![Platform: Arch Linux | Wayland](https://img.shields.io/badge/platform-Arch%20Linux%20%7C%20Wayland-1793D1?logo=archlinux&logoColor=white)](https://archlinux.org/)
[![Compositor: Hyprland 0.56.2](https://img.shields.io/badge/compositor-Hyprland%200.56.2-58E1FF)](https://hyprland.org/)
[![Pull requests](https://img.shields.io/github/issues-pr/argvus/argvus)](https://github.com/argvus/argvus/pulls)
[![Issues](https://img.shields.io/github/issues/argvus/argvus)](https://github.com/argvus/argvus/issues)
[![Last commit](https://img.shields.io/github/last-commit/argvus/argvus)](https://github.com/argvus/argvus/commits/main)
[![Docs](https://img.shields.io/badge/docs-argvus.github.io-blue)](https://argvus.github.io/docs/intro/)

</div>

<div align="center">

**A modular desktop environment for Wayland and Arch Linux, built on Hyprland.**

ARGVUS coordinates a modular Wayland desktop ecosystem: session lifecycle,
Quickshell control panel, taskbar, launcher, lock screen, settings, storage,
power, display, network, notifications and themes — all packaged for Arch Linux.

</div>

---

## Features

- **Session** — Login through `greetd` with the ARGVUS greeter, session startup and systemd user services
- **Hyprland** — Wayland compositor configured in Lua, with tiling, workspaces, monitors and keybindings
- **Taskbar** — Waybar-based taskbar with the ARGVUS patched Waybar binary
- **Control panel** — Quickshell panel for quick system controls and widgets
- **Control Center** — Terminal settings application for appearance, apps, displays, locale, users and more
- **Launcher** — Application launcher with ARGVUS themes
- **Lock screen** — Hyprlock-based lock screen synchronized with the active theme
- **Notifications, power, display, network** — Desktop providers with their own packaged configuration
- **Removable devices** — UDisks2-backed storage actions in the taskbar and menus
- **Terminal and TUI apps** — Terminal profiles, system monitor and app profiles for bundled TUI tools
- **Themes** — 23 official theme families, each in the Sticky and Float layouts
- **Games** — Optional. Snake for the terminal (`argvus-game-snake`, an independent project), with a local high score list
- **Languages** — English (en-US, canonical) and Portuguese (pt-BR), managed by `argvus i18n`

## Modular Ecosystem

`argvus` is the master package and the only package built from this repository. It installs the
command, its in-process subcommands and the shared runtime, the Control Center (a terminal
application), the GTK 4 helpers for the taskbar calendar, removable devices and the loading
overlay, and the login greeter (built from `crates/argvus-greeter`). Every component package
depends on it, so no install order is required.

| Area | Packages and commands |
|------|-----------------------|
| Session lifecycle, login and startup | `argvus-session`, `argvus greeter` (in `argvus`), `argvus-boot-splash` |
| Hyprland configuration and scripts | `argvus-hyprland` |
| Quickshell control panel | `argvus-control-panel` |
| Taskbar, Waybar and widgets | `argvus-taskbar`, `argvus-waybar`, `argvus widget-telemetry` |
| Settings application | `argvus control-center` (in `argvus`) |
| Launcher and lock screen | `argvus-launcher`, `argvus-lock` |
| Appearance, wallpapers, fonts and icons | `argvus-appearance`, `argvus-wallpapers`, `argvus-fonts`, `argvus-icons`, `argvus-branding` |
| Themes | `argvus-theme-*` (one package per official theme) |
| Terminal, TUI apps and system monitor | `argvus terminal`, `argvus tui-terminal`, `argvus system-monitor`, `argvus-app-profiles` (data) |
| Display, power, network and notifications | `argvus-display`, `argvus-power`, `argvus network`, `argvus-notifications` |
| Desktop portals and Wayland/DBus defaults | `argvus-portal` |
| Removable storage | `argvus removable-devices` (in `argvus`) |
| Accounts | `argvus accounts` (runs as root through `pkexec`) |
| Firewall | `argvus-firewall` |
| Games | `argvus-games`; `argvus-game-snake` is independent and optional |
| Configuration, translations and keys | `argvus config`, `argvus i18n` (with the language catalogs), `argvus keys` |

The `argvus` command routes to these subcommands. For example, `argvus control-center
about` opens the About page of the Control Center.

Source-tree `tools/sh/install.sh` only installs the `argvus` command for local
testing. Runtime files are installed by the packages above.

## Command

```sh
argvus --help                 # full list of routes
argvus --control-center       # settings application
argvus --system-monitor       # btop-based system monitor
argvus --spf                  # superfile, with ARGVUS config
argvus --yazi                 # yazi, with ARGVUS config
argvus --setup --copy <app>   # copy packaged defaults for explicit customization
```

## Themes

Twenty-three theme families, each available in the Sticky and Float layouts,
with a shared accent-color system that unifies GTK, terminals, rofi, Waybar and
Hyprland. Change the accent color from the Highlight color control in the Control Panel or Control Center.

| Dark                  | Light              |
|-----------------------|--------------------|
| ARGVUS Dark           | ARGVUS Light       |
| Dracula               | Catppuccin Latte   |
| Gruvbox Dark          | Everforest Light   |
| Gruvbox High Dark     |                    |
| Matrix             | GitHub Light       |
| Monokai Dark          | Gruvbox Light      |
| One Dark              | One Light          |
| Rosé Pine             | Solarized Light    |
| Silver Dark           | Watercolor         |
|                       | Nord Light         |
| Slate Dark            |                    |
| Solitude              |                    |
| Sunset                |                    |
| Tokyo Night           |                    |
| Universe              |                    |

Float is a layout applied to any family, not a separate copy of the theme.
ARGVUS Dark and ARGVUS Light ship in `argvus-appearance`; every other family comes from its
own `argvus-theme-*` package. These packages are optional and can be installed individually,
for example `pacman -S argvus-theme-sunset` or `pacman -S argvus-theme-watercolor`.

## Install

```sh
# Import the GPG key
curl -fsSLo /tmp/argvus.gpg https://argvus.github.io/packages/arch/argvus.gpg
sudo pacman-key --add /tmp/argvus.gpg
ARGVUS_KEY="$(gpg --show-keys --with-colons /tmp/argvus.gpg | grep '^pub:' | head -n1 | cut -d: -f5)"
sudo pacman-key --lsign-key "$ARGVUS_KEY"

# Add the repository
curl -fsSL https://argvus.github.io/packages/arch/argvus.conf \
  | sudo tee /etc/pacman.d/argvus.conf
echo "Include = /etc/pacman.d/argvus.conf" \
  | sudo tee -a /etc/pacman.conf

# Install
sudo pacman -Syu argvus
```

Optional packages:

```sh
# All official themes
pacman -Ssq '^argvus-theme-' | sudo xargs pacman -S --needed

# Optional applications and utilities
sudo pacman -S blueman starship kooha nwg-look
```

## Links

- Organization: [@argvus](https://github.com/argvus/)
- Official page: [argvus.github.io](https://argvus.github.io/)
- Documentation: [argvus.github.io/docs/](https://argvus.github.io/docs/intro/)
- Repository documentation: [docs/](./docs/)
- Development workflow: [DEVELOPMENT.md](./docs/DEVELOPMENT.md)
- Contributing: [CONTRIBUTING.md](https://github.com/argvus/.github/blob/main/CONTRIBUTING.md)
- Security policy: [SECURITY.md](https://github.com/argvus/.github/blob/main/SECURITY.md)
- Code of Conduct: [CODE_OF_CONDUCT.md](https://github.com/argvus/.github/blob/main/policies/CODE_OF_CONDUCT.md)
- AI usage policy: [IA_USAGE.md](https://github.com/argvus/.github/blob/main/policies/IA_USAGE.md)
- Ecosystem workflow: [argvus/workflow](https://github.com/argvus/workflow)

## Related repositories

| Package | Repository |
|---------|------------|
| argvus-app-profiles | [argvus/argvus-app-profiles](https://github.com/argvus/argvus-app-profiles) |
| argvus-appearance | [argvus/argvus-appearance](https://github.com/argvus/argvus-appearance) |
| argvus-boot-splash | [argvus/argvus-boot-splash](https://github.com/argvus/argvus-boot-splash) |
| argvus-branding | [argvus/argvus-branding](https://github.com/argvus/argvus-branding) |
| argvus-control-panel | [argvus/argvus-control-panel](https://github.com/argvus/argvus-control-panel) |
| argvus-display | [argvus/argvus-display](https://github.com/argvus/argvus-display) |
| argvus-firewall | [argvus/argvus-firewall](https://github.com/argvus/argvus-firewall) |
| argvus-fonts | [argvus/argvus-fonts](https://github.com/argvus/argvus-fonts) |
| argvus-game-snake | [argvus/argvus-game-snake](https://github.com/argvus/argvus-game-snake) |
| argvus-games | [argvus/argvus-games](https://github.com/argvus/argvus-games) |
| argvus-hyprland | [argvus/argvus-hyprland](https://github.com/argvus/argvus-hyprland) |
| argvus-icons | [argvus/argvus-icons](https://github.com/argvus/argvus-icons) |
| argvus-launcher | [argvus/argvus-launcher](https://github.com/argvus/argvus-launcher) |
| argvus-lock | [argvus/argvus-lock](https://github.com/argvus/argvus-lock) |
| argvus-notifications | [argvus/argvus-notifications](https://github.com/argvus/argvus-notifications) |
| argvus-portal | [argvus/argvus-portal](https://github.com/argvus/argvus-portal) |
| argvus-power | [argvus/argvus-power](https://github.com/argvus/argvus-power) |
| argvus-session | [argvus/argvus-session](https://github.com/argvus/argvus-session) |
| argvus-taskbar | [argvus/argvus-taskbar](https://github.com/argvus/argvus-taskbar) |
| argvus-wallpapers | [argvus/argvus-wallpapers](https://github.com/argvus/argvus-wallpapers) |
| argvus-waybar | [argvus/argvus-waybar](https://github.com/argvus/argvus-waybar) |
| argvus-theme-*, for example | [argvus/argvus-theme-dracula](https://github.com/argvus/argvus-theme-dracula) |

The Rust workspace, with the `argvus` command, the Control Center, the greeter source,
the configuration model and translations, lives in this repository (`crates/`).

## Donate to the development of ARGVUS

If [ARGVUS](https://argvus.github.io) is useful to you, please consider supporting the project's development. Your contribution helps maintain the infrastructure and supports the project's ongoing maintenance and evolution—including the costs of eventually acquiring and maintaining a dedicated domain for ARGVUS.

[CLICK HERE TO DONATE](https://argvus.github.io/#support)

<!-- | Contribution | Support |
|:---:|:---:|
| **US$ 1** | [Contribute](#) |
| **US$ 5** | [Contribute](#) |
| **US$ 10** | [Contribute](#) |
| **US$ 20** | [Contribute](#) |
| **US$ 50** | [Contribute](#) | -->

> The links above will be replaced by the respective [PayPal](https://paypal.com) payment links.

---

<div align="center">

Licensed under [GPL-3.0](./LICENSE)

</div>
