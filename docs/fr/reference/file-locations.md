---
title: Emplacements des fichiers
description: Fichiers et répertoires ARGVUS importants installés.
---

- `/usr/bin/argvus*` — commandes publiques.
- `/usr/share/argvus/hyprland/` — configuration et aides Hyprland.
- `/usr/share/argvus/control-panel/` — QML du panneau de contrôle et scripts.
- `/usr/share/argvus/taskbar/` — configuration de la barre des tâches et scripts.
- `/usr/share/argvus/lock/` — configuration de Hyprlock.
- `/usr/share/argvus/portal/` — fichiers d'intégration du portail.
- `/usr/share/backgrounds/argvus/` — ressources de fonds d'écran.
- `/etc/argvus/` — configuration du fournisseur système.
- `/etc/argvus/greeter.toml` — configuration de l'écran de bienvenue.
- `/etc/argvus/removable-devices/config.json` — paramètres système par défaut pour le stockage amovible.
- `$XDG_CONFIG_HOME/argvus/config.json` — préférences portables canoniques.
- `$XDG_CONFIG_HOME/argvus/data/` — données des composants gérés, sauvegardes, verrous et sorties générées.
- `$XDG_CONFIG_HOME/waybar/argvus-taskbar.{jsonc,css}` — remplacements facultatifs complets de la barre de tâches native Waybar.
- `$XDG_CONFIG_HOME/argvus/data/taskbar/argvus-taskbar.{jsonc,css}` — gestion des copies de la barre des tâches ARGVUS. 
- `$XDG_CONFIG_HOME/argvus/data/generated/waybar/` — profils Waybar générés, incluant `argvus-taskbar.css` et `argvus-widget-telemetry.{jsonc,css}` ; ceux-ci prennent le pas sur les copies ci-dessus. 
- `$XDG_CONFIG_HOME/argvus/data/generated/waybar/argvus-widget-telemetry.{jsonc,css}` — également reflété sous `data/waybar/`, qui est remplacé par le défaut fourni lors d’un changement d’apparence, puis a ses blocs `ARGVUS_TELEMETRY_*` et de polices réécrits. 
- `$XDG_CONFIG_HOME/argvus/data/generated/yazi/` — arbre de configuration Yazi projeté, avec un mécanisme de repli conscient des variantes pour les thèmes qui ne fournissent pas de version Yazi. 
- `$XDG_CONFIG_HOME/argvus/data/generated/superfile/` — ensemble de thèmes Superfile projeté. 
- `$XDG_CONFIG_HOME/argvus/data/generated/terminal/` — profils de terminal projetés.
- `$XDG_CONFIG_HOME/argvus/data/generated/qt6ct/` — palette Qt6ct projetée. 
- `$XDG_CONFIG_HOME/argvus/data/generated/gtk/` — fichiers de thème GTK projetés. 
- `$XDG_CONFIG_HOME/argvus/data/rofi/` — copies utilisateur de la configuration Rofi, des thèmes et du mode. ARGVUS crée chaque copie lorsqu'il modifie le fichier pour la première fois ; les fichiers inchangés sont lus depuis `/usr/share/argvus/launcher/config/`. 
- `$XDG_CONFIG_HOME/argvus/data/generated/removable-devices/theme.css` — feuille de style des périphériques amovibles projetée. 
- `$XDG_CONFIG_HOME/argvus/data/focus/settings.json`, `$XDG_DATA_HOME/argvus/focus/history.json`, `$XDG_STATE_HOME/argvus/focus/session.json` — paramètres de Focus ARGVUS, historique de session et point de contrôle de session en cours. 
- `$XDG_RUNTIME_DIR/argvus/focus/status.json` — statut éphémère de Focus ARGVUS consommé par la barre des tâches ; supprimé lorsque Focus se ferme.
- `$XDG_CONFIG_HOME/argvus/data/clock/settings.json`, `$XDG_DATA_HOME/argvus/clock/alarms.json`, `$XDG_STATE_HOME/argvus/clock/timers.json` — Paramètres de l'horloge ARGVUS et horloges mondiales, alarmes et point de contrôle du minuteur/chronomètre. 
- `$XDG_CONFIG_HOME/argvus/data/generated/theme/current/time/theme.css` — couleurs de l'ARGVUS Focus et de l'ARGVUS Clock (consommateur de thème `time`). 
- `/usr/share/argvus/sounds/` — sons de l'ARGVUS Focus et de l'ARGVUS Clock (`argvus-chime.oga`, `argvus-beacon.oga`, `argvus-bell.oga`). 
- `$XDG_CONFIG_HOME/argvus/data/generated/theme/` — constructions de thème (`builds/<id>-<variant>-<hash>/`), écrites par `argvus theme render` et par chaque changement de thème et d’accent, et le lien symbolique `current` vers la construction du thème actif. 
- `$XDG_CONFIG_HOME/argvus/data/hypr/` — surcharges Lua de l’utilisateur Hyprland et projections natives de Hyprland.
- `$XDG_CONFIG_HOME/hypr/hyprland.lua` — configuration complète native optionnelle de Hyprland sélectionnée par `argvus start` ; ce n'est pas une superposition incrémentale.
- `$XDG_STATE_HOME/argvus/config-projection.json` — manifeste de projection : génération, hachage de la configuration effective, sections modifiées et travail en cours à l'exécution. Lu par `argvus sessionctl reload` ; jamais une source de vérité.
- `$XDG_STATE_HOME/argvus/widget-telemetry-blocks` — préférences du bloc de télémétrie legacy. Entrée pour migration uniquement ; les valeurs canoniques se trouvent dans `config.json` et ce fichier n'est plus écrit.
- `$XDG_STATE_HOME/argvus/session.log` — journal de session graphique (normalement `~/.local/state/argvus/session.log`).
- `/usr/share/wayland-sessions/argvus.desktop` — entrée de session graphique.
- `/usr/share/plymouth/themes/argvus/` — ressources du splash d'amorçage emballées.

Les contenus exacts sont spécifiques au paquet ; voir la référence des composants pour développeurs et le manifeste du paquet concerné.
