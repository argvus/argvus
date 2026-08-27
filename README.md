<!-- markdownlint-disable MD033 -->
<!-- markdownlint-disable MD041 -->

<div align="center">
  <img src="https://raw.githubusercontent.com/argvus/argvus-logo/refs/heads/main/svg/argvus-banner.svg" width="540">
</div>

<div align="center">

**A complete desktop, ready to use. Focused on Hyprland.**

ARGVUS bundles a rich, integrated Wayland environment: bar, launchers, system sidebar, Rust-powered removable storage and eight theme families — all packaged for Arch Linux.

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
- **argvus-storage** — Rust-powered removable storage module

## Themes

Eight theme families with a shared accent-color system that unifies GTK, terminals, rofi, Waybar and Hyprland. Change the accent color anytime with `SUPER + SHIFT + A`.

| Dark          | Float               | Light            |
|---------------|---------------------|------------------|
| Dark Aether   | Dark Aether Float   | Light Veil       |
| Dark Silver   | Dark Silver Float   | Light Veil Float |
| Dark Slate    | Dark Slate Float    |                  |
| Dark Universe | Dark Universe Float |                  |

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
| argvus-storage | [argvus/argvus-storage](https://github.com/argvus/argvus-storage) | [![Release](https://github.com/argvus/argvus-storage/actions/workflows/release.yml/badge.svg)](https://github.com/argvus/argvus-storage/actions/workflows/release.yml) |
| argvus-calendar | [argvus/argvus-calendar](https://github.com/argvus/argvus-calendar) | [![Release](https://github.com/argvus/argvus-calendar/actions/workflows/release.yml/badge.svg)](https://github.com/argvus/argvus-calendar/actions/workflows/release.yml) |
| argvus-greeter | [argvus/argvus-greeter](https://github.com/argvus/argvus-greeter) | [![Release](https://github.com/argvus/argvus-greeter/actions/workflows/release.yml/badge.svg)](https://github.com/argvus/argvus-greeter/actions/workflows/release.yml) |
| argvus-appearance | [argvus/argvus-appearance](https://github.com/argvus/argvus-appearance) | [![Release](https://github.com/argvus/argvus-appearance/actions/workflows/release.yml/badge.svg)](https://github.com/argvus/argvus-appearance/actions/workflows/release.yml) |
| argvus-waybar | [argvus/argvus-waybar](https://github.com/argvus/argvus-waybar) | [![Release](https://github.com/argvus/argvus-waybar/actions/workflows/release.yml/badge.svg)](https://github.com/argvus/argvus-waybar/actions/workflows/release.yml) |
| argvus-session | [argvus/argvus-session](https://github.com/argvus/argvus-session) | [![Release](https://github.com/argvus/argvus-session/actions/workflows/release.yml/badge.svg)](https://github.com/argvus/argvus-session/actions/workflows/release.yml) |
| argvus-splash | [argvus/argvus-splash](https://github.com/argvus/argvus-splash) | [![Release](https://github.com/argvus/argvus-splash/actions/workflows/release.yml/badge.svg)](https://github.com/argvus/argvus-splash/actions/workflows/release.yml) |

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

> The links above will be replaced by the respective [Stripe](https://stripe.com) payment links.

---

<div align="center">

Licensed under [GPL-3.0](./LICENSE)

</div>
