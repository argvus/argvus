---
title: Sitzungen
description: Starten und verwalten Sie eine ARGVUS-Grafiksitzung.
---
`argvus-session` besitzt die Einstiegspunkte der Sitzung und den `systemd --user`-Lebenszyklus. Es startet Hyprland über `argvus start`, importiert die Sitzungsumgebung und verwaltet das ARGVUS-Ziel.

- [Grafische Sitzung](/docs/argvus-session/graphical-session/)
- [TTY-Sitzung](/docs/argvus-session/tty-session/)
- [Gruß](/docs/components/argvus-greeter/)

Nützliche Diagnosen:

```sh
argvus sessionctl status
argvus sessionctl logs
argvus sessionctl restart waybar
```
Siehe den Entwickler [runtime lifecycle](/docs/developer-guide/architecture/runtime-lifecycle/) für Dienstverantwortung.
