---
title: Fichiers de configuration
description: Configuration ARGVUS pour l'utilisateur, le système, par défaut du package et générée.
---

| Couche | Emplacement | Signification |
| --- | --- | --- |
| Système | `/etc/argvus/` | Configuration administrateur pour les services et fournisseurs. |
| Paramètres par défaut du paquet | `/usr/share/argvus/` | Paramètres par défaut des composants en lecture seule et ressources d'exécution. |
| Utilisateur | `$XDG_CONFIG_HOME/argvus/config.json` | Préférences portables canoniques. |
| Données gérées | `$XDG_CONFIG_HOME/argvus/data/` | Données des composants, fichiers de compatibilité et métadonnées internes. |
| Généré | `$XDG_CONFIG_HOME/argvus/data/generated/` | Projections de `config.json`, écrites uniquement par `argvus-config`. Ne jamais modifier, ne jamais copier ailleurs comme configuration, et ne jamais relire comme source. |
| Exécution | `$XDG_RUNTIME_DIR/` et `$XDG_CACHE_HOME/` | Sockets, verrous et état transitoire. |

Le thème actif, l'accent, le fond d'écran personnalisé et l'état des effets sont des états logiques de l'utilisateur. Le modèle logique géré est documenté dans [Canonical configuration](/docs/reference/argvus-config/). Les fichiers générés Hyprland, GTK, Qt6ct, Waybar, Quickshell, Dunst, Hyprlock, Yazi, Superfile, terminal, police et périphériques amovibles ne doivent pas être considérés comme des sources de vérité indépendantes.

`argvus-config` reflète également la couche de thème dans `data/waybar/argvus-taskbar.{jsonc,css}` et `data/waybar/argvus-widget-telemetry.{jsonc,css}`. Ces copies sont remplacées par les valeurs par défaut packagées lors d'un changement d'apparence, puis leurs blocs gérés délimités sont réécrits, donc traitez-les comme dérivés plutôt que comme des fichiers maintenus manuellement. La configuration des applications natives sous `$XDG_CONFIG_HOME/<app>` est différente : c'est une cible d'adaptateur écrite par les orchestrateurs d'apparence, jamais par le projecteur.

## Exception à la règle du rédacteur unique

`argvus config` est le seul rédacteur de `data/generated/`. Une seule exception existe pour la migration : lorsqu'une copie utilisateur d'un fichier empaqueté est créée, un fichier qu'une version plus ancienne a laissé sous `data/generated/` est déplacé vers son emplacement utilisateur (`data/`), afin que l'utilisateur conserve ses modifications. Le déplacement ne se produit que pour les fichiers que la version actuelle ne génère pas. Ne vous fiez pas à ce chemin pour les nouveaux fichiers : ajoutez un fichier généré dans la projection, pas ici.
