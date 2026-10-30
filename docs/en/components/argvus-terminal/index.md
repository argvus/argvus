---
title: Terminal
description: Open the ARGVUS terminal and terminal application profiles.
---

Open the configured terminal with:

```sh
argvus --terminal
```

`argvus terminal` (a subcommand of `argvus`) provides the terminal integration and Kitty configuration. `argvus-app-profiles` supplies profiles for terminal applications such as Superfile and Yazi; it does not provide the `argvus` dispatcher.

`argvus-config` projects the theme into `data/generated/terminal/`, and `argvus --yazi` and `argvus --spf` read their configuration from the projected `data/generated/yazi/` and `data/generated/superfile/` trees rather than from copies materialized into your home directory. A complete native override in the application's own config directory still wins, so you can replace the projected tree with your own configuration.

Theme changes regenerate Kitty's derived files under `$XDG_CACHE_HOME/argvus/argvus-terminal`. The generator serializes updates and replaces each file atomically. If the terminal still does not open, inspect the active theme and regenerate the cache with:

```sh
argvus config get /appearance/theme --raw
argvus terminal --apply
argvus terminal --print-config
```

The terminal reads the managed theme and accent from
`$XDG_CONFIG_HOME/argvus/config.json` first. The legacy `.active-theme` and
`.accent-color` files remain fallbacks for older installations.

## Transparency and blur

Under **Control Center → Appearance → Terminal**, configure **Transparency**
and **Blur** independently from `0%` to `100%`. Changes are committed with the
**Apply** button and regenerate the Kitty profile under
`$XDG_CACHE_HOME/argvus/argvus-terminal`; existing Kitty windows receive the
updated configuration through a reload.

`0%` transparency keeps the background opaque and `100%` makes it fully
transparent. Blur uses Kitty's supported background-blur radius and only has a
visible effect when the window is transparent.

## Long-running command notification

Commands that run for 30 seconds or more send a notification when they finish,
as long as the terminal window is unfocused. The notification uses Kitty's
`notify_on_cmd_finish` with the shell integration that is already enabled, so
`~/.zshrc` and `~/.bashrc` do not need changes. The 30-second threshold is fixed
in the Kitty configuration.

## TUI App Mode

ARGVUS Control Center and taskbar TUI popups use `argvus tui-terminal` with a
dedicated Kitty app profile. It inherits the active ARGVUS theme and font,
assigns a stable window class, and exits with its child instead of opening a
shell. Control Center hides tabs and has independent Transparency and Blur
values; taskbar monitor and temperature popups use a bottom tab bar for their
localized `Press q to close` title. The launcher accepts `--title TITLE` to set
a fixed Kitty window and tab title. The normal terminal keeps its tabs and
settings.
