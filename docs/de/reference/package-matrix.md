---
title: Paketmatrix
description: Verantwortlichkeiten des ARGVUS-Pakets.
---

| Paket | Verantwortung |
| --- | --- |
| `argvus` | Master-Paket: Dispatcher, In-Prozess-Befehle (Konfiguration, Übersetzungen, Schlüssel, Projekte, Terminal, Systemmonitor, Netzwerk, Bluetooth, Telemetrie, TUI-Terminal, Einrichtung), die gemeinsamen Daten und die i18n-Kataloge. Jedes Komponentenpaket hängt davon ab. |
| `argvus` (Hilfsprogramme) | Hilfsprogramme im gleichen Paket: das Kontrollzentrum und der Festplattennutzungs-Analysator (Terminal-Anwendungen) sowie die GTK 4-Anwendungen für die Aufgabenleisten-Kalender, Wechselgeräte, das Lade-Overlay, ARGVUS Focus und ARGVUS Clock. |
| `argvus` (Begrüßer) | Login-Greeter für greetd, im gleichen Paket ausgeliefert und aus `crates/argvus-greeter` gebaut. |
| `argvus-desktop` | Desktop- und Alltagsanwendungsprofil, das vom Master-Paket `argvus` benötigt wird. Es hängt nicht von `argvus` ab, daher bleibt die Abhängigkeit einseitig. |
| `argvus-devtools`, `argvus-pentest` | Optionale unabhängige Metapakete für Entwicklungstools und autorisierte Sicherheitsüberprüfung. Das Hauptpaket installiert diese Profile nicht. |
| `argvus-session` | Sitzungseintrittspunkte und Benutzerlebenszyklus. |
| `argvus-hyprland` | Hyprland-Konfiguration. |
| `argvus-control-panel` | Quickshell-Kontrollpanel. |
| `argvus-taskbar`, `argvus-waybar` | Waybar-Konfiguration und gepatchtes Binary. |
| `argvus-branding` | Logo- und Wortzeichen-Assets. |
| `argvus-appearance`, `argvus-wallpapers`, `argvus-fonts`, `argvus-icons` | Erscheinungszustand und visuelle Assets. |
| `argvus-display`, `argvus-power`, `argvus-notifications` | Desktop-Anbieter. |
| `argvus-lock`, `argvus-boot-splash` | Sperrbildschirm und Boot-Splash. |
| `argvus-firewall` | Firewall, mit eigenen Berechtigungen. |
| `argvus-launcher` | Launcher-Konfiguration und Einstiegspunkte. |
| `argvus-app-profiles` | Nur-Datenprofile für gebündelte Terminal-Anwendungen. |
| `argvus-portal` | XDG-Portal und Wayland/DBus-Standards. |
| `argvus-theme-*` | Ein optionales Paket pro offizieller Themenfamilie. |
| `argvus-game-snake`, `argvus-games` | Optional. Snake ist ein unabhängiges Projekt; `argvus-games` gruppiert die optionalen Spiele. |

## Ersetzte Pakete

`argvus` deklariert `replaces`, `conflicts` und `provides` für die Pakete, die es übernommen hat, sodass ein Upgrade sie entfernt und deren Dateien nicht als Waisen zurückbleiben:

`argvus-config`, `argvus-i18n` (mit `argvus-language-en-us` und `argvus-language-pt-br`), `argvus-keys`, `argvus-projects`, `argvus-terminal`, `argvus-system-monitor`, `argvus-network`, `argvus-widget-telemetry` und `argvus-du`. `argvus` ersetzt auch `argvus-control-center`, `argvus-taskbar-calendar`, `argvus-loading-theme`, `argvus-removable-devices` und die früheren gesplitteten Pakete `argvus-gtk` und `argvus-greeter`, die weiterhin bereitgestellt werden, damit Pakete, die von ihnen abhängen, weiterhin aufgelöst werden. Der Kontomanager (`argvus accounts`) wird aus `crates/argvus-accounts` gebaut und als `/usr/lib/argvus/commands/accounts` installiert, mit seiner Polkit-Regel. Der Festplattennutzungsanalysator wird aus `crates/argvus-du` gebaut und als `/usr/lib/argvus/commands/argvus-du` installiert.

`/usr/bin/argvus-terminal` bleibt als Kompatibilitätsstarter für `argvus terminal` installiert. Hyprlands Standardterminal, der Control-Center-Katalog und gespeicherte Standardanwendungen nennen diesen Befehl weiterhin.

Die entfernten Pakete `argvus-pwvucontrol` und `argvus-snappy-switcher` werden von keinem Paket bereitgestellt.

Keine Installationsreihenfolge ist unter den ARGVUS-Komponentenpaketen erforderlich: `argvus` ist ihr Hauptpaket, und jede Komponente hängt davon ab. Die Installation von `argvus` installiert auch das `argvus-desktop`-Profil; `argvus-devtools` und `argvus-pentest` bleiben optional. Die Anwesenheit eines Pakets in dieser Tabelle bedeutet nicht, dass es das gesamte Feature enthält: Zum Beispiel kombiniert die Taskleiste `argvus-waybar`, `argvus-taskbar`, `argvus-session` und Domain-Anbieter.
