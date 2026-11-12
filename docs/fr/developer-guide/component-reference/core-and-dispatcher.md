---
title: Noyau et répartiteur
description: L'espace de travail argvus, son répartiteur et les crates Rust partagées.
---
`argvus` est l'espace de travail Cargo à `de/argvus` et le paquet qui fournit `/usr/bin/argvus`. La crate `argvus-cli` est le répartiteur : elle lit le premier argument et exécute soit une commande de bibliothèque en processus, soit un assistant GTK 4 ou un exécutable privé, soit un script empaqueté. La liste complète des sous-commandes se trouve dans la [référence de ligne de commande](/docs/reference/command-line/).

L'espace de travail contient également les crates partagées par les commandes :

- `argvus-config` : le magasin de configuration canonique et le seul écrivain de l'arborescence générée (`data/generated/`). C'est également une bibliothèque utilisée en processus par `argvus config`, `theme-switch` et `effects-toggle`.
- `argvus-i18n` et `argvus-i18n-core` : les catalogues dans `resources/locales/`, leur validation, et le module QML `org.argvus.i18n`.
- `argvus-tui` et `argvus-theme` : interface utilisateur terminale partagée et chargement du thème.
- `argvus-runtime` : les messages traduits et les assistants de processus partagés par les commandes.

`argvus-session` reste un référentiel séparé. Il fournit les points d'entrée de session (`session`, `start`, `tty`) et les unités de cycle de vie, et `argvus sessionctl` atteint son contrôleur.
