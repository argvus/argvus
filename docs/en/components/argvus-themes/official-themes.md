---
title: Official themes
description: The official ARGVUS theme catalog, default highlight colors, and how to select a theme and its Sticky or Float mode.
---

This page lists the official themes available as theme packages and explains how to select one. For installing them, see [Installing themes](/docs/components/argvus-themes/installing-themes/).

## Select a theme

1. Open **Control Center → Appearance → Themes**.
2. Choose **Official Themes**.
3. Choose **Dark** or **Light**.
4. Choose the theme family. The theme is applied as soon as you select it.

The selected theme is applied to the whole desktop at once: the running taskbar, panels, notifications, windows and wallpaper change colour in place, without a loading screen and without restarting. ARGVUS keeps your wallpaper selection unless the theme is applied with its own wallpaper association.

You can also press `SUPER + SHIFT + T` to open the theme selector from the keyboard. Use the arrow keys to move through the categories, `Enter` to choose, and `Esc` to close.

## Sticky and Float

Each theme works in two modes. The mode is independent of the theme: changing the theme keeps the current mode, and changing the mode keeps the current theme.

| Mode | Window gaps | Outer gap | Window rounding | Taskbar and shell margin |
| --- | ---: | ---: | ---: | ---: |
| Sticky | `2` | `0` | `0` | near the edge |
| Float | `10` | `18` | `4` | `18` |

To switch the mode, open **Control Center → Appearance → Mode** and choose **Sticky** or **Float**. Your manual adjustments to these layout values remain in effect until the next explicit theme selection.

## Dark themes

| Theme | Package | Default highlight |
| --- | --- | --- |
| One Dark | `argvus-theme-one-dark` | `#61AFEF` |
| Dracula | `argvus-theme-dracula` | `#BD93F9` |
| Silver Dark | `argvus-theme-silver-dark` | `#595959` |
| Slate Dark | `argvus-theme-slate-dark` | `#7391A5` |
| Universe | `argvus-theme-universe` | `#EEEEEE` |
| Gruvbox High Dark | `argvus-theme-gruvbox-high-dark` | `#D79921` |
| Gruvbox Dark | `argvus-theme-gruvbox-dark` | `#D4BE98` |
| Rosé Pine | `argvus-theme-rose-pine` | `#C4A7E7` |
| Tokyo Night | `argvus-theme-tokyo-night` | `#7AA2F7` |
| Solitude | `argvus-theme-solitude` | `#798186` |
| Sunset | `argvus-theme-sunset` | `#E2BE8A` |
| Matrix | `argvus-theme-matrix` | `#82FB9C` |
| Monokai Dark | `argvus-theme-monokai-dark` | `#78DCE8` |

## Light themes

| Theme | Package | Default highlight |
| --- | --- | --- |
| GitHub Light | `argvus-theme-github-light` | `#0969DA` |
| Solarized Light | `argvus-theme-solarized-light` | `#268BD2` |
| Nord Light | `argvus-theme-nord-light` | `#5E81AC` |
| One Light | `argvus-theme-one-light` | `#4078F2` |
| Everforest Light | `argvus-theme-everforest-light` | `#3A94C5` |
| Catppuccin Latte | `argvus-theme-catppuccin-latte` | `#1E66F5` |
| Gruvbox Light | `argvus-theme-gruvbox-light` | `#458588` |

## Built-in themes

**ARGVUS Dark** and **ARGVUS Light** are provided by `argvus-appearance` and are always available under **Official Themes**. ARGVUS Dark uses the default highlight `#3590BD`; ARGVUS Light uses `#181818`.

## What a theme controls

A theme defines the palette and surface treatment used by the desktop components, the default highlight color, the default wallpaper association and the transparency and blur defaults. The desktop components read this state rather than storing their own copy, so a theme change reaches the compositor, the taskbar, the Control Panel, the lock screen, notifications and applications together.

The default highlight color is only the starting point. You can replace it with any six-digit color, see [Customizing the highlight color](/docs/components/argvus-themes/highlight-color/).
