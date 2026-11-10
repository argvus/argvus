---
title: Rôles du dépôt
description: Responsabilités des dépôts sources ARGVUS après la consolidation de l'espace de travail.
---
Les dépôts sont regroupés par responsabilité plutôt que par page de navigation :

- workspace : `argvus` (dispatcher, commandes en cours d'exécution, assistants GTK 4, crates partagés, le greeter dans `crates/argvus-greeter` et les données des commandes) ;
- session et intégration : `argvus-session`, `argvus-hyprland`, `argvus-portal`, `argvus-lock`, `argvus-boot-splash`, et le greeter (`argvus-greeter`, construit à partir de `argvus/crates/argvus-greeter`) ;
- UI en dehors du workspace : Panneau de configuration, barre des tâches (`argvus-taskbar`), lanceur et Waybar ;
- fournisseurs : affichage, alimentation, pare-feu, notifications ;
- ressources et packaging : apparence, fonds d'écran, polices, icônes, profils.

## Consolidé dans le workspace

Ces dépôts ont été fusionnés dans `de/argvus`. Leur documentation se trouve désormais sous `docs/` du workspace, et leurs dépôts sont conservés uniquement pour l'historique :

| Ancien dépôt | Maintenant | Documentation | | --- | --- | --- |
| `argvus-config` | `argvus config` | [ligne de commande](/docs/reference/command-line/), [fichiers de configuration](/docs/reference/configuration-files/) |
| `argvus-i18n` | `argvus i18n` | [Composant d'internationalisation](/docs/components/argvus-i18n/) |
| `argvus-keys`, `argvus-projects` | `argvus keys`, `argvus projects` | ligne de commande |
| `argvus-terminal` | `argvus terminal` | [Composant Terminal](/docs/components/argvus-terminal/) |
| `argvus-system-monitor` | `argvus system-monitor` | [Composant Moniteur Système](/docs/components/argvus-system-monitor/) |
| `argvus-network` | `argvus network`, `argvus bluetooth` | [Composant Réseau](/docs/components/argvus-network/) |
| `argvus-widget-telemetry` | `argvus widget-telemetry` | ligne de commande |
| `argvus-control-center` | `argvus control-center` (application terminal) | [Composant Centre de Contrôle](/docs/components/argvus-control-center/) |
| `argvus-taskbar-calendar` | `argvus taskbar-calendar` (assistant GTK 4) | [Composant calendrier de la barre des tâches](/docs/components/argvus-taskbar-calendar/) |
| `argvus-removable-devices` | `argvus removable-devices` (assistant GTK 4) | [Composant des périphériques amovibles](/docs/components/argvus-removable-devices/) |
| `argvus-loading-theme` | `argvus loading-theme` (assistant `splash`) | ligne de commande |
| `argvus-tui-terminal` | `argvus tui-terminal` (crate dans l'espace de travail) | ligne de commande |
| `argvus-tui`, `argvus-theme` | Crates partagées `crates/argvus-tui` et `crates/argvus-theme` | notes pour développeurs |
| `argvus-greeter` | Source dans `crates/argvus-greeter`, livrée dans le paquet `argvus` | [Composant greeter](/docs/components/argvus-greeter/) |
| `argvus-themes` | Retiré. Chaque thème officiel est un paquet `argvus-theme-*` à part entière. | [installation des thèmes](/docs/user-guide/appearance/) |

Les notes du développeur concernant les composants consolidés se trouvent dans `docs/internal/<repository>/`.

La langue ne délimite pas la propriété : chaque composant conserve ses aides d'exécution avec son paquet. La seule règle stricte qui traverse les paquets : seul `argvus-config` écrit en dessous de `data/generated/`. Un composant qui doit atteindre une application incapable de lire `config.json` ajoute un adaptateur dans sa propre aide et laisse l'orchestrateur l'appeler après le commit canonique ; il n'écrit jamais de fichier généré. Les CLI publics appartiennent à `/usr/bin` ; les aides internes se trouvent sous `/usr/lib/argvus/<component>` ou dans le répertoire des commandes lorsqu'elles sont nécessaires à l'exécution ; les données indépendantes de l'architecture restent sous `/usr/share/argvus/<component>`.

Voir la [matrice des paquets](/docs/reference/package-matrix/) pour la limite des paquets installés.
