---
title: Сессии
description: Запустите и управляйте графической сессией ARGVUS.
---
`argvus-session` управляет точками входа сессии и жизненным циклом `systemd --user`. Он запускает Hyprland через `argvus start`, импортирует окружение сессии и управляет целью ARGVUS.

- [Графическая сессия](/docs/argvus-session/graphical-session/)
- [TTY сессия](/docs/argvus-session/tty-session/)
- [Приветствие](/docs/components/argvus-greeter/)

Полезная диагностика:

```sh
argvus sessionctl status
argvus sessionctl logs
argvus sessionctl restart waybar
```
Обратитесь к разработчику [runtime lifecycle](/docs/developer-guide/architecture/runtime-lifecycle/) для владения сервисом.
