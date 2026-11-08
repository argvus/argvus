---
title: Appearance
description: Configure the visual language shared by ARGVUS components.
---

The appearance system applies one theme state to Hyprland, GTK, Qt, Waybar, Quickshell, Rofi, Dunst, Hyprlock and ARGVUS applications.

## Theme families

The twenty-three families below are defined by the current theme payload. The color column is the family's default active-border/accent color; users may choose another valid RGB accent.

| Family | Reset highlight color | Base background | Modes |
| --- | --- | --- | --- |
| One Dark | `#61AFEF` | `#282C34` | Sticky, Spaced |
| Dracula | `#BD93F9` | `#282A36` | Sticky, Spaced |
| ARGVUS Dark | `#3590bd` | `#111316` | Sticky, Spaced |
| Dark Silver | `#595959` | `#111316` | Sticky, Spaced |
| Dark Slate | `#7391a5` | `#2f3541` | Sticky, Spaced |
| Dark Universe | `#eeeeee` | `#000000` | Sticky, Spaced |
| ARGVUS Dark Gruvbox High | `#D79921` | `#282828` | Sticky, Spaced |
| ARGVUS Dark Gruvbox | `#D4BE98` | `#282828` | Sticky, Spaced |
| ARGVUS Light | `#181818` | `#f7f7f7` | Sticky, Spaced |
| GitHub Light | `#0969DA` | `#FFFFFF` | Sticky, Spaced |
| Catppuccin Latte | `#1E66F5` | `#EFF1F5` | Sticky, Spaced |
| ARGVUS Light Gruvbox | `#458588` | `#FBF1C7` | Sticky, Spaced |
| Rosé Pine | `#C4A7E7` | `#191724` | Sticky, Spaced |
| Tokyo-Night | `#7AA2F7` | `#1A1B26` | Sticky, Spaced |
| Solitude | `#798186` | `#101315` | Sticky, Spaced |
| Sunset | `#E2BE8A` | `#0F0F0F` | Sticky, Spaced |
| Matrix | `#82FB9C` | `#0B0C16` | Sticky, Spaced |
| Monokai Dark | `#78DCE8` | `#2D2A2E` | Sticky, Spaced |
| Lady Mist | `#354B78` | `#131313` | Sticky, Spaced |
| One Light | `#4078F2` | `#FAFAFA` | Sticky, Spaced |
| Everforest Light | `#3A94C5` | `#FDF6E3` | Sticky, Spaced |
| Solarized Light | `#268BD2` | `#FDF6E3` | Sticky, Spaced |
| Nord Light | `#5E81AC` | `#ECEFF4` | Sticky, Spaced |
| Watercolor | `#1A639C` | `#E9E3D3` | Sticky, Spaced |

The color shown in the table is the value written to `config.json` when that family is selected or when **Reset to theme default** is used. A manually chosen six-digit RGB accent applies until the next theme switch, which replaces it with the selected theme's default.

**Sticky** is the compact layout: tiled windows use small internal gaps, the outer gaps are zero and the taskbar sits close to the screen edge. **Spaced** adds larger gaps, rounded corners, shadows and margins around the taskbar and shell surfaces.

Existing `float` layout values and `-float` theme identifiers are recognized as legacy names for Spaced. New layout values and resolved theme identifiers use `spaced` and `-spaced`.

The mode changes geometry and surface treatment; it does not create a sixth color family. See [Themes](/docs/user-guide/appearance/) for selection and [Desktop](/docs/user-guide/desktop/) for taskbar and window placement.

- [Themes](/docs/user-guide/appearance/)
- [Wallpapers](/docs/argvus-appearance/wallpapers/)
- [Fonts and icons](/docs/argvus-appearance/fonts-and-icons/)
- [Effects](/docs/argvus-appearance/effects/)

The Control Center exposes the user-facing settings. The implementation is shared by `argvus-appearance`, `argvus-session`, the shell components and the asset packages. See [appearance architecture](/docs/developer-guide/subsystems/appearance-and-themes/).

## Window wobble

Open **Desktop & Input → Hyprland → Animations → Windows** and use the
**Wobble** controls. Search for `wobble`, or open the page directly:

```sh
argvus control-center hyprland animations windows
```

Wobble is disabled by default. Its ARGVUS defaults are stiffness `100`, damping
`10`, mass `1`, intensity `1` and mesh resolution `8`. Stiffness and mass accept
`0.0001–1000`, damping `0–1000`, intensity `0–2`, and mesh an integer `2–32`.
Higher mesh resolution increases rendering cost. Use the arrow keys to adjust
values or Enter to type them. **Restore Wobble defaults** changes the draft;
**Apply** saves all pending animation edits and reloads Hyprland once, without
restarting the session. Going back within Animations keeps the shared draft.

The global Animations switch suppresses Wobble while retaining its individual
settings. The canonical values live under `effects.wobble_*` in ARGVUS state;
`hypr/state.lua` is generated and must not be edited manually.

This requires Hyprland **0.57.0 or newer**, enforced by `argvus-hyprland`'s
package dependency. Version 0.56 is unsupported for this configuration. The
intensity maximum follows the [released 0.57.0 source](https://github.com/hyprwm/Hyprland/blob/v0.57.0/src/config/values/ConfigValues.cpp),
which limits it to `2`, even though newer documentation may list `3`.
