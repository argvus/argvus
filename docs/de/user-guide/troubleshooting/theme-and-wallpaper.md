---
title: Problembehebung bei Designs und Hintergrundbildern
description: Diagnosethema-Verbreitung und erzeugte Erscheinungsdateien.
---
Überprüfen Sie den kanonischen `appearance`-Abschnitt in `$XDG_CONFIG_HOME/argvus/config.json` und inspizieren Sie dann die generierten Dateien unter `$XDG_CONFIG_HOME/argvus/data/generated`. Alte Punktdateien sind nur Migrationseingaben.

`argvus-config` ist der einzige Schreiber unterhalb von `data/generated/`. Um ein defektes Thema oder Hintergrundbild wiederherzustellen, erzeugen Sie die gesamte Generierung aus dem kanonischen Dokument neu und wenden Sie das Ergebnis an, anstatt die generierten Verbraucher direkt zu bearbeiten:

```sh
argvus config validate
argvus config apply
argvus sessionctl reload
```
`argvus theme-switch` (das `argvus config theme-apply` ausführt) und `argvus config accent` sind Orchestratoren: Sie prüfen, sperren, zeigen das Splash an, führen Commit über `argvus-config` durch und gleichen dann die externen Verbraucher ab, die `config.json` nicht lesen können. Sie schreiben niemals `data/generated/`, und die Themen-Transaktion schreibt die GTK-, Speicher- oder `mode.css`-Dateien nicht über separate Skripte. Verwenden Sie `argvus config accent --apply`, um nur die Akzentabstimmung erneut auszuführen.

Offizielle Themenänderungen wenden ihre gespeicherten Erscheinungswerte an und aktualisieren die betroffenen Laufzeitverbraucher. Das Anwenden eines benutzerdefinierten Themas startet die Taskleiste, Hyprland-Neuladen, Hintergrundbild- und Systemsteuerungsdienste neu. Das Umschalten von Sticky oder Spaced wendet die Abstände, Ränder und Taskleistenränder dieses Modus erneut an und startet dann die Taskleiste, Telemetrie- und Hyprland-Neuladungsdienste neu. Dunsts Benachrichtigungsfarben sind die Ausnahme, solange „Nicht stören“ aktiv ist: Ihr Neuladen wird verschoben, bis DND ausgeschaltet wird. Wenn eine andere Oberfläche die alten Farben beibehält, überprüfen Sie, ob der Themenaufbau aktiv ist und ob es das gewählte Thema ist, und führen Sie dann die Neuladungen erneut aus:

```sh
cat "${XDG_CONFIG_HOME:-$HOME/.config}/argvus/data/generated/theme/current/build.toml"
argvus config theme-runtime consumers
```
Jede themenbezogene Datei stammt aus dem Build: Ein Theme, das nicht installiert ist (kein `theme.toml` unter `/usr/share/argvus/themes/` oder `~/.local/share/argvus/themes/`) oder nicht rendert, wird abgelehnt, und das vorherige Theme bleibt bestehen. `argvus theme check DIRECTORY` zeigt die Vorlage und die Zeile jedes Renderfehlers an. Eine Waybar-Leiste lädt ihre Farben nur mit `reload_style_on_change` in ihrem JSONC neu. Wenn ein Schalter scheinbar festhängt, prüfen Sie, ob er noch läuft; er zeichnet sich in `theme-switch.active` auf, neben seinem Sperrmechanismus:

```sh
cat "${XDG_CACHE_HOME:-$HOME/.cache}/argvus/theme-switch.active"
ps -ef | grep -E 'theme-switch|theme-apply'
```
Eine PID im Marker, die noch lebt, bedeutet, dass der Wechsel noch nicht abgeschlossen ist. Ein Wechsel wartet höchstens 60 Sekunden auf das Thema-Lock und auf die Projektion, und 5 Sekunden auf die Reload-Befehle; ein Marker, dessen Prozess beendet ist, ist veraltet und der nächste Wechsel ignoriert ihn. Fügen Sie beim Melden des Problems die Ausgabe der obigen Befehle bei.

Wenn nur der Pre-Login-Greeter weiß ist oder das Fallback-Thema verwendet, vergleichen Sie die private Quelle mit ihrer öffentlichen Projektion:

```sh
uid=$(id -u)
argvus config get /appearance/theme --raw
sudo cat "/var/lib/argvus/greeter/themes/$uid"
sudo cat "/var/lib/argvus/greeter/themes/$uid.accent" 2>/dev/null || true
```
Reparieren Sie die Projektion, ohne die greetd-Konfiguration zu ändern:

```sh
sudo argvus greeter-setup --sync-themes
```
Diese Datei wird zur Vorautorisierung vom Appearance-Orchestrator veröffentlicht und befindet sich außerhalb von `data/generated/`, daher aktualisiert ein einfaches `argvus config apply` sie nicht.

Wenn der Greeter `Invalid color name: 'null'` in einem Kitty-Cache meldet, aktualisieren Sie das `argvus terminal`-Paket und generieren Sie den Cache neu, indem Sie den Greeter erneut öffnen. Ein fehlender kanonischer Akzent ist ein Standard, keine Kitty-Farbe; der aktuelle Terminal-Launcher lehnt JSON `null` und ungültige Legacy-Werte ab, bevor er `active_tab_foreground` schreibt.

Wenn sich das Standard-Terminal nach mehreren Themenänderungen nicht mehr öffnet, prüfen Sie den abgeleiteten Kitty-Cache, anstatt ihn manuell zu bearbeiten:

```sh
argvus config get /appearance/theme --raw
ls -l "${XDG_CACHE_HOME:-$HOME/.cache}/argvus/argvus-terminal"
argvus terminal --apply
argvus terminal --print-config
```
Der obige Befehl liest die kanonische Präferenz. `.active-theme` ist nur eine Kompatibilitätsprojektion. Der Cache wird von `argvus terminal` regeneriert und mit der Themenanwendung synchronisiert. Ein fehlendes oder veraltetes `/usr/lib/argvus/loading-theme/splash` weist auf ein unvollständiges Paket-Update hin; bauen Sie die koordinierten Pakete neu auf und installieren Sie sie, bevor Sie ein Live-Sitzungsergebnis beurteilen.

Wenn Fensterränder die vorherige Themenfarbe beibehalten, bearbeiten Sie die Lua-Datei von Hyprland nicht manuell. Themen- und Akzentänderungen werden von `config.json` projiziert, dann lädt `argvus sessionctl reload` Hyprland neu und wendet die aktiven, inaktiven und Gruppenrandfarben während des Sitzungslebenszyklus erneut an:

```text
config.json → projection plan → Hyprland reload → hyprctl reapply of active/inactive/group borders
```
Die *Einstellungen* für Ränder werden über `argvus config borders` (oder deren `borders-switch.sh`-Bevollmächtigten) im Kontrollzentrum und in der Systemsteuerung geändert; ein Neuladen wendet die festgelegten Farben während des gesamten Sitzungslebenszyklus erneut an und leitet sie nicht neu ab.

Überprüfen Sie den Plan und die effektiven Compositor-Werte:

```sh
jq . "${XDG_STATE_HOME:-$HOME/.local/state}/argvus/config-projection.json"
hyprctl -j getoption general:col.active_border
hyprctl -j getoption general:col.inactive_border
argvus sessionctl reload
```
Wenn `hyprctl` die aktuelle Hyprland-Socket nicht erreichen kann, wird die Randprojektion als fehlgeschlagen gemeldet und abhängige Komponenten werden nicht als erfolgreich neu geladen angesehen. Überprüfen Sie das Benutzer-Sitzungsjournal, bevor Sie es erneut versuchen.
