---
title: Dateispeicherorte
description: Wichtige installierte ARGVUS-Dateien und -Verzeichnisse.
---

- `/usr/bin/argvus*` — öffentliche Befehle.
- `/usr/share/argvus/hyprland/` — Hyprland-Konfiguration und Hilfsprogramme.
- `/usr/share/argvus/control-panel/` — Control Panel QML und Skripte.
- `/usr/share/argvus/taskbar/` — Taskleisten-Konfiguration und Skripte.
- `/usr/share/argvus/lock/` — Hyprlock-Konfiguration.
- `/usr/share/argvus/portal/` — Portal-Integrationsdateien.
- `/usr/share/backgrounds/argvus/` — Hintergrundbild-Ressourcen.
- `/etc/argvus/` — Systemanbieter-Konfiguration.
- `/etc/argvus/greeter.toml` — Begrüßungsbildschirm-Konfiguration.
- `/etc/argvus/removable-devices/config.json` — Systemstandards für Wechseldatenträger.
- `$XDG_CONFIG_HOME/argvus/config.json` — Kanonische tragbare Einstellungen.
- `$XDG_CONFIG_HOME/argvus/data/` — Verwaltene Komponentendaten, Backups, Sperren und erzeugte Ausgaben.
- `$XDG_CONFIG_HOME/waybar/argvus-taskbar.{jsonc,css}` — Optionale vollständige native Waybar-Taskleisten-Überschreibungen.
- `$XDG_CONFIG_HOME/argvus/data/taskbar/argvus-taskbar.{jsonc,css}` — verwaltete ARGVUS-Taskleiste-Kopien.  
- `$XDG_CONFIG_HOME/argvus/data/generated/waybar/` — generierte Waybar-Profile, einschließlich `argvus-taskbar.css` und `argvus-widget-telemetry.{jsonc,css}`; diese haben Vorrang vor den oben genannten Kopien.  
- `$XDG_CONFIG_HOME/argvus/data/generated/waybar/argvus-widget-telemetry.{jsonc,css}` — ebenfalls unter `data/waybar/` gespiegelt, das bei einer Erscheinungsänderung aus dem gelieferten Standard ersetzt wird und dann seine `ARGVUS_TELEMETRY_*`- und Schriftblocke umgeschrieben bekommt.  
- `$XDG_CONFIG_HOME/argvus/data/generated/yazi/` — projizierter Yazi-Konfigurationsbaum, mit einer variantenbewussten Fallback-Lösung für Themes, die keinen Yazi-Stil liefern.  
- `$XDG_CONFIG_HOME/argvus/data/generated/superfile/` — projiziertes Superfile-Theme-Set.  
- `$XDG_CONFIG_HOME/argvus/data/generated/terminal/` — projizierte Terminal-Profile.
- `$XDG_CONFIG_HOME/argvus/data/generated/qt6ct/` — prognostizierte Qt6ct-Palette.  
- `$XDG_CONFIG_HOME/argvus/data/generated/gtk/` — prognostizierte GTK-Theme-Dateien.  
- `$XDG_CONFIG_HOME/argvus/data/rofi/` — Benutzerkopien der Rofi-Konfiguration, Themes und Modi. ARGVUS erstellt jede Kopie, wenn die Datei erstmals geändert wird; unveränderte Dateien werden von `/usr/share/argvus/launcher/config/` gelesen.  
- `$XDG_CONFIG_HOME/argvus/data/generated/removable-devices/theme.css` — prognostizierte Stylesheet-Datei für Wechseldatenträger.  
- `$XDG_CONFIG_HOME/argvus/data/focus/settings.json`, `$XDG_DATA_HOME/argvus/focus/history.json`, `$XDG_STATE_HOME/argvus/focus/session.json` — ARGVUS-Fokus-Einstellungen, Sitzungshistorie und aktueller Sitzungs-Checkpoint.  
- `$XDG_RUNTIME_DIR/argvus/focus/status.json` — kurzlebiger ARGVUS-Fokus-Status, der von der Taskleiste verwendet wird; wird entfernt, wenn Focus beendet wird.
- `$XDG_CONFIG_HOME/argvus/data/clock/settings.json`, `$XDG_DATA_HOME/argvus/clock/alarms.json`, `$XDG_STATE_HOME/argvus/clock/timers.json` — ARGVUS-Uhreinstellungen und Weltuhren, Alarme sowie Timer/Stoppuhr-Checkpoint.  
- `$XDG_CONFIG_HOME/argvus/data/generated/theme/current/time/theme.css` — Farben von ARGVUS Focus und ARGVUS Clock (Theme-Verbraucher `time`).  
- `/usr/share/argvus/sounds/` — Sounds von ARGVUS Focus und ARGVUS Clock (`argvus-chime.oga`, `argvus-beacon.oga`, `argvus-bell.oga`).  
- `$XDG_CONFIG_HOME/argvus/data/generated/theme/` — Theme-Builds (`builds/<id>-<variant>-<hash>/`), geschrieben von `argvus theme render` und bei jedem Themewechsel und Akzentänderung, sowie der `current`-Symlink zum Build des aktiven Themes.  
- `$XDG_CONFIG_HOME/argvus/data/hypr/` — Benutzerdefinierte Hyprland-Lua-Overrides und native Hyprland-Projektionen.
- `$XDG_CONFIG_HOME/hypr/hyprland.lua` — optionale vollständige native Hyprland-Konfiguration, ausgewählt von `argvus start`; es ist kein inkrementelles Overlay.  
- `$XDG_STATE_HOME/argvus/config-projection.json` — Projektionsmanifest: Generierung, Hash der effektiven Konfiguration, geänderte Abschnitte und ausstehende Laufzeitarbeiten. Gelesen von `argvus sessionctl reload`; niemals eine Quelle der Wahrheit.  
- `$XDG_STATE_HOME/argvus/widget-telemetry-blocks` — Legacy-Telemetrie-Blockpräferenzen. Nur Migrationsinput; kanonische Werte befinden sich in `config.json` und diese Datei wird nicht mehr geschrieben.  
- `$XDG_STATE_HOME/argvus/session.log` — Protokoll der grafischen Sitzung (normalerweise `~/.local/state/argvus/session.log`).  
- `/usr/share/wayland-sessions/argvus.desktop` — Eintrag der grafischen Sitzung.  
- `/usr/share/plymouth/themes/argvus/` — Verpackte Boot-Splash-Assets.  

Exakte Inhalte sind paketabhängig; siehe die Entwicklerkomponentenreferenz und das relevante Paketmanifest.
