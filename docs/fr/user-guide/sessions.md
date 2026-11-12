---
title: Séances
description: Démarrer et gérer une session graphique ARGVUS.
---
`argvus-session` possède les points d'entrée de session et le cycle de vie `systemd --user`. Il lance Hyprland via `argvus start`, importe l'environnement de session et gère la cible ARGVUS.

- [Session graphique](/docs/argvus-session/graphical-session/)
- [Session TTY](/docs/argvus-session/tty-session/)
- [Greeter](/docs/components/argvus-greeter/)

Diagnostics utiles :

```sh
argvus sessionctl status
argvus sessionctl logs
argvus sessionctl restart waybar
```
Voir le développeur [runtime lifecycle](/docs/developer-guide/architecture/runtime-lifecycle/) pour la propriété du service.
