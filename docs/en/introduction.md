---
title: Introduction
description: What ARGVUS is, how its desktop works and where its components fit.
---

ARGVUS is a modular Hyprland and Wayland desktop environment for Arch Linux. It is an integrated collection of packages rather than one monolithic application: the session, compositor, shell surfaces, settings applications, providers and visual assets work together through shared state and explicit runtime contracts.

This page is the map of the documentation. It explains the whole desktop at a high level; the linked pages contain the operational and implementation details.

## How ARGVUS works

The normal session follows this path:

```text
graphical login or TTY
        ↓
argvus-session → argvus-start
        ↓
Hyprland + argvus-hyprland
        ↓
argvus-session.target (systemd --user)
        ↓
taskbar · control panel · notifications · wallpaper · idle · clipboard
```

`argvus-session` owns session lifecycle and environment setup. It does not implement every desktop feature: each component keeps ownership of its own commands, configuration and service payload.

For installation and startup, see [Installation](./user-guide/installation/) and [Sessions](./user-guide/sessions/). For service ownership, see the developer [runtime lifecycle](./developer-guide/architecture/runtime-lifecycle/).

## Main layers

### Entry points and infrastructure

The `argvus` package provides the `/usr/bin/argvus` dispatcher and the main package entry point. `argvus-session` provides graphical and TTY entry points, the user target and `argvus-sessionctl`. Shared TUI and localization functionality comes from `argvus-tui` and `argvus-i18n`.

### Compositor and desktop shell

`argvus-hyprland` supplies the Hyprland configuration and keybindings. `argvus-waybar` supplies the patched Waybar binary, while `argvus-taskbar` owns taskbar configuration and actions. `argvus-control-panel` supplies the Quickshell panel, and `argvus-widget-telemetry` supplies optional system widgets.

See [Desktop](./user-guide/desktop/) for usage and [shell surfaces](./developer-guide/subsystems/shell-surfaces/) for implementation ownership.

### Settings and applications

`argvus-control-center` is the keyboard-first settings application. The calendar, system monitor, terminal, launcher and removable-device tools are separate applications integrated with the shell and dispatcher.

See [Applications](./user-guide/applications/) and the [command reference](./reference/command-line/).

### Providers and integrations

Display, network, Bluetooth, power, notifications, firewall and removable storage are provided by focused projects. The Control Center and Control Panel consume these providers; they do not replace them.

Login, lock and startup visuals are also separate: `argvus-greeter` handles greetd, `argvus-lock` handles Hyprlock, `argvus-theme-splash` handles the session-loading overlay and `argvus-splash` handles the Plymouth boot theme.

See [Hardware](./user-guide/hardware/), [Privacy and security](./user-guide/privacy-and-security/) and the developer [component reference](./developer-guide/component-reference/).

## Shared appearance state

Theme, accent, wallpaper, font and effects preferences are stored as logical state under `$XDG_CONFIG_HOME/argvus` (normally `~/.config/argvus`). `argvus-config` is the only component that writes `data/generated/`, where it projects consumer files for Hyprland, GTK, Qt6ct, Waybar, Quickshell, Rofi, Dunst, notifications, the lock screen, Yazi, Superfile and terminals. Appearance helpers commit the state change and then reconcile external applications; they never write the generated tree.

The generated directory is not the source of truth. See [Appearance](./user-guide/appearance/) and [configuration and state](./developer-guide/architecture/configuration-and-state/).

The visual system also controls layout. Sticky mode is compact and square; Float mode uses larger window gaps, rounded corners and wider margins around the taskbar and shell. The taskbar defaults to the top edge but supports top/bottom placement through the shared layout state. Effects control transparency, blur, shadows and animations across the compositor and desktop surfaces.

See the [theme table](./user-guide/appearance/) for the current families and colors, [effects](./user-guide/appearance/effects/) for the shared effects contract, and [taskbar](./user-guide/desktop/taskbar/) for bar and window placement.

## What ARGVUS is—and is not

ARGVUS is the desktop environment layer on top of Arch Linux. Arch Linux provides the operating system and packages; Hyprland provides Wayland composition; ARGVUS coordinates the graphical session, shell surfaces, settings, appearance, desktop services and integrated applications.

It is therefore more than a Hyprland configuration or a collection of personal dotfiles. ARGVUS defines a supported session entry point, owns the lifecycle of its desktop services, provides a common settings and appearance model, and connects focused components such as networking, displays, power, notifications, accounts and removable storage.

ARGVUS does not need to replace every application. It integrates selected upstream applications where that is the most reliable choice, while keeping the desktop-wide behavior and configuration coherent.

For the implementation boundaries and package ownership, see the developer [architecture overview](./developer-guide/architecture/overview/) and [repository roles](./developer-guide/architecture/repository-roles/).

## Continue by goal

- [Read the FAQ](./user-guide/faq/).
- [Install ARGVUS](./user-guide/installation/).
- [Start a graphical or TTY session](./user-guide/sessions/).
- [Configure themes and wallpapers](./user-guide/appearance/).
- [Use the taskbar and Control Panel](./user-guide/desktop/).
- [Understand commands, paths and services](./reference/).
- [Study the architecture](./developer-guide/architecture/overview/).
