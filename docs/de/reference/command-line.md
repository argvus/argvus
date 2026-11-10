---
title: Befehlszeile
description: Der argvus-Befehl, seine Unterbefehle, die für Tastenkombinationen vorgesehenen Flags und wo jeder ausgeführt wird.
---
`/usr/bin/argvus` ist der öffentliche Einstiegspunkt von ARGVUS. Es ist ein Rust-Programm, das aus dem `argvus-cli`-Crate erstellt wurde. Jeder Unterbefehl wird entweder innerhalb dieses Programms ausgeführt, startet ein Hilfsprogramm aus dem Befehlsverzeichnis oder führt ein verpacktes Skript aus.

Führen Sie `argvus --help` aus, um die mit Ihrer Version installierten Unterbefehle aufzulisten. Die Unterbefehle werden ohne das führende `--` geschrieben. Die `--`-Formen werden nur beibehalten, wenn eine Tastenkombination oder ein Desktop-Eintrag sie verwendet.

## Wo jeder Befehl ausgeführt wird

- **Im Prozess:** der Befehl ist eine Bibliothek des Arbeitsbereichs und wird innerhalb von `argvus` ausgeführt. Es existiert keine separate ausführbare Datei. Diese sind `config`, `theme`, `i18n`, `keys`, `gpg`, `projects`, `terminal`, `system-monitor`, `network`, `bluetooth`, `focus status`, `widget-telemetry`, `tui-terminal`, `sensors` und `setup`.
- **Helfer:** eine separate ausführbare Datei in `/usr/lib/argvus/commands/`, installiert durch das `argvus`-Paket. Dies sind `control-center` (Terminalanwendung), `argvus-du` (gestartet mit `argvus du`) und die GTK 4-Anwendungen `taskbar-calendar`, `removable-devices`, `focus` und `clock`.
- **Private ausführbare Datei:** eine ausführbare Datei, die ein anderes Paket in `/usr/lib/argvus/commands/` installiert. `argvus` startet es nur. Dazu gehören `session`, `start`, `tty`, `sessionctl`, `launcher`, `snippets`, `ssh`, `git`, `app-profiles-yazi-keymap`, `notifications`, `displayctl`, `firewall`, `accounts`, `taskbar-actions` und die `greeter-*`-Einstiegspunkte.
- **Skript:** ein gepacktes Shell-Skript, das mit `/bin/sh` ausgeführt wird. Dies sind die unten aufgeführten Hyprland-Skriptbefehle.

`ARGVUS_COMMANDS_DIR` ändert das Befehlsverzeichnis für Tests und `DESTDIR`-Prüfungen. Eine private ausführbare Datei, die dort nicht gefunden wird, wird unter `PATH` gesucht.

## Unterbefehle

| Befehl | Läuft | Notizen |
| --- | --- | --- |
| `argvus config ...` | In Bearbeitung | Konfigurationsmodell und Werkzeuge. Ebenfalls erreichbar über `theme-switch` und `effects-toggle`. |
| `argvus theme ...` | In Bearbeitung | `apply ID` ist der Themenumschalter (derselbe wie `theme-switch`). Themen-Erstellungen: `list` die installierten Themen, `check DIRECTORY` ein Thema gegen die installierten Vorlagen, `render ID [--variant sticky\|spaced] [--accent #RRGGBB] [--output DIRECTORY]`. Siehe [Aussehen und Themen](/docs/developer-guide/subsystems/appearance-and-themes/)]. |
| `argvus i18n ...` | In Bearbeitung | Übersetzungen: `get DOMAIN KEY [name=value...]`, `locale`, `list`, `validate [LOCALE]`. |
| `argvus keys ...` | In Bearbeitung | SSH-Schlüsselmanager. `gpg` ist ein Alias für die GPG-Unterbefehle. |
| `argvus gpg ...` | In Bearbeitung | GPG-Schlüssel: `list`, `status`, `generate`, `export-public`, `export-private`, `import`, `import-private`, `delete`, `edit`, `keyserver`. |
| `argvus projects ...` | In Bearbeitung | Projektwähler. |
| `argvus terminal ...` | In Bearbeitung | Das ARGVUS-Terminalprofil. |
| `argvus system-monitor ...` | In Bearbeitung | Der btop-Systemmonitor; `--gpu` wählt nur GPU-Boxen, und `--panel [--cpu|--mem|--gpu]` bettet ihn in die ARGVUS-TUI-Shell ein. |
| `argvus du` | Helfer (Terminalanwendung) | Öffnet den ARGVUS-Disk-Usage-Analyzer in einem eigenen Kitty-Fenster und führt ncdu aus. |
| `argvus network ...` | In Bearbeitung | NetworkManager-Status und -Steuerung (`networkctl` bleibt als Alias erhalten). |
| `argvus bluetooth ...` | In Bearbeitung | Bluetooth-Status und -Steuerung (`bluetoothctl` bleibt als Alias erhalten). |
| `argvus widget-telemetry block ...` | In Bearbeitung | Der Waybar-Telemetrieblock. Dies ist der heiße Pfad der Leiste. |
| `argvus widget-telemetry toggle ...` | In Bearbeitung | Schaltet die Telemetrieblöcke ein und aus sowie deren Reihenfolge. |
| `argvus game-snake` | Private ausführbare Datei | Das Terminal-Snake-Spiel, vom unabhängigen `argvus-game-snake` Projekt, ein optionales Paket. Außerhalb eines Terminals öffnet es eines. |
| `argvus control-center [page]` | Helfer (Terminalanwendung) | Öffnet das Kontrollzentrum oder fokussiert das Fenster, das bereits geöffnet ist. |
| `argvus welcome [--if-needed]` | Dispatcher → Kontrollzentrum | Öffnet die integrierte Willkommensseite im Kontrollzentrum. `--if-needed` öffnet sich einmal, es sei denn, die pro-Benutzer-Fortschrittsaufzeichnungen zeigen, dass die Willkommensseite bereits angezeigt, abgeschlossen oder geschlossen wurde. |
| `argvus taskbar-calendar [args]` | GTK 4-Anwendung | Standardmäßig `toggle`. |
| `argvus removable-devices [args]` | GTK 4-Anwendung | Standardmäßig `menu`. |
| `argvus focus [--toggle] [--start\|--pause] [--x X --y Y] [--bar-bottom PX --bar-right PX]` | GTK 4-Anwendung | ARGVUS Fokus-Popup. Eine Instanz pro Sitzung. |
| `argvus focus status --waybar [--follow]` | In Bearbeitung | Gibt den Fokus-Taskleistenindikator als Waybar-JSON aus; `--follow` überträgt Änderungen, ohne GTK zu laden. |
| `argvus clock [--toggle] [--background] [--page PAGE] [--x X --y Y] [--bar-bottom PX --bar-right PX]` | GTK 4 Anwendung | ARGVUS Clock Popup. Eine Instanz pro Sitzung; `--background` hält Alarme aktiv, ohne ein Popup anzuzeigen. |
| `argvus loading-theme ...` | Verpackt `splash` bei `/usr/lib/argvus/loading-theme/splash` | Die Overlay-Funktion für direkte Sitzungsneuläufe; ein Themenwechsel verwendet sie nicht. |
| `argvus session`, `start`, `tty`, `sessionctl` | Private ausführbare Dateien | Sitzungs-Einstiegspunkte und der Sitzungscontroller. |
| `argvus greeter`, `greeter-session`, `greeter-tui`, `greeter-setup` | Private ausführbare Dateien | Greeter-Einstiegspunkte. |
| `argvus accounts`, `firewall` | Private ausführbare Dateien | Konten- und Firewall-Tools. |
| `argvus launcher`, `snippets`, `ssh`, `git` | Private ausführbare Dateien | Launch- und Auswahlwerkzeuge. |
| `argvus tui-terminal` | Im Prozess | Terminal-Wrapper für TUI-Anwendungen. |
| `argvus sensors` | Im Prozess | Live-Temperaturübersicht von `sensors -j`. |
| `app-profiles-yazi-keymap` | Private ausführbare Datei | Keymap-Helfer. |
| `argvus notifications`, `displayctl`, `taskbar-actions` | Private ausführbare Dateien | Benachrichtigungen, Anzeige- und Taskleistenaktionen. |
| `argvus setup [options]` | Im Prozess | Kopiert verpackte Standardwerte in die Benutzerkonfiguration. Siehe unten. |

Argumente werden unverändert weitergegeben, einschließlich Leerzeichen und leerer Werte. Standard-Eingabe, Ausgabe, Fehler, Signale und der Exit-Status gehören dem Befehl, der ausgeführt wird. Ein unbekannter Name schlägt mit einem Hinweis auf, `argvus --help` auszuführen. Ein Befehl, der im Prozess ausgeführt wird, verweigert ein Argument, das kein gültiges UTF-8 ist (Exit-Status 2), anstatt einen veränderten Wert weiterzugeben; Befehle, die ein separates Programm ausführen, erhalten die Bytes unverändert.

## Flags und Aliase

Diese Namen existieren, damit Tastenkombinationen, Desktop-Einträge und ältere Skripte weiterhin funktionieren. Verwenden Sie in neuem Code vorzugsweise die Unterbefehl-Form.

| Flag oder Alias | Entsprechung |
| --- | --- |
| `--control-center`, `--about`, `--default-apps` | `control-center` (mit `about` und `apps` Seiten) |
| `--displays`, `--monitors` Seite | `control-center displays` |
| `--network-interfaces` | `control-center network interfaces` |
| `--system-monitor`, `--terminal`, `--game-snake` | `system-monitor`, `terminal`, `game-snake` |
| `--calendar`, `--removable-devices`, `--storage` | `taskbar-calendar`, `removable-devices` |
| `--focus`, `--clock` | `focus`, `clock` |
| `--spf`, `--superfile` | der Dateimanager (`spf`) |
| `--yazi`, `--yazy` | der Dateibrowser (`yazi`) |
| `--setup` | `setup` |
| `--start-desktop` | startet `tty` (es werden keine Argumente weitergeleitet) |
| `--version`, `-V`, `version` | druckt `ARGVUS <version>` |
| `help`, `--help`, `-h` | druckt die Verwendung |
| `networkctl`, `bluetoothctl` | `network`, `bluetooth` |
| `widget-telemetry-toggle`, `widget-telemetry-block` | `widget-telemetry toggle`, `widget-telemetry block` |
| `theme-switch` | `config theme-apply` |
| `effects-toggle` | `config effects` |

Das Kontrollzentrum akzeptiert Legacy-Seitenflags als erstes Argument und ordnet sie Seitennamen zu. Beispiele: `--apps` wird zu `apps`, `--keyboard-layout` wird zu `locale` und `--hostname` wird zu `system`.

## Hyprland-Skriptbefehle

Diese führen verpackte Skripte aus und werden von der Hyprland-Konfiguration verwendet:

`cheatsheets-hypr`, `cheatsheets-launcher`, `scratchpad`, `hypr-screenshot`, `spaces-switch`, `emoji-picker`, `toggle-sidebar`, `weather-location`, `layout-mode-menu`, `brightness-switch`, `toggle-mode`, `idle-timeout`, `keep-awake`, `hypr-power-menu`.

Ein Skript in der Benutzerkonfiguration überschreibt das verpackte Skript. Die Suchreihenfolge ist dieselbe wie `_config_path` in `hyprland.lua`: die Kopie der Benutzerkonfiguration, dann die ARGVUS-Datenkopie, dann die Statuskopie, dann die generierte Kopie, und erst danach die verpackte Datei unter `/usr/share/argvus`.

## Beispiele

```sh
argvus control-center displays
argvus taskbar-calendar toggle
argvus widget-telemetry block header system
argvus widget-telemetry toggle status
argvus i18n get widget-telemetry label.device
argvus config get /appearance/theme
argvus network status
argvus bluetooth status
argvus gpg list
```
## Einrichtung

`argvus setup` kopiert die vorgefertigten Standardwerte von `/usr/share/argvus/<app>` in `~/.config/argvus/data/<app>`, sodass Sie sie anpassen können. Es bearbeitet niemals die vorgefertigten Dateien.

```sh
argvus setup --copy <app>          # copy one packaged app
argvus setup --copy-all            # copy every packaged app directory
argvus setup --copy <app> --force  # back up the existing destination first
argvus setup --copy <app> --dry-run
```
Ein Ziel, das bereits existiert, wird gemeldet und unangetastet gelassen, es sei denn, `--force` wird angegeben. Mit `--force` wird das bestehende Verzeichnis vor dem Kopieren in `<name>.bak-<timestamp>` umbenannt.

Die `argvus`-App ist speziell: Sie kopiert den Inhalt ihres gepackten Verzeichnisses in das Konfigurationsstammverzeichnis. Da das Konfigurationsstammverzeichnis bereits existiert, wenn das Setup ausgeführt wird, benötigt `--copy argvus` im aktuellen Code `--force`.

Das Setup überprüft jede Option, bevor es irgendetwas ändert. Ungültige App-Namen (leer, beginnend mit `.` oder enthaltend `/`) werden abgelehnt.

## Sitzung und Neustart

`SUPER + SHIFT + R` führt `argvus sessionctl reload` aus. Der Sitzungscontroller wendet die generierte Konfiguration an und startet nur die Komponenten neu, die er besitzt. Er kopiert `/usr/share/argvus` nicht in das Home-Verzeichnis und ersetzt auch keine fehlerhafte JSON-Datei.

## Nachtlicht

Öffnen Sie **Desktop & Eingabe > Anzeigen > Nachtlicht** oder führen Sie `argvus control-center displays night-light` aus. Konfigurieren Sie Temperatur, manuelle Aktivierung und feste lokale Zeitpläne über `argvus config display night-light`. Siehe [Nachtlicht](/docs/user-guide/night-light/) für Befehle, Status und Laufzeitanforderungen.

`argvus tui-terminal --frame TITLE FOOTER COMMAND [ARG...]` führt einen interaktiven Befehl innerhalb des gemeinsamen ARGVUS-Terminalrahmens aus. Strg+E schließt den Rahmen; der Aufrufer liefert die lokalisierte Fußzeile. Verwenden Sie dies innerhalb eines Terminals oder als Befehl, der an `argvus tui-terminal --term kitty -- ...` übergeben wird.

`argvus tui-terminal --term kitty --profile ai-agents` verwendet eine dedizierte Kitty-Konfiguration mit versteckten Tabs und nur Zwischenablagenkürzeln. Der Aufrufer liefert den Befehl und den Titel; die Fensterplatzierung wird durch die Hyprland-Regel für die Klasse `argvus-ai` gehandhabt.
