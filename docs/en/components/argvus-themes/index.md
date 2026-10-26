---
title: Themes
description: Install, select, customize, import and export ARGVUS themes.
---

A theme in ARGVUS sets the colors, surfaces, wallpaper defaults and layout geometry used across the desktop: Hyprland, Waybar, the Control Panel, the taskbar, the lock screen, notifications and ARGVUS applications. This guide covers everything you need to work with themes as a user.

## What is included in a theme

Each official theme is a complete visual identity with two modes:

- **Sticky** — a compact layout with small window gaps, no outer gaps and a taskbar close to the screen edge.
- **Float** — a spacious layout with larger gaps, rounded corners and margins around the taskbar and shell surfaces.

Selecting a theme does not change its identity: the palette, the default highlight color and the wallpaper association stay the same between modes. The mode is a separate setting, see [Official themes](/docs/components/argvus-themes/official-themes/).

## Pages in this guide

- [Installing themes](/docs/components/argvus-themes/installing-themes/) — where the official themes come from and how to install one or all of them with `pacman`.
- [Official themes](/docs/components/argvus-themes/official-themes/) — the catalog, the default highlight color of each theme, and how to choose a theme and its mode.
- [Customizing the highlight color](/docs/components/argvus-themes/highlight-color/) — change the accent color independently of the theme.
- [Importing and exporting themes](/docs/components/argvus-themes/import-and-export/) — export the active theme, with your highlight color and wallpaper, as a theme of your own and import it on another machine.

## Quick reference

| Task | Where |
| --- | --- |
| Pick a theme | **Control Center → Appearance → Themes → Official Themes** |
| Switch Sticky and Float | **Control Center → Appearance → Mode** |
| Change the highlight color | **Control Center → Appearance → Highlight color** |
| Export the active theme | **Control Center → Appearance → Themes → Export theme** |
| Import a theme | **Control Center → Appearance → Themes → Import theme** |
| Open the theme selector from the keyboard | `SUPER + SHIFT + T` |

## Where themes come from

ARGVUS ships two built-in themes, **ARGVUS Dark** and **ARGVUS Light**, with the `argvus-appearance` package. Every other official theme is a separate `argvus-theme-<name>` package. Installing a theme package adds it to the theme selector; no restart of the session is needed. Your own themes, exported or imported, are listed separately as **Custom Themes**.

## Keyboard hints

The Control Center theme pages show their keys in the footer:

- `↑` / `↓` — navigate.
- `Enter` — apply or open.
- `e` — export the active theme, on a Themes page.
- `i` — import a theme, on a Themes page.
- `d` — delete a custom theme.
- `Esc` — go back.

## Troubleshooting

- **A theme I installed does not appear.** Confirm that the package is installed with `pacman -Q argvus-theme-<name>`, then reopen the Control Center. The package must be installed with `argvus-appearance` present.
- **The highlight color changed after I switched theme.** A custom highlight color survives theme switches. Use **Reset to theme default** to give the theme its own color back.
- **A theme archive is not listed for import.** It must be directly in your home directory and hold `<id>/theme.toml`. See [Importing and exporting themes](/docs/components/argvus-themes/import-and-export/).
