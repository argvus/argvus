---
title: Matrice de paquet
description: Responsabilités du paquet ARGVUS.
---

| Package | Responsabilité |
| --- | --- |
| `argvus` | Package principal : dispatcher, commandes en cours d'exécution (configuration, traductions, clés, projets, terminal, moniteur système, réseau, Bluetooth, télémétrie, terminal TUI, configuration), les données partagées et les catalogues i18n. Tous les packages composants en dépendent. |
| `argvus` (helpers) | Helpers dans le même package : le Centre de Contrôle et l'Analyseur d'Utilisation du Disque (applications terminal), plus les applications GTK 4 pour le calendrier de la barre des tâches, les périphériques amovibles, le superposition de chargement, ARGVUS Focus et ARGVUS Clock. |
| `argvus` (greeter) | Accueil de connexion pour greetd, livré dans le même package et construit à partir de `crates/argvus-greeter`. |
| `argvus-desktop` | Profil de bureau et d'applications quotidiennes requis par le package principal `argvus`. Il ne dépend pas de `argvus`, donc la dépendance reste unidirectionnelle. |
| `argvus-devtools`, `argvus-pentest` | Méta-paquets indépendants optionnels pour les outils de développement et l'audit de sécurité autorisé. Le paquet principal n'installe pas ces profils. |
| `argvus-session` | Points d'entrée de session et cycle de vie de l'utilisateur. |
| `argvus-hyprland` | Configuration de Hyprland. |
| `argvus-control-panel` | Panneau de contrôle Quickshell. |
| `argvus-taskbar`, `argvus-waybar` | Configuration de Waybar et binaire patché. |
| `argvus-branding` | Ressources de logo et de marque textuelle. |
| `argvus-appearance`, `argvus-wallpapers`, `argvus-fonts`, `argvus-icons` | État d'apparence et ressources visuelles. |
| `argvus-display`, `argvus-power`, `argvus-notifications` | Fournisseurs de bureau. |
| `argvus-lock`, `argvus-boot-splash` | Écran de verrouillage et écran de démarrage. |
| `argvus-firewall` | Pare-feu, avec ses propres privilèges. |
| `argvus-launcher` | Configuration du lanceur et points d'entrée. |
| `argvus-app-profiles` | Profils uniquement de données pour les applications terminales groupées. |
| `argvus-portal` | Portail XDG et valeurs par défaut Wayland/DBus. |
| `argvus-theme-*` | Un paquet optionnel par famille de thèmes officielle. |
| `argvus-game-snake`, `argvus-games` | Optionnel. Snake est un projet indépendant ; `argvus-games` regroupe les jeux optionnels. |

## Paquets remplacés

`argvus` déclare `replaces`, `conflicts` et `provides` pour les paquets qu'il a absorbés, donc une mise à jour les supprime et leurs fichiers ne restent pas en tant qu'orphelins :

`argvus-config`, `argvus-i18n` (avec `argvus-language-en-us` et `argvus-language-pt-br`), `argvus-keys`, `argvus-projects`, `argvus-terminal`, `argvus-system-monitor`, `argvus-network`, `argvus-widget-telemetry` et `argvus-du`. `argvus` remplace également `argvus-control-center`, `argvus-taskbar-calendar`, `argvus-loading-theme`, `argvus-removable-devices`, et les anciens paquets scindés `argvus-gtk` et `argvus-greeter`, qu'il fournit toujours afin que les paquets qui en dépendent continuent à se résoudre. Le gestionnaire de comptes (`argvus accounts`) est construit à partir de `crates/argvus-accounts` et installé en tant que `/usr/lib/argvus/commands/accounts`, avec sa règle polkit. L'analyseur d'utilisation du disque est construit à partir de `crates/argvus-du` et installé en tant que `/usr/lib/argvus/commands/argvus-du`.

`/usr/bin/argvus-terminal` reste installé comme lanceur de compatibilité pour `argvus terminal`. Le terminal par défaut de Hyprland, le catalogue du Centre de contrôle et les applications par défaut enregistrées continuent de nommer cette commande.

Les paquets supprimés `argvus-pwvucontrol` et `argvus-snappy-switcher` ne sont fournis par aucun paquet.

Aucun ordre d'installation n'est requis parmi les paquets composants ARGVUS : `argvus` est leur paquet principal, et chaque composant en dépend. L'installation de `argvus` installe également le profil `argvus-desktop` ; `argvus-devtools` et `argvus-pentest` restent en option. La présence d'un paquet dans ce tableau ne signifie pas qu'il possède l'ensemble de la fonctionnalité : par exemple, la barre des tâches combine `argvus-waybar`, `argvus-taskbar`, `argvus-session` et les fournisseurs de domaine.
