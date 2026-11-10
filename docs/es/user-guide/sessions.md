---
title: Sesiones
description: Iniciar y gestionar una sesión gráfica de ARGVUS.
---
`argvus-session` posee los puntos de entrada de la sesión y el ciclo de vida de `systemd --user`. Inicia Hyprland a través de `argvus start`, importa el entorno de la sesión y gestiona el objetivo ARGVUS.

- [Sesión gráfica](/docs/argvus-session/graphical-session/)
- [Sesión TTY](/docs/argvus-session/tty-session/)
- [Greeter](/docs/components/argvus-greeter/)

Diagnósticos útiles:

```sh
argvus sessionctl status
argvus sessionctl logs
argvus sessionctl restart waybar
```
Consulta al desarrollador [runtime lifecycle](/docs/developer-guide/architecture/runtime-lifecycle/) para la propiedad del servicio.
