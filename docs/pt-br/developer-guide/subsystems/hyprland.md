---
title: Integração Hyprland
description: Configuração e contratos do compositor.
slug: pt/0.4.0/docs/developer-guide/subsystems/hyprland
---

`argvus-hyprland` empacota `/usr/share/argvus/hyprland/config/hyprland.lua`, keybindings e helpers. `argvus-session` inicia o compositor, `argvus-config` projeta os arquivos de tema, layout, bordas e espaçamento do Hyprland em `data/generated/hypr/`, e `argvus-appearance` reconcilia o compositor via `hyprctl` e `argvus-display` fornece overrides de monitor.
