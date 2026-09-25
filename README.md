<!-- markdownlint-disable MD033 -->
<!-- markdownlint-disable MD041 -->

<div align="center">
  <img src="https://raw.githubusercontent.com/argvus/argvus-logo/refs/heads/main/svg/argvus-banner.svg" width="540">
</div>

<div align="center">

**A modular desktop environment for Wayland and Arch Linux, built on Hyprland.**

ARGVUS coordinates a modular Wayland desktop ecosystem: session lifecycle,
shell UI, taskbar, launchers, storage, lock screen, power, display, network,
notifications and themes — all packaged for Arch Linux.

</div>

---

## Features

- **Hyprland** — Wayland compositor with tiling, workspaces and smooth animations
- **Waybar** — Status bar with system, media, network and storage modules
- **Quickshell** — QML system sidebar with weather, calendar, brightness and notifications
- **Rofi** — Application launcher and menus with unified themes
- **superfile** — Fast TUI file manager
- **btop** — System monitor
- **hyprlock** — Lock screen with synchronized theme
- **argvus-removable-devices** — Rust-powered removable storage module

## Modular Ecosystem

`argvus` is the full desktop package/coordinator. Component ownership is
split across smaller packages:

| Area | Package |
|------|---------|
| Session lifecycle, shared bootstrap and session services | `argvus-session` |
| Hyprland configuration and Hyprland-specific scripts | `argvus-hyprland` |
| Quickshell control panel, ARGVUS Waybar taskbar/sysinfo, rofi/wofi shell UI | `argvus-shell` |
| Patched Waybar package/binary | `argvus-waybar` |
| Terminal/TUI app profiles and the `argvus` compatibility command | `argvus-app-profiles` |
| Appearance, wallpapers, fonts and themes | `argvus-appearance` |
| Notifications and Dunst config | `argvus-notifications` |
| Power menu, idle and DPMS policy | `argvus-power` |
| Display/monitor integration | `argvus-display` |
| NetworkManager and Bluetooth commands | `argvus-network` |
| Hyprlock config and lock screen themes | `argvus-lock` |
| Wayland, DBus and portal defaults | `argvus-portal` |
| Accounts, settings, about, calendar, storage and greeter | `argvus-accounts`, `argvus-settings`, built-in `argvus-about`, `argvus-calendar`, `argvus-removable-devices`, `argvus-greeter` |

`argvus` itself remains the install target for users, coordinates the complete
package set, and ships the `argvus-about` GTK application.

Source-tree `make install` in this repository keeps local checks focused on
the coordinator package. Runtime files are installed by the module packages
listed above.

## Themes

Twenty theme families with a shared accent-color system that unifies GTK, terminals, rofi, Waybar and Hyprland. Change the accent color from the Highlight color control in the Control Panel or Control Center.

| Dark             | Float                    | Light             |
|------------------|--------------------------|-------------------|
| ARGVUS Dark      | ARGVUS Dark Float        | ARGVUS Light      |
| Dracula          | Dracula Float            | Catppuccin Latte  |
| Gruvbox Dark     | Gruvbox Dark Float       | Frost             |
| Gruvbox High Dark| Gruvbox High Dark Float  | GitHub Light      |
| Monokai Dark     | Monokai Dark Float       | Gruvbox Light     |
| One Dark         | One Dark Float           | Solarized Light   |
| Rosé Pine        | Rosé Pine Float          |                   |
| Silver Dark      | Silver Dark Float        |                   |
| Slate Dark       | Slate Dark Float         |                   |
| Sunset           | Sunset Float             |                   |
| Tokyo-Night      | Tokyo-Night Float        |                   |
| Hackerman        | Hackerman Float          |                   |
| Solitude         | Solitude Float           |                   |
| Universe         | Universe Float           |                   |

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

## Links

- Organization: [@argvus](https://github.com/argvus/)
- Official page: [argvus.github.io](https://argvus.github.io/)
- Documentation: [argvus.github.io/docs/](https://argvus.github.io/docs/intro/)
- Development workflow: [DEVELOPMENT.md](./DEVELOPMENT.md)
- Contribution guide: [CONTRIBUTING.md](./CONTRIBUTING.md)

## Related repositories

| Name | Repository | Status |
|------|------------|--------|
| argvus-removable-devices | [argvus/argvus-removable-devices](https://github.com/argvus/argvus-removable-devices) | [![Release](https://github.com/argvus/argvus-removable-devices/actions/workflows/release.yml/badge.svg)](https://github.com/argvus/argvus-removable-devices/actions/workflows/release.yml) |
| argvus-calendar | [argvus/argvus-calendar](https://github.com/argvus/argvus-calendar) | [![Release](https://github.com/argvus/argvus-calendar/actions/workflows/release.yml/badge.svg)](https://github.com/argvus/argvus-calendar/actions/workflows/release.yml) |
| argvus-greeter | [argvus/argvus-greeter](https://github.com/argvus/argvus-greeter) | [![Release](https://github.com/argvus/argvus-greeter/actions/workflows/release.yml/badge.svg)](https://github.com/argvus/argvus-greeter/actions/workflows/release.yml) |
| argvus-appearance | [argvus/argvus-appearance](https://github.com/argvus/argvus-appearance) | [![Release](https://github.com/argvus/argvus-appearance/actions/workflows/release.yml/badge.svg)](https://github.com/argvus/argvus-appearance/actions/workflows/release.yml) |
| argvus-waybar | [argvus/argvus-waybar](https://github.com/argvus/argvus-waybar) | [![Release](https://github.com/argvus/argvus-waybar/actions/workflows/release.yml/badge.svg)](https://github.com/argvus/argvus-waybar/actions/workflows/release.yml) |
| argvus-session | [argvus/argvus-session](https://github.com/argvus/argvus-session) | [![Release](https://github.com/argvus/argvus-session/actions/workflows/release.yml/badge.svg)](https://github.com/argvus/argvus-session/actions/workflows/release.yml) |
| argvus-hyprland | [argvus/argvus-hyprland](https://github.com/argvus/argvus-hyprland) | [![Release](https://github.com/argvus/argvus-hyprland/actions/workflows/release.yml/badge.svg)](https://github.com/argvus/argvus-hyprland/actions/workflows/release.yml) |
| argvus-splash | [argvus/argvus-splash](https://github.com/argvus/argvus-splash) | [![Release](https://github.com/argvus/argvus-splash/actions/workflows/release.yml/badge.svg)](https://github.com/argvus/argvus-splash/actions/workflows/release.yml) |
| argvus-app-profiles | [argvus/argvus-app-profiles](https://github.com/argvus/argvus-app-profiles) | Planned |
| argvus-shell | [argvus/argvus-shell](https://github.com/argvus/argvus-shell) | Planned |
| argvus-settings | [argvus/argvus-settings](https://github.com/argvus/argvus-settings) | Planned |
| argvus-notifications | [argvus/argvus-notifications](https://github.com/argvus/argvus-notifications) | Planned |
| argvus-power | [argvus/argvus-power](https://github.com/argvus/argvus-power) | Planned |
| argvus-display | [argvus/argvus-display](https://github.com/argvus/argvus-display) | Planned |
| argvus-network | [argvus/argvus-network](https://github.com/argvus/argvus-network) | Planned |
| argvus-lock | [argvus/argvus-lock](https://github.com/argvus/argvus-lock) | Planned |
| argvus-portal | [argvus/argvus-portal](https://github.com/argvus/argvus-portal) | Planned |

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
