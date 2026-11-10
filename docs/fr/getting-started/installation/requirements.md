---
title: Exigences
description: Exigences système et prérequis pour ARGVUS.
---
ARGVUS cible Arch Linux et nécessite une pile graphique compatible Wayland, Hyprland et les dépendances d'exécution fournies par les paquets. Un gestionnaire d'affichage est optionnel : ARGVUS prend en charge un flux de connexion graphique via greetd et un flux manuel sur TTY.

Avant l'installation, assurez-vous que :

- le système est entièrement mis à jour avec `pacman` ;
- un noyau fonctionnel et un pilote graphique sont installés ;
- l'utilisateur peut exécuter une session Wayland ;
- le réseau est disponible lorsque le dépôt de paquets est configuré.

Les machines virtuelles peuvent nécessiter les paramètres de compatibilité décrits dans [Virtual machines](/docs/user-guide/virtual-machines/).
