---
title: Introduction
description: Ce qu'est ARGVUS, comment fonctionne son bureau et où ses composants s’intègrent.
---
ARGVUS est un environnement de bureau modulaire Hyprland et Wayland pour Arch Linux. Il s'agit d'une collection intégrée de paquets plutôt qu'une application monolithique : la session, le compositeur, les surfaces du shell, les applications de paramètres, les fournisseurs et les éléments visuels fonctionnent ensemble grâce à un état partagé et des contrats d'exécution explicites.

Cette page est la carte de la documentation. Elle explique l'ensemble du bureau à un niveau général ; les pages liées contiennent les détails opérationnels et d'implémentation.

## Comment fonctionne ARGVUS

La session normale suit ce chemin :

```text
graphical login or TTY
        ↓
argvus-session → argvus start
        ↓
Hyprland + argvus-hyprland
        ↓
argvus-session.target (systemd --user)
        ↓
taskbar · control panel · notifications · wallpaper · idle · clipboard
```
`argvus-session` gère le cycle de vie des sessions et la configuration de l'environnement. Il n'implémente pas toutes les fonctionnalités du bureau : chaque composant conserve la propriété de ses propres commandes, configurations et charges de service.

Pour l'installation et le démarrage, voir [Installation](/docs/getting-started/installation/) et [Sessions](/docs/user-guide/sessions/). Pour la propriété des services, voir le [cycle de vie runtime](/docs/developer-guide/architecture/runtime-lifecycle/) pour les développeurs.

## Couches principales

### Points d'entrée et infrastructure

Le package `argvus` fournit le dispatcher `/usr/bin/argvus` et le point d'entrée principal du package. `argvus-session` fournit des points d'entrée graphiques et TTY, la cible utilisateur et `argvus sessionctl`. La fonctionnalité partagée TUI et la localisation proviennent de l'espace de travail `argvus` (les crates `argvus-tui` et `argvus-i18n`) et de la commande `argvus i18n`.

### Compositeur et shell de bureau

`argvus-hyprland` fournit la configuration et les raccourcis clavier de Hyprland. `argvus-waybar` fournit le binaire Waybar patché, tandis que `argvus-taskbar` possède la configuration et les actions de la barre des tâches. `argvus-control-panel` fournit le panneau Quickshell, et `argvus widget-telemetry` fournit des widgets système optionnels, en cours dans `argvus`.

Voir [Desktop](/docs/user-guide/desktop/) pour l'utilisation et [shell surfaces](/docs/developer-guide/subsystems/shell-surfaces/) pour la responsabilité de l'implémentation.

### Paramètres et applications

`argvus control-center` est l'application de paramètres orientée clavier, livrée comme un outil dans `argvus`. Les outils calendrier et périphériques amovibles sont des assistants GTK 4 dans `argvus` ; le moniteur système et le terminal sont des sous-commandes de `argvus` ; le lanceur est `argvus-launcher`. Tous sont intégrés avec le shell et le dispatcher.

Voir [Applications](/docs/user-guide/applications/) et la [référence de commande](/docs/reference/command-line/).

### Fournisseurs et intégrations

L'affichage, le réseau, le Bluetooth, l'alimentation, les notifications, le pare-feu et le stockage amovible sont fournis par des projets spécialisés. Le Centre de contrôle et le Panneau de configuration consomment ces fournisseurs ; ils ne les remplacent pas.

Les écrans de connexion, de verrouillage et de démarrage sont également séparés : `argvus-greeter` gère greetd, `argvus-lock` gère Hyprlock, l'assistant de transfert fourni dans `argvus` dessine le spinner de connexion, la superposition de chargement gère les transitions de thème, et `argvus-boot-splash` gère l'écran de démarrage.

Voir [Matériel](/docs/user-guide/hardware/), [Confidentialité et sécurité](/docs/user-guide/privacy-and-security/) et la [référence de composants pour développeurs](/docs/developer-guide/).

## État d'apparence partagé

Les préférences de thème, d'accent, de fond d'écran, de police et d'effets sont stockées sous forme d'état logique dans `$XDG_CONFIG_HOME/argvus` (normalement `~/.config/argvus`). `argvus-config` est le seul composant qui écrit dans `data/generated/`, où il projette les fichiers consommateurs pour Hyprland, GTK, Qt6ct, Waybar, Quickshell, Dunst, les notifications, l'écran de verrouillage, Yazi, Superfile et les terminaux. Rofi fait exception : `argvus launcher` emballe sa configuration et ses thèmes en fichiers simples, et les copies modifiées par l'utilisateur vivent sous `data/rofi/`. Les assistants d'apparence enregistrent le changement d'état puis réconcilient les applications externes ; ils n'écrivent jamais l'arborescence générée.

Le répertoire généré n'est pas la source de vérité. Voir [Appearance](/docs/user-guide/appearance/) et [configuration and state](/docs/developer-guide/architecture/configuration-and-state/).

Le système visuel contrôle également la disposition. Le mode Sticky est compact et carré ; le mode Spaced utilise des espaces de fenêtres plus larges, des coins arrondis et des marges plus larges autour de la barre des tâches et du shell. La barre des tâches est par défaut placée en haut mais supporte une position en haut/en bas grâce à l'état de disposition partagé. Les effets contrôlent la transparence, le flou, les ombres et les animations sur le compositeur et les surfaces du bureau.

Consultez le [tableau des thèmes](/docs/user-guide/appearance/) pour les familles et couleurs actuelles, [effets](/docs/argvus-appearance/effects/) pour le contrat d'effets partagés, et [barre des tâches](/docs/argvus-taskbar/taskbar/) pour le placement des barres et des fenêtres.

## Ce qu'ARGVUS est - et n'est pas

ARGVUS est la couche de l'environnement de bureau au-dessus d'Arch Linux. Arch Linux fournit le système d'exploitation et les paquets ; Hyprland fournit la composition Wayland ; ARGVUS coordonne la session graphique, les surfaces du shell, les paramètres, l'apparence, les services de bureau et les applications intégrées.

Il s'agit donc de plus qu'une configuration Hyprland ou d'une collection de dotfiles personnels. ARGVUS définit un point d'entrée de session pris en charge, gère le cycle de vie de ses services de bureau, fournit un modèle commun de paramètres et d'apparence, et connecte des composants ciblés tels que le réseau, les écrans, l'alimentation, les notifications, les comptes et le stockage amovible.

ARGVUS n'a pas besoin de remplacer toutes les applications. Il intègre des applications sélectionnées depuis l'aval lorsque c'est le choix le plus fiable, tout en gardant un comportement et une configuration cohérents à l'échelle du bureau.

Le nom combine deux idées. *ARGV*, de *argument vector*, est la liste des arguments qu'un programme reçoit, et reflète l'objectif axé sur le clavier de contrôler le bureau via des commandes et des raccourcis. *US*, de *User System*, reflète l'objectif d'un bureau centré sur son utilisateur. Pour l'explication complète et l'audio de prononciation, voir [À propos de ARGVUS](/about/).

Pour les limites de mise en œuvre et la propriété des packages, voir le développeur [aperçu de l'architecture](/docs/developer-guide/architecture/overview/) et [rôles du dépôt](/docs/developer-guide/architecture/repository-roles/).

## Continuer par objectif

- [Lire la FAQ](/docs/user-guide/faq/).
- [Installer ARGVUS](/docs/getting-started/installation/).
- [Démarrer une session graphique ou TTY](/docs/user-guide/sessions/).
- [Configurer les thèmes et fonds d'écran](/docs/user-guide/appearance/).
- [Utiliser la barre des tâches et le Panneau de configuration](/docs/user-guide/desktop/).
- [Comprendre les commandes, les chemins et les services](/docs/reference/).
- [Étudier l'architecture](/docs/developer-guide/architecture/overview/).
