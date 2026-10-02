---
title: Sessões
description: Inicie e gerencie uma sessão ARGVUS.
slug: pt/0.4.0/docs/user-guide/sessions
---

`argvus-session` possui os pontos de entrada e o ciclo de vida `systemd --user`. Ele inicia o Hyprland através de `argvus-start` e gerencia o target ARGVUS.

* [Sessão gráfica](./sessions/graphical-session/)
* [Sessão TTY](./sessions/tty-session/)
* [Greeter](./sessions/greeter/)

```sh
argvus-sessionctl status
argvus-sessionctl logs
argvus-sessionctl restart waybar
```
