---
title: Mapa de integração
description: Relações entre os projetos no runtime.
slug: pt/0.4.0/docs/developer-guide/architecture/integration-map
---

```text
greeter ou TTY -> argvus-session -> argvus-start -> Hyprland
                 -> argvus-session.target
                    -> taskbar / painel / notificações / wallpaper / idle

Control Center -> argvus-config (estado canônico) -> argvus-sessionctl reload
argvus-config  -> único escritor de data/generated/
                  Hyprland / GTK / Qt6ct / Waybar / Quickshell / Dunst /
                  Hyprlock / Yazi / Superfile / terminal / fontes / dispositivos removíveis
theme-switch / accent-switch -> argvus-config (commit) -> adapters externos
                  arquivo do Qt6ct / settings.ini do GTK + gsettings / terminais /
                  system-monitor / snappy-switcher / foot / superfile / greeter
```
