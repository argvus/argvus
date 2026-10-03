---
title: First configuration
description: A practical first-boot path for configuring ARGVUS.
---

After the first graphical session, you do not need to configure every part of ARGVUS. Start with the settings that affect how the desktop feels, then configure hardware and services as needed.

## A useful order

1. Read the [desktop overview](/docs/user-guide/desktop/) to identify windows, workspaces, the taskbar, the Control Panel and the launcher.
2. Open the [Control Center](/docs/argvus-control-center/) and choose a theme, mode, accent and wallpaper in **Appearance**.
3. Adjust [windows and layout](/docs/argvus-hyprland/windows-and-layout/) if the default gaps, borders or taskbar spacing do not suit your screen.
4. Configure the [taskbar](/docs/argvus-taskbar/taskbar/) and decide which [Control Panel](/docs/argvus-control-panel/) cards you want visible.
5. Review [keyboard shortcuts](/docs/argvus-hyprland/keyboard-shortcuts/) and change only the bindings you actually use.
6. Configure [mouse and touchpad](/docs/argvus-hyprland/input/), displays, networking and power from the relevant settings pages.

The first four steps are optional personalization. The remaining steps are normally only needed when your hardware, language or workflow requires them.

## Two places to change settings

Use **Control Center** for settings that describe how ARGVUS should be configured: appearance, layout, fonts, input, shortcuts, language, region and default applications. Use **Control Panel** for status and frequent session actions such as volume, brightness, network state, notifications, power and session controls.

See [Control Center vs Control Panel](/docs/argvus-control-center/) for the full distinction and [Where to configure things](/docs/user-guide/where-to-configure/) for a quick index.

## Changes, persistence and recovery

Most Control Center changes are applied while the session is running and saved as ARGVUS user state. Appearance changes are committed to the canonical configuration, and `argvus-config` projects the files consumed by Hyprland and the shell from it. A Control Panel toggle may be a direct action or a session state change instead of a permanent configuration.

When a page offers **Reset defaults**, use that page action rather than deleting generated files. Appearance has separate reset actions for accents, wallpapers and layout values; the keybinding page has its own restore action. There is no single universal reset button for every desktop setting.

## Next steps

- [Appearance](/docs/user-guide/appearance/) — themes, wallpapers, effects and layout.
- [Desktop layout](/docs/argvus-hyprland/windows-and-layout/) — understand gaps, borders and panel space.
- [Input devices](/docs/argvus-hyprland/input/) — mouse and touchpad controls.
- [Troubleshooting theme and wallpaper](/docs/user-guide/troubleshooting/theme-and-wallpaper/) — recovery when a visual change is not what you expected.
