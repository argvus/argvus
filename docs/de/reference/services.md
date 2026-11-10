---
title: Dienstleistungen
description: ARGVUS systemd- und Benutzerdienste.
---
Das Hauptbenutzerziel ist `argvus-session.target`. Übliche Benutzerdienste umfassen `argvus-taskbar.service`, `argvus-control-panel.service`, `argvus-widget-telemetry.service`, `argvus-hyprland.service`, `argvus-dunst.service`, `argvus-wallpaper.service`, `argvus-hypridle.service`, Zwischenablagedienste und `argvus-session-loading.service`.

Die Firewall ist ein Systemdienst namens `argvus-firewall.service`. Das Kalenderpaket installiert `argvus-taskbar-calendar.service` als Benutzereinheit, und `argvus-clock.service` (standardmäßig deaktiviert) startet die ARGVUS-Uhr nach dem Login verborgen, wenn **Mit der Sitzung starten** aktiviert ist.

Aktive Einheiten überprüfen mit:

```sh
systemctl --user status argvus-session.target
systemctl --user list-units 'argvus-*.service'
```
## Sitzungs-eigene Einheiten

Das Sitzungs-Paket installiert diese Benutzereinheiten unter `/usr/lib/systemd/user` (die Quell-Payload befindet sich unter dem gemeinsamen Konfigurationsbaum des Sitzungs-Pakets):

| Einheit | Rolle |
| --- | --- |
| `argvus-session.target` | Lebenszeitgrenze für die ARGVUS-Grafik-Sitzung. |
| `argvus-session-prepare.service` | Einmalige Startvorbereitung und erzeugte Konfiguration. |
| `argvus-control-panel.service` | Quickshell-Kontrollpanel. |
| `argvus-taskbar.service` | Haupt-ARGBUS-Taskleiste. |
| `argvus-widget-telemetry.service` | Optionale Telemetrie-/Systeminfo-Oberfläche. |
| `argvus-hyprland.service` | Bedarfsorientierte Einmal-Einheit, die `hyprctl reload` ausführt. |
| `argvus-config.service` | Projiziert die kanonische Konfiguration (`argvus config project`); Neuladen führt `argvus sessionctl reload-config-transaction` aus. |
| `argvus-session-reload.service` | Login-Neuladen (`argvus sessionctl reload-desktop --login`): lädt Hyprland nur neu, wenn die Projektion betroffen war. |
| `argvus-ssh-agent.service` | SSH-Agent für die Sitzung. |
| `argvus-secret-service.service` | Startet den Secret Service über `argvus sessionctl start-secret-service`. |
| `argvus-wallpaper.service` | Aktiver Wallpaper-Backend. |
| `argvus-dunst.service` | Dunst-Benachrichtigungsdaemon. |
| `argvus-hypridle.service` | Leerlauf- und Sperrrichtlinie. |
| `argvus-clipboard-text.service` / `argvus-clipboard-image.service` | Zwischenablage-Verlaufsüberwacher. |
| `argvus-keyboard-layout.service` | Tastaturlayout-Benachrichtigungen. |
| `argvus-polkit.service` | PolicyKit-Agent-Integration. |
| `argvus-session-loading.service` | Session-Ladeoverlay vom Compositor-Handoff bis zum Start der Desktop-Komponenten. |

`argvus-blueman-applet.service` wird ebenfalls durch das Sitzungspaket verschickt. Einige Einheiten sind optional oder werden bedingt gestartet; sie sind nicht in jeder Sitzung aktiv.

Um die effektive Definition einer Einheit zu prüfen, anstatt nur ihren Quell-Checkout:

```sh
systemctl --user cat argvus-session.target
systemctl --user status argvus-session.target
journalctl --user -u argvus-session.target -u 'argvus-*.service'
```
Das systemweite `argvus-firewall.service` gehört zu `argvus-firewall`. Das Kalenderpaket liefert `argvus-taskbar-calendar.service` separat als Benutzereinheit. `argvus-clock.service` ist eine Benutzereinheit des `argvus`-Pakets, `WantedBy=graphical-session.target`, aktiviert über die ARGVUS-Uhreinstellungen.

## Sitzungsverwaltung Befehle

Der Befehl `argvus sessionctl` verwaltet den Sitzungslebenszyklus und kann bestimmte Komponenten neu starten, ohne sich abzumelden:

### Sitzungslebenszyklus

```sh
argvus sessionctl start               # Start a new session
argvus sessionctl stop                # Stop the current session
argvus sessionctl restart             # Restart the entire session
argvus sessionctl reload              # Reload configuration and active services
argvus sessionctl status              # Show session status
argvus sessionctl logs                # Display session logs
```
### Komponentenspezifische Neustarts

Starten Sie einzelne Dienste neu, ohne die gesamte Sitzung zu beeinflussen:

```sh
# Desktop surfaces
argvus sessionctl restart waybar
argvus sessionctl restart wallpaper
argvus sessionctl restart shell

# Notifications and switcher
argvus sessionctl restart dunst

# Clipboard and input
argvus sessionctl restart clipboard keyboard-layout

# Bluetooth applet (optional)
argvus sessionctl restart blueman-applet
```
### Umgebungs-Synchronisation

Umgebungsvariablen in die Sitzung importieren:

```sh
argvus sessionctl import-environment
```
Dies synchronisiert Wayland, Hyprland, Qt, Cursor und DBus Variablen zwischen der Shell und systemd Benutzer-Diensten.

## Referenz für Service-Einheiten

Zusätzliche Benutzereinheiten können direkt mit `systemctl --user` verwaltet werden:

```sh
systemctl --user status argvus-session.target
systemctl --user start argvus-session.target
systemctl --user stop argvus-session.target
systemctl --user restart <unit>

# View logs for a specific service
journalctl --user -u argvus-wallpaper.service -b -f
```
Protokolle für das anfängliche Sitzungs-Bootstrap werden gespeichert unter:

```
${XDG_STATE_HOME:-$HOME/.local/state}/argvus/session.log
```
