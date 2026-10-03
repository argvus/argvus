---
title: Appearance
description: Configure the visual language shared by ARGVUS components.
---

The appearance system applies one theme state to Hyprland, GTK, Qt, Waybar, Quickshell, Rofi, Dunst, Hyprlock and ARGVUS applications.

## Theme families

The twenty-four families below are defined by the current theme payload. The color column is the family's default active-border/accent color; users may choose another valid RGB accent.

| Family | Reset highlight color | Base background | Modes |
| --- | --- | --- | --- |
| One Dark | `#61AFEF` | `#282C34` | Sticky, Float |
| Dracula | `#BD93F9` | `#282A36` | Sticky, Float |
| ARGVUS Dark | `#3590bd` | `#111316` | Sticky, Float |
| Dark Silver | `#595959` | `#111316` | Sticky, Float |
| Dark Slate | `#7391a5` | `#2f3541` | Sticky, Float |
| Dark Universe | `#eeeeee` | `#000000` | Sticky, Float |
| ARGVUS Dark Gruvbox High | `#D79921` | `#282828` | Sticky, Float |
| ARGVUS Dark Gruvbox | `#D4BE98` | `#282828` | Sticky, Float |
| ARGVUS Light | `#181818` | `#f7f7f7` | Sticky, Float |
| GitHub Light | `#0969DA` | `#FFFFFF` | Sticky, Float |
| Catppuccin Latte | `#1E66F5` | `#EFF1F5` | Sticky, Float |
| ARGVUS Light Gruvbox | `#458588` | `#FBF1C7` | Sticky, Float |
| Rosé Pine | `#C4A7E7` | `#191724` | Sticky, Float |
| Tokyo-Night | `#7AA2F7` | `#1A1B26` | Sticky, Float |
| Solitude | `#798186` | `#101315` | Sticky, Float |
| Sunset | `#E2BE8A` | `#0F0F0F` | Sticky, Float |
| Hackerman | `#82FB9C` | `#0B0C16` | Sticky, Float |
| Monokai Dark | `#78DCE8` | `#2D2A2E` | Sticky, Float |
| One Light | `#4078F2` | `#FAFAFA` | Sticky, Float |
| Flexoki Light | `#205EA6` | `#FFFCF0` | Sticky, Float |
| Everforest Light | `#3A94C5` | `#FDF6E3` | Sticky, Float |
| ARGVUS Kanagawa Lotus | `#4D699B` | `#F2ECBC` | Sticky, Float |
| Nord Light | `#5E81AC` | `#ECEFF4` | Sticky, Float |

The color shown in the table is the value written to `config.json` when that family is selected or when **Reset to theme default** is used. A manually chosen six-digit RGB accent applies until the next theme switch, which replaces it with the selected theme's default.

**Sticky** is the compact layout: tiled windows use small internal gaps, the outer gaps are zero and the taskbar sits close to the screen edge. **Float** adds larger gaps, rounded corners, shadows and margins around the taskbar and shell surfaces.

The mode changes geometry and surface treatment; it does not create a sixth color family. See [Themes](/docs/argvus-themes/) for selection and [Desktop](/docs/user-guide/desktop/) for taskbar and window placement.

- [Themes](/docs/argvus-themes/)
- [Wallpapers](/docs/argvus-appearance/wallpapers/)
- [Fonts and icons](/docs/argvus-appearance/fonts-and-icons/)
- [Effects](/docs/argvus-appearance/effects/)

The Control Center exposes the user-facing settings. The implementation is shared by `argvus-appearance`, `argvus-session`, the shell components and the asset packages. See [appearance architecture](/docs/developer-guide/subsystems/appearance-and-themes/).
