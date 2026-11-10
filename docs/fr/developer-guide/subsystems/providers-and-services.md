---
title: Fournisseurs et services
description: Packages de fournisseurs et leurs interfaces système.
---
Les projets fournisseurs exposent des interfaces ciblées aux utilisateurs de l'interface :

- `argvus network` et `argvus bluetooth` intègrent NetworkManager et les assistants Bluetooth ;
- `argvus-display` intègre l'état du moniteur avec Hyprland ;
- `argvus-power` intègre les actions d'inactivité, de verrouillage et d'alimentation ;
- `argvus-firewall` gère `argvus-firewall.service` ;
- l'assistant `argvus removable-devices` dans `argvus` utilise UDisks2 ;
- l'assistant `argvus control-center` dans `argvus` utilise les API des fournisseurs et les interfaces D-Bus du système.

Le Centre de contrôle est un client de ces fournisseurs, et non leur remplacement.
