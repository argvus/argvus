---
title: Sessions
description: Start and manage an ARGVUS graphical session.
---

`argvus-session` owns the session entry points and the `systemd --user` lifecycle. It starts Hyprland through `argvus-start`, imports the session environment and manages the ARGVUS target.

- [Graphical session](/docs/argvus-session/graphical-session/)
- [TTY session](/docs/argvus-session/tty-session/)
- [Greeter](/docs/argvus-greeter/)

Useful diagnostics:

```sh
argvus-sessionctl status
argvus-sessionctl logs
argvus-sessionctl restart waybar
```

See the developer [runtime lifecycle](/docs/developer-guide/architecture/runtime-lifecycle/) for service ownership.
