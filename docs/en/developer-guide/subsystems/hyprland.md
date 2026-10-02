---
title: Hyprland integration
description: ARGVUS compositor configuration and integration contracts.
---

`argvus-hyprland` packages `/usr/share/argvus/hyprland/config/hyprland.lua`, keybindings and helper scripts. `argvus-session` launches Hyprland and applies session compatibility setup; `argvus-config` projects the Hyprland theme, layout, border and spacing files into `data/generated/hypr/`, and `argvus-appearance` reconciles the compositor over `hyprctl`; `argvus-display` supplies monitor overrides.

The legacy package path is not part of the current installed layout.
