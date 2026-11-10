---
title: Surfaces de coquille
description: Barre des tâches, panneau de configuration et propriété de la télémétrie.
---
`argvus-waybar` fournit le `/usr/bin/waybar` corrigé. `argvus-taskbar` possède les actions de la barre des tâches et l'UX ; le JSONC et le CSS de la barre des tâches qu'il fournit sont des projections de `config.json`, et `argvus-config` écrit la couche active sous `data/generated/waybar/` ainsi qu'une copie miroir sous `data/waybar/`. `argvus-control-panel` possède les cartes Quickshell. `argvus-widget-telemetry` possède la barre de télémétrie optionnelle et ses sémantiques de bloc ; son JSONC et son CSS sont écrits dans `data/waybar/` à partir des valeurs par défaut empaquetées lors d'un changement d'apparence, après quoi `argvus-config` réécrit uniquement les blocs `ARGVUS_TELEMETRY_*` délimités et les blocs de polices. `argvus-session` possède leurs services utilisateur.

Il n'existe actuellement aucun package `argvus-shell` ni `argvus-shell.service` dans ce modèle d'exécution.
