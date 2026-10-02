---
title: Sessões
description: Inicie e gerencie uma sessão ARGVUS.
slug: pt/0.4.0/docs/user-guide/sessions
---

`argvus-session` possui os pontos de entrada e o ciclo de vida `systemd --user`. Ele inicia o Hyprland através de `argvus-start` e gerencia o target ARGVUS.

* [Sessão gráfica](/pt/docs/argvus-session/graphical-session/)
* [Sessão TTY](/pt/docs/argvus-session/tty-session/)
* [Greeter](/pt/docs/argvus-greeter/)

```sh
argvus-sessionctl status
argvus-sessionctl logs
argvus-sessionctl restart waybar
```
