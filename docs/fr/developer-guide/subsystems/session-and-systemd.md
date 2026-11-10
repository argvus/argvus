---
title: Session et systemd
description: La cible de session ARGVUS et ses services utilisateur.
---
Les définitions d'unités sont emballées par `argvus-session` sous `/usr/share/argvus/session/config/systemd/user/`. Le gestionnaire de session possède l'importation du cycle de vie et de l'environnement. Les dépôts de composants possèdent les commandes exécutées par leurs unités.

Utilisez `argvus sessionctl` au lieu de démarrer manuellement un composant lors du diagnostic de la session intégrée.
