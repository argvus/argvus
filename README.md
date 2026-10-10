<!-- markdownlint-disable MD033 -->
<!-- markdownlint-disable MD041 -->

<div align="center">
  <img src="https://raw.githubusercontent.com/argvus/argvus-logo/refs/heads/main/svg/ARGVUS-banner.svg" width="540">
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
- **Themes** — 22 official theme families (44 variants with Light/Dark and Float styles)
- **Games** — Snake for the terminal, with a local high score list

## Modular Ecosystem

`argvus` is the full desktop package and coordinator. Component ownership is
split across smaller packages:

| Area | Packages |
|------|----------|
| Session lifecycle, login and startup | `argvus-session`, `argvus-greeter`, `argvus-boot-splash`, `argvus-loading-theme` |
| Hyprland configuration and scripts | `argvus-hyprland` |
| Quickshell control panel | `argvus-control-panel` |
| Taskbar, Waybar and widgets | `argvus-taskbar`, `argvus-waybar`, `argvus-taskbar-calendar`, `argvus-widget-telemetry` |
| Settings application (TUI) | `argvus-control-center` |
| Launcher and lock screen | `argvus-launcher`, `argvus-lock` |
| Appearance, wallpapers, fonts and icons | `argvus-appearance`, `argvus-wallpapers`, `argvus-fonts`, `argvus-icons`, `argvus-branding` |
| Themes | `argvus-themes`, `argvus-theme-*` |
| Terminal, TUI apps and system monitor | `argvus-terminal`, `argvus-app-profiles`, `argvus-system-monitor` |
| Display, power, network and notifications | `argvus-display`, `argvus-power`, `argvus-network`, `argvus-notifications` |
| Desktop portals and Wayland/DBus defaults | `argvus-portal` |
| Removable storage | `argvus-removable-devices` |
| Accounts | `argvus-accounts` |
| Firewall | `argvus-firewall` |
| Games | `argvus-games`, `argvus-game-snake` |
| Shared libraries | `argvus-config`, `argvus-i18n` (with `argvus-language-en-us` and `argvus-language-pt-br`), `argvus-tui` |

The `argvus` package itself provides the `argvus` command, the install target for
users, and the metapackage that pulls the desktop components listed in its
dependencies. The `argvus` command routes to the applications above; for example,
`argvus --about` opens the About page inside `argvus-control-center`.

Source-tree `tools/sh/install.sh` only installs the `argvus` command for local
testing. Runtime files are installed by the module packages above.

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

Twenty-two theme families, each available in a standard and a Float variant,
with a shared accent-color system that unifies GTK, terminals, rofi, Waybar and
Hyprland. Change the accent color from the Highlight color control in the Control Panel or Control Center.

| Dark                  | Light              |
|-----------------------|--------------------|
| ARGVUS Dark           | ARGVUS Light       |
| Dracula               | Catppuccin Latte   |
| Gruvbox Dark          | Everforest Light   |
| Gruvbox High Dark     | Frost              |
| Hackerman             | GitHub Light       |
| Monokai Dark          | Gruvbox Light      |
| One Dark              | One Light          |
| Rosé Pine             | Solarized Light    |
| Silver Dark           |                    |
| Slate Dark            |                    |
| Solitude              |                    |
| Sunset                |                    |
| Tokyo Night           |                    |
| Universe              |                    |

Every family above has a Float variant (for example `dracula-float`).
Official themes are defined in `argvus-appearance`. The `argvus-theme-*`
packages are optional and can be installed individually or together through
`argvus-themes`.

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
sudo pacman -S argvus-themes

# Optional applications and utilities
sudo pacman -S blueman starship kooha nwg-look
```

## Links

- Organization: [@argvus](https://github.com/argvus/)
- Official page: [argvus.github.io](https://argvus.github.io/)
- Documentation: [argvus.github.io/docs/](https://argvus.github.io/docs/intro/)
- Repository documentation: [docs/](./docs/)
- Development workflow: [DEVELOPMENT.md](./DEVELOPMENT.md)
- Contribution guide: [CONTRIBUTING.md](./CONTRIBUTING.md)
- Ecosystem workflow: [argvus/workflow](https://github.com/argvus/workflow)

## Related repositories

| Package | Repository |
|---------|------------|
| argvus-accounts | [argvus/argvus-accounts](https://github.com/argvus/argvus-accounts) |
| argvus-app-profiles | [argvus/argvus-app-profiles](https://github.com/argvus/argvus-app-profiles) |
| argvus-appearance | [argvus/argvus-appearance](https://github.com/argvus/argvus-appearance) |
| argvus-boot-splash | [argvus/argvus-boot-splash](https://github.com/argvus/argvus-boot-splash) |
| argvus-branding | [argvus/argvus-branding](https://github.com/argvus/argvus-branding) |
| argvus-config | [argvus/argvus-config](https://github.com/argvus/argvus-config) |
| argvus-control-center | [argvus/argvus-control-center](https://github.com/argvus/argvus-control-center) |
| argvus-control-panel | [argvus/argvus-control-panel](https://github.com/argvus/argvus-control-panel) |
| argvus-display | [argvus/argvus-display](https://github.com/argvus/argvus-display) |
| argvus-firewall | [argvus/argvus-firewall](https://github.com/argvus/argvus-firewall) |
| argvus-fonts | [argvus/argvus-fonts](https://github.com/argvus/argvus-fonts) |
| argvus-game-snake | [argvus/argvus-game-snake](https://github.com/argvus/argvus-game-snake) |
| argvus-games | [argvus/argvus-games](https://github.com/argvus/argvus-games) |
| argvus-greeter | [argvus/argvus-greeter](https://github.com/argvus/argvus-greeter) |
| argvus-hyprland | [argvus/argvus-hyprland](https://github.com/argvus/argvus-hyprland) |
| argvus-i18n | [argvus/argvus-i18n](https://github.com/argvus/argvus-i18n) |
| argvus-icons | [argvus/argvus-icons](https://github.com/argvus/argvus-icons) |
| argvus-launcher | [argvus/argvus-launcher](https://github.com/argvus/argvus-launcher) |
| argvus-loading-theme | [argvus/argvus-loading-theme](https://github.com/argvus/argvus-loading-theme) |
| argvus-lock | [argvus/argvus-lock](https://github.com/argvus/argvus-lock) |
| argvus-network | [argvus/argvus-network](https://github.com/argvus/argvus-network) |
| argvus-notifications | [argvus/argvus-notifications](https://github.com/argvus/argvus-notifications) |
| argvus-portal | [argvus/argvus-portal](https://github.com/argvus/argvus-portal) |
| argvus-power | [argvus/argvus-power](https://github.com/argvus/argvus-power) |
| argvus-removable-devices | [argvus/argvus-removable-devices](https://github.com/argvus/argvus-removable-devices) |
| argvus-session | [argvus/argvus-session](https://github.com/argvus/argvus-session) |
| argvus-system-monitor | [argvus/argvus-system-monitor](https://github.com/argvus/argvus-system-monitor) |
| argvus-taskbar | [argvus/argvus-taskbar](https://github.com/argvus/argvus-taskbar) |
| argvus-taskbar-calendar | [argvus/argvus-taskbar-calendar](https://github.com/argvus/argvus-taskbar-calendar) |
| argvus-terminal | [argvus/argvus-terminal](https://github.com/argvus/argvus-terminal) |
| argvus-tui | [argvus/argvus-tui](https://github.com/argvus/argvus-tui) |
| argvus-wallpapers | [argvus/argvus-wallpapers](https://github.com/argvus/argvus-wallpapers) |
| argvus-waybar | [argvus/argvus-waybar](https://github.com/argvus/argvus-waybar) |
| argvus-widget-telemetry | [argvus/argvus-widget-telemetry](https://github.com/argvus/argvus-widget-telemetry) |
| argvus-themes, argvus-theme-* | [argvus/argvus-themes](https://github.com/argvus/argvus-themes) and one repository per theme, for example [argvus/argvus-theme-dracula](https://github.com/argvus/argvus-theme-dracula) |

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

Licensed under [GPL-3.0-only](./LICENSE)

</div>
