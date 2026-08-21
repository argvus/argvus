---
title: Introduction
description: What ARGVUS is and how it is put together.
slug: 0.1.1/docs/intro
---

ARGVUS is a complete, ready-to-use desktop environment focused on [Hyprland](https://hypr.land), packaged for [Arch Linux](https://archlinux.org).

It is not just a personal configuration: the project assembles a rich, integrated Wayland environment with a top [Waybar](https://github.com/Alexays/Waybar) bar, [Rofi](https://github.com/davatorium/rofi) and [Wofi](https://hg.sr.ht/~scoopta/wofi) launchers, a system information sidebar built with [Quickshell](https://quickshell.outfoxxed.de)/QML, a Rust-powered removable storage manager for Waybar, and a native calendar popup. Eight theme families — dark, dark silver, light and slate (plus their float variants) — unify GTK, terminal, rofi, dunst, waybar and Hyprland itself, all driven by a shared accent-color system.

The project is under development. Packages are published to the public [packages](https://github.com/argvus/packages) repository by the **Build and publish Arch packages** workflow. The desktop is split into focused component packages: `argvus` (configuration), `argvus-session` (session launchers), `argvus-appearance` (wallpapers and fonts), `argvus-storage`, `argvus-calendar` and `argvus-greeter`.

## Next steps

* [Installation](/0.1.1/docs/install/) — add the repository, install, update and remove the desktop.
* [Themes](/0.1.1/docs/themes/) — theme families and accent colors.
* [Sessions](/0.1.1/docs/sessions/gdm/) — start ARGVUS from GDM or a TTY.
* [Calendar](/0.1.1/docs/calendar/) — the native Waybar calendar popup.
* [Removable Storage](/0.1.1/docs/storage/) — the Waybar storage module.
* [Packaging](/0.1.1/docs/packaging/) — package layout and publishing.
