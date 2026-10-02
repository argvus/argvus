---
title: Sessions
description: Start and manage an ARGVUS graphical session.
---

`argvus-session` owns the session entry points and the `systemd --user` lifecycle. It starts Hyprland through `argvus-start`, imports the session environment and manages the ARGVUS target.

- [Graphical session](./sessions/graphical-session/)
- [TTY session](./sessions/tty-session/)
- [Greeter](./sessions/greeter/)

Useful diagnostics:

```sh
argvus-sessionctl status
argvus-sessionctl logs
argvus-sessionctl restart waybar
```

See the developer [runtime lifecycle](../developer-guide/architecture/runtime-lifecycle/) for service ownership.
