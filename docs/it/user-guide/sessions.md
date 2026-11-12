---
title: Sessioni
description: Avvia e gestisci una sessione grafica ARGVUS.
---
`argvus-session` possiede i punti di accesso della sessione e il ciclo di vita di `systemd --user`. Avvia Hyprland tramite `argvus start`, importa l'ambiente della sessione e gestisce il target ARGVUS.

- [Sessione grafica](/docs/argvus-session/graphical-session/)
- [Sessione TTY](/docs/argvus-session/tty-session/)
- [Saluto](/docs/components/argvus-greeter/)

Diagnostica utile:

```sh
argvus sessionctl status
argvus sessionctl logs
argvus sessionctl restart waybar
```
Consulta lo sviluppatore [runtime lifecycle](/docs/developer-guide/architecture/runtime-lifecycle/) per la responsabilità del servizio.
