---
title: Themes
description: Theme families and the shared accent-color system.
slug: 0.1.1/docs/themes
---

Available themes: `argvus-dark-aether`, `argvus-dark-aether-aether-float`, `argvus-dark-silver`, `argvus-dark-silver-float`, `argvus-light-veil`, `argvus-light-veil-float`, `argvus-dark-aether-slate` and `argvus-dark-aether-slate-float`. Accent changes affect borders, titles, selections, and other highlights without replacing the theme background.

Open the selector with `SUPER + SHIFT + A`, use the **Accent** control in the sidebar, or run:

```sh
sh /usr/share/argvus/argvus/sh/accent-switch.sh
```

You can also apply one of the supported colors directly:

```sh
sh /usr/share/argvus/argvus/sh/accent-switch.sh '#17d174'
```

Palette: `#996548`, `#3590bd`, `#7391a5`, `#17d174`, `#cb17d1`, `#d1174f`, `#d1ce17`, `#9617d1` and `#595959`. The default accents are `#3590bd` for Dark, `#595959` for Dark Silver, `#181818` for Light, and `#7391a5` for Slate. Switching themes replaces any custom accent with the selected theme's default.

Theme and accent preferences are stored as small user files under `$XDG_CONFIG_HOME/argvus`. Runtime application config generated from those preferences is written under `$XDG_STATE_HOME/argvus/config`, with `~/.local/state` used when `XDG_STATE_HOME` is unset.

Dark themes use `default.png`; Light uses `argvus-light-veil.png`; Slate uses `argvus-dark-aether-slate.png`; and Dark Silver uses `argvus-dark-silver.png`. Normal and Float variants in each family share the same wallpaper. Wallpapers ship with the `argvus-appearance` package under `/usr/share/backgrounds/argvus/`.
