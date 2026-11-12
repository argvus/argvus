---
title: ARGVUS-Konfiguration
description: Der logische JSON-Konfigurationsvertrag, der von ARGVUS verwendet wird.
---
ARGVUS speichert verwaltete Benutzerpräferenzen in modularen JSON-Dateien unter `$XDG_CONFIG_HOME/argvus/config/` (normalerweise `~/.config/argvus/config/`). Sie bilden das kanonische logische Modell für von ARGVUS verwaltete Präferenzen, und `argvus-config` ist die einzige Komponente, die Dateien unterhalb von `data/generated/` schreibt. Native Dateien und generierte Dateien bleiben Kompatibilitätsprojektionen oder anwendungsspezifische Überschreibungen.

## Benutzerhandbuch: Schnellstart

Für die meisten Benutzer erfolgt die Konfiguration über das ARGVUS-Kontrollzentrum oder dieses Befehlszeilen-Tool:

```sh
# View your configuration location and validate it
argvus config path
argvus config validate

# Get a specific value
argvus config get /appearance/theme
argvus config get /appearance/theme --raw    # Without JSON formatting

# Set a value (must be valid JSON)
argvus config set /appearance/theme '"gruvbox-dark"'

# Apply changes to reload the session
argvus config apply
```
Für Administratoren und erweiterte Konfiguration lesen Sie unten weiter.

Der Benutzer root enthält nur das kanonische Dokument und verwaltete Daten:

```text
~/.config/argvus/
├── config.json
└── data/
```
Die Datei wird atomar geschrieben, bleibt privat für den Benutzer und wird durch `.config.lock` geschützt, außerhalb des `data/`-Baums, der während der Projektion ausgetauscht werden kann. Eine vorherige Version wird unter `data/backups/config.json.bak` gespeichert.

## Kommandozeile

Der `argvus config`-Unterbefehl wird im Prozess vom `argvus`-Paket bereitgestellt:

```sh
argvus config path
argvus config validate
argvus config ensure
argvus config migrate
argvus config status
argvus config explain /layout/gaps_out
argvus config get /appearance/theme --effective --raw
argvus config set /appearance/accent '"#61AFEF"'
argvus config patch appearance-patch.json
argvus config audio volume +5
argvus config audio mute toggle
argvus config apply-theme universe --variant sticky --accent '#EEEEEE' --gtk-mode dark
argvus config apply-theme universe-spaced --variant spaced --accent '#EEEEEE' --gtk-mode dark
argvus config theme-manifest universe-spaced
argvus config unset /appearance/accent
argvus config apply
argvus config rebuild
argvus config project
argvus config project --force
```
`ensure` erzeugt ein neues `config.json` mit den Standardwerten des Schemas oder füllt fehlende Felder, ohne explizite Werte zu ersetzen. Es schlägt bei fehlerhaftem JSON fehl.
`migrate` importiert den bestehenden alten Erscheinungszustand, ohne bereits vorhandene Werte in `config.json` zu ersetzen. `argvus sessionctl` führt `recover`, `ensure`, `migrate` und `project` aus, bevor es nach Konfigurationsänderungen einen Reload anwendet:

```sh
argvus sessionctl apply-config
```
`apply` und `rebuild` sind Aliase für das Wiederaufbauen jeder Projektion von `config.json`; `project` ist die historische Schreibweise und akzeptiert `--force`. `apply` delegiert außerdem das Post-Commit-Laufzeitneuladen an `argvus sessionctl`. Setzen Sie `ARGVUS_NO_RUNTIME=1` für Vorbereitungen oder isolierte Tests. Die Generierung erfolgt in der Staging-Umgebung, und der `data/`-Baum wird erst ausgetauscht, nachdem alle Projektoren erfolgreich sind. Daher ist das Löschen von `data/generated/` und Ausführen von `argvus config apply` ein reproduzierbares Wiederaufbauen ohne Veröffentlichung einer partiellen Sequenz von Hyprland- oder Taskbar-Ausgaben. Der Befehl zeichnet abgeleitete Abschnittshashes und die Laufzeitentscheidung in `$XDG_STATE_HOME/argvus/config-projection.json` (normalerweise `~/.local/state/argvus/config-projection.json`) auf. Weder das Manifest noch die generierten Dateien sind Quellen der Wahrheit. `apply` führt das Post-Commit-Neuladen über `argvus sessionctl` durch; `project` bleibt nur für Projektionen zur Kompatibilität.

Jede persistente Mutation verwendet dieselbe Regel: Wenn sich das kanonische Dokument geändert hat, ruft die CLI nach dem Commit einen zentralisierten `argvus sessionctl reload` auf. Das Ergebnis des Reloads wird in `data/internal/last-reload.json` aufgezeichnet.

`argvus sessionctl reload` ist daher ein *Anwendungsschritt* und kein Projektion-Schritt. Wenn das eingehende Manifest ausstehende Arbeiten meldet, stellt es die vom Plan beschriebenen Zielkonsumenten wieder her und ruft `argvus config project` nicht erneut auf; eine Reprojektion würde einen identischen Baum neu erstellen und den vom Mutation gerade erzeugten Plan verwerfen. Es greift nur dann auf Migration und Projektion zurück, wenn kein gesicherter Plan existiert, beispielsweise beim Start einer Sitzung.

### Reload-Ziele

`argvus config session-plan` antwortet auf das Neuladen mit den Zielen der Abschnitte, die sich geändert haben, und `argvus sessionctl` führt nur diese aus. Der Splash deckt ein Neuladen nur ab, wenn ein Ziel eine Oberfläche neu erstellt: eine Waybar-Leiste oder das Zurückbewegen des Bedienfelds auf den Bildschirm verschiebt jedes Fenster. Alles andere wird an Ort und Stelle neu geladen.

| Änderung | Ziele | Splash |
| --- | --- | --- |
| Thema, Akzentfarbe oder GTK-Modus | `hyprland`, `consumers`, `wallpaper`, `polkit` | nein |
| Nur Hintergrundbild | `wallpaper` (hyprpaper IPC; der Dienst startet nur für swaybg neu) | nein |
| `effects` (Weichzeichnen, Animationen, Transparenz, Schatten) | `hyprland`, `consumers` | nein |
| `layout` von Fenstern (Abstände, Rahmen) | `hyprland` | nein |
| `layout.taskbar` (Position, Ränder) | `hyprland`, `taskbar`, `widget_telemetry` | ja |
| `fonts` | `consumers` | nein |
| `control_panel` (Wechsel, Karten) | `control_panel` | ja |
| `default_apps`, `keyboard_shortcuts`, `hyprland`, `displays` | `hyprland` | nein |
| `power` | `hypridle` | nein |

`consumers` führt `argvus config theme-runtime consumers` aus (die Bar-Stylesheets, das Kontrollpanel über IPC, Hyprland-Farben, das Neuladen des Dunst-Themes, btop und kitty). Dunst verschiebt das Neuladen seines Themes, solange „Nicht stören“ aktiv ist. `wallpaper` führt `argvus config theme-runtime wallpaper` aus, das die effektive `/appearance/wallpaper` anzeigt. Der `layout.taskbar`-Hash wird getrennt vom Fensterlayout im Projektionsmanifest aufbewahrt, sodass eine Änderung der Lücke die Bars nicht neu startet. Der Fallback-`full`-Plan, der verwendet wird, wenn kein festgelegter Plan existiert, startet jede Oberfläche unter dem Splash neu.

Jede effektive Konfiguration hat eine operationale Generation. Sie wird in `data/internal/generation.json` und im Projektionsmanifest unter `$XDG_STATE_HOME/argvus/config-projection.json` gespeichert; sie ist nicht Teil der Dotfiles-Konfiguration. Die erneute Anwendung einer unveränderten effektiven Konfiguration bewahrt die Generation. `status` meldet, ob der Manifest-Hash, die Generations-Metadaten und alle generierten Dateien übereinstimmen; `apply` repariert eine veraltete oder unvollständige Projektion.

`explain /path` liest den Resolver direkt aus und gibt den effektiven Wert, die Quelle, das Thema und die Variante aus. Es werden keine generierten Dateien geparst oder eine zweite Präzedenzregel erstellt.

Themen- und Effekprojectierung werden direkt in `argvus-config` implementiert. Die Thementransaktion (`argvus theme-switch`, die `theme-apply` ausführt), `argvus config accent` und `argvus effects-toggle` projizieren nicht: sie validieren, sperren, committen über diese CLI und gleichen dann externe Verbraucher ab, die `config.json` nicht lesen können. `argvus effects-toggle` ist ein dünner Delegat ohne eigene Ausgabe. Die Appearance-Helper werden vom Projector nie aufgerufen und dürfen niemals unter `data/generated/` schreiben.

### Projektionsoberflächen

`project_effective` rendert jeden ARGVUS-eigenen Verbraucher in einem Staging-Baum, bevor er veröffentlicht wird:

| Oberfläche | Ausgabe |
| --- | --- |
| Hyprland | `generated/hypr/`, einschließlich Theme-Profile, `spaces-effective.conf`, `borders-effective.conf`, `hypridle.conf` und `monitors.lua` |
| Layout/Effekte | `generated/effects/`, `generated/spaces.conf`, `generated/borders.conf` |
| Waybar | `generated/waybar/`, einschließlich `argvus-taskbar.css` und `argvus-widget-telemetry.{jsonc,css}` |
| Widget-Telemetrie-Themen | `generated/widget-telemetry/` und `generated/waybar/widget-telemetry-themes/` |
| QuickShell | `generated/quickshell/` |
| Benachrichtigungen | `generated/notifications/` und `dunstrc` |
| Sperrbildschirm | `generated/hypr/hyprlock.conf` und die Hyprtoolkit-Themenprofile |
| GTK | `generated/gtk/` |
| Yazi | `generated/yazi/` |
| Superdatei | `generated/superfile/` |
| Terminal- und Qt-Profile | `generated/terminal/`, `generated/qt6ct/` |
| Schriftarten | `generated/fonts.conf` plus verwaltete Schriftblocks in den Waybar-Stylesheets |
| Wechselbare Geräte | `generated/removable-devices/theme.css` |
| Hintergrundbild | `generated/hypr/hyprpaper.conf` und `data/hypr/hyprpaper.conf` |
| Systemsteuerung und Standardwerte | `data/control-panel/`, `data/control-center/defaults.json` |
| Kalender, Eingabe, Tastatur, Strom | Kalender-TOML, Eingabefragment, Tastaturzustand, Wachhalte-Marker |

Die Themenebene wird auch außerhalb der generierten Wurzel gespiegelt:
`data/waybar/argvus-taskbar.{jsonc,css}`,
`data/waybar/argvus-widget-telemetry.{jsonc,css}`, `data/foot/`,
`data/qt6ct/`, `data/hypr/hyprtoolkit.conf` und
`data/hypr/application-style.conf`. Diese Kopien werden durch die verpackten
Standardwerte ersetzt, wenn sich der Erscheinungsabschnitt ändert, und der Projektor schreibt dann nur seine abgegrenzten verwalteten Blöcke um — den Schriftblock in den Waybar-Stylesheets und die `ARGVUS_TELEMETRY_*`-Blöcke im Telemetrieprofil — sodass Text außerhalb der
Marker einen Themenwechsel nicht überlebt. `fonts.targets` und
`control_panel.widget_telemetry_blocks` bleiben kanonisch: das Kontrollzentrum
bewahrt sie durch `argvus-config` und schreibt diese Dateien selbst nie.

Die Auswahl der Widget-Telemetrieblöcke ist kanonisch bei `/control_panel/widget_telemetry_blocks`, einem Array der sieben Blocknamen (`system`, `cpu_gpu`, `memory`, `storage`, `processes`, `network`, `keys`). Die veraltete `$XDG_STATE_HOME/argvus/widget-telemetry-blocks`-Datei wird nur als Migrations-Fallback gelesen und niemals beschrieben. ` and ` and blocks all patcht das kanonische Array in einem einzigen atomaren Schreibvorgang, und `apply-state` geht davon aus, dass die Auswahl bereits von seinem Aufrufer gespeichert wurde.

Die Hyprland-Abstands- und Randpräferenzen folgen derselben Grenze. Die Befehle argvus config spaces und argvus config borders (und ihre historischen Kompatibilitäts-Wrapper) aktualisieren /layout; Rust generiert data/generated/spaces-effective.conf und data/generated/borders-effective.conf und ist verantwortlich für Validierung und die Live-Anwendung über hyprctl. .spaces und .borders sind nur Migrationszustände und niemals die Quelle der Wahrheit.

Akzent- und GTK-Modus-Zustand verwenden die gleiche Rust-Grenze. Verwenden Sie ` or ` or argvus config accent --theme-default für den Akzentbesitz und `argvus config mode toggle|set dark|light` für den GTK-Modus. Die historischen Akzent- und Modusskripte sind Kompatibilitäts-Wrapper; `.accent-color`, `.accent-custom` und `.gtk-mode` sind nur Migrations-/Projektdaten-Dateien.

Logische ARGVUS-Pfade werden ebenfalls durch `argvus config paths` aufgelöst. Die Legacy-Datei `paths.sh` stellt nur Kompatibilitätsfunktionen bereit; native Überschreibungen, Benutzerdaten, generierte Daten und System-Fallback werden durch das Rust-Pfadmodell aufgelöst.

`audio` ändert den Standardausgabespeicher und zeichnet das Ergebnis in `/audio` in einem Schritt auf: `argvus config audio volume <N|+N|-N> [--limit PERCENT]` und `argvus config audio mute <on|off|toggle>`. Es schreibt den Speicher, ohne die Konfiguration neu zu laden, da PipeWire den Wert bereits hat. Das Waybar-Lautstärkemodul, die Hyprland-Mediatasten und das Kontrollpanel verwenden ihn, sodass `argvus config project`, das `/audio` auf den Speicher projiziert, niemals eine ältere Lautstärke wiederherstellt. Eine Lautstärke über 100 % (die Hyprland-Tasten erlauben bis zu 150 %) wird als 100 gespeichert.

`patch` wendet ein JSON-Objekt an, dessen Schlüssel JSON-Pointer in einer gesperrten Read-Modify-Write-Operation sind. `apply-theme` führt die äquivalente atomare Themen-Transaktion aus und ersetzt die verwalteten Erscheinungsabschnitte aus dem optionalen Erscheinungsprofil des Themas oder den standardmäßigen Paketvorgaben. Mutierende Befehle erstellen ein `NextUserConfig`, lösen ein `EffectiveConfig` auf, stagen alle Projektoren und veröffentlichen erst dann das kanonische JSON. Ein operatives `.config-transaction.json`-Marker erlaubt `argvus config recover`, die vorherige Konfiguration nach einem Absturz zwischen Daten- und JSON-Veröffentlichung erneut zu projizieren. Themenstandards werden vom Manifest jedes Themas deklariert (`themes/<theme>/theme.toml`): der Standard-Akzent (`palette.accent`), das GTK-Farbschema und das Hintergrundbild. CSS-, Lua-, Rasi- und Qt-Dateien werden daraus in den Themen-Build gerendert.

Das Ändern des Themas, einschließlich Hell/Dunkel, setzt die Hervorhebungsfarbe auf den Standardwert des Themas zurück, selbst wenn der Benutzer eine benutzerdefinierte Farbe gewählt hat: Die benutzerdefinierte Farbe bleibt nur bis zur nächsten Themenänderung erhalten. Verwenden Sie `argvus config accent` nach der Themenänderung, um sie erneut festzulegen. Das Anwenden eines Themas mit `argvus config theme-apply` schreibt `/appearance/accent` und `/appearance/gtk_mode` aus dem Manifest, und `argvus config accent` schreibt die gewählte Farbe mit `accent_custom` gesetzt, in einem Befehl.

`apply-theme` nimmt die Variante explizit als `--variant sticky|spaced` und löst den angeforderten Namen über das Manifest auf, sodass eine Anfrage niemals zu den Assets der anderen Variante auflösen kann. `theme-manifest THEME` ist dieselbe Auflösung, die nur lesbar angezeigt wird: Sie druckt den Manifest-Eintrag zusammen mit den `asset_id` und `layout_variant` auf, zu denen sie aufgelöst wurde, was ein Aufrufer benötigt, um auf verpackte Assets zuzugreifen. Sticky ist die unabgeschnittene ID und nur Spaced trägt ein `-spaced`-Suffix, daher werden `dracula-sticky` und `argvus-dark-dracula-sticky` akzeptiert und normalisiert zu `dracula`.

Mit `--raw` werden Zeichenkettenwerte ohne JSON-Anführungszeichen ausgegeben, während Objekte und Arrays JSON-Werte bleiben. Theme- und Layout-Dot-Dateien werden vom Projektion-Helfer nicht mehr geschrieben; `config.json` bleibt die einzige persistente Quelle für diese Einstellungen. Wenn die Projektion fehlschlägt, stoppt das Neuladen der Sitzung, bevor Desktop-Oberflächen neu gestartet werden, sodass ein älteres partielles Theme nicht als aktuell angezeigt wird.

## Export und Import

Dasselbe logische Modell wird für den Profil-Austausch verwendet:

```sh
argvus config export --scope appearance --output appearance.json
argvus config import --scope appearance appearance.json
```
Verwenden Sie `--scope desktop` für das vollständige verwaltete Desktop-Modell. Fehlende Felder werden mit den ARGVUS-Standards gefüllt; das erzeugte Ergebnis sollte nicht direkt bearbeitet werden.

Der Export und Import von Control-Center-Themen tauscht ein Thema und dessen visuelles Profil aus: Das Archiv enthält ein Schema-2-Themenverzeichnis (`<id>/theme.toml`, optional `appearance.json` und `effects.conf`, `overrides/`, die Hintergrundgrafik), das unter `$XDG_DATA_HOME/argvus/themes/` installiert wird. Das Profil enthält Erscheinungsbild, Layout-Geometrie, visuelle Effekte, Schriftarten und Panel-Darstellung. Audio-, Tastatur-, Netzwerk- und andere nicht-visuelle Präferenzen bleiben außerhalb der Themen. Verwenden Sie den oben genannten `appearance`-Bereich, um die vollständige Erscheinungskonfiguration zu übertragen, und den `desktop`-Bereich für ein vollständiges Konfigurations-Backup.

Das aktuelle Schema gruppiert Präferenzen unter `appearance`, `layout`, `effects`, `fonts`, `control_panel`, `default_apps`, `keyboard_shortcuts`, `hyprland`, `displays`, `power`, `session`, `calendar` und `removable_devices`. `default_apps` ist die einzige persistente Quelle für bevorzugte Anwendungen, und `keyboard_shortcuts` speichert stabile englische, lokal-unabhängige Schlüssel mit Zeichenkettenwerten oder `null` für deaktivierte Bindungen.

Das Kontrollzentrum liest die kanonischen Erscheinungswerte, wenn eine Erscheinungsseite geöffnet wird, und schreibt Änderungen über `argvus-config`. Die vorhandenen nativen Statusdateien bleiben Projektionen für die Erscheinungsskripte und ältere Installationen.

Das Kontrollpanel verwendet `control_panel.cards` für Kartenbestellung und Sichtbarkeit und `control_panel.enabled` für den Hauptschalter des Panels. Seine `control-panel/cards.json`- und `state/control-panel`-Dateien werden als Kompatibilitätsprojektionen aufbewahrt. Änderungen, die im Kontrollpanel vorgenommen werden, sind daher Teil des `desktop`-Exportspektrums.

Schriftartpräferenzen verwenden `fonts.targets` für Familie, Stil und Größe der Taskleiste, Telemetrie, des Kontrollpanels, Systems, Anwendungen, Terminals und Browser-Ziele. `fonts.rendering` speichert Antialiasing-, Hinting-, Subpixel-Reihenfolge- und DPI-Präferenzen. `fonts.conf`, die Fontconfig-Regel, Desktopeinstellungen und die verwalteten Schriftblocke innerhalb der Waybar-Stylesheets sind Projektionen: Das Kontrollzentrum persistiert `/fonts/targets/*` bis `argvus-config`, und der Projektor schreibt nur seinen abgegrenzten Block um.

Die Hyprland-Einstellungen verwenden `keyboard_shortcuts` für Tastenkürzel, `hyprland.input` für Maus- und Touchpad-Einstellungen und `hyprland.keyboard` für XKB-Layout, Variante und Optionen. Eine Manifest-Bindungs-ID wird einer stabilen `config_key` zugeordnet (zum Beispiel `window.close` → `close_window` und `window.drag_mouse` → `drag_window__floating_window_only`). Anzeigeeinstellungen verwenden `displays.monitors`, `displays.workspaces` und `displays.primary_monitor`. Native TOML-Dateien und generierte Lua-Dateien unter `data/generated/` bleiben Projektionen, die von Hyprland verwendet werden.

Jeder `effects`-Schalter ist standardmäßig aktiviert (Animationen, Blur, Widget-Telemetrie und jede Transparenzoberfläche), auch auf virtuellen Maschinen und in Energiesparmodi; Fensterschatten (`shadow_enabled`) und Wackeln (`wobble_enabled`) sind standardmäßig deaktiviert. Effektpräferenzen verwenden `effects` für den Animations-Master-Schalter (`animations`, angewendet auf `hyprland.lua` aus dem projizierten `hypr/state.lua`, unabhängig vom Effektschalter, der `blur_global_enabled` plus dem `transparency_<surface>_enabled` von `taskbar`, `control-panel`, `widget-telemetry`, `terminal` und `launchers` ist und als aktiviert angezeigt wird, wenn einer von ihnen aktiviert ist; `argvus effects-toggle animations|effects` schaltet sie), den globalen Blur-Schalter (`blur_global_enabled`), die Hyprland-Blur-Parameter (`blur_size`, `blur_passes`, `blur_brightness`, `blur_noise`, `blur_contrast`, `blur_vibrancy`, `blur_vibrancy_darkness`), den Fensterschatten (`shadow_enabled`, `shadow_sharp`, `shadow_color`, `shadow_range`, `shadow_render_power`, `shadow_offset_x`, `shadow_offset_y`, `shadow_scale`), die Animationswerte (`animation_<leaf>_speed`, `_curve`, `_style`, `_popin` und `_enabled` für jede Animation, sowie `animation_bezier_x1` bis `animation_bezier_y2` und `animation_spring_mass`, `animation_spring_stiffness` und `animation_spring_dampening` für die benutzerdefinierten Kurven) und Transparenzwerte pro Oberfläche. Effekte gehören zum Thema, nicht zur Variante: Sticky und Spaced verwenden dieselbe `<theme>.conf` Effekprojektion, während ihre Layoutprojektionen unterschiedlich sind. Oberflächen umfassen derzeit `taskbar`, `control-panel` und `widget-telemetry`; Transparenzwerte sind Prozentsätze von `0` bis `100` (das Kontrollzentrum und die Launcher-Oberflächen standardmäßig zu `15`, die anderen zu `50`), Unschärfeparameter sind Zahlen innerhalb der in `schema.json` angegebenen Bereiche, und aktivierte Zustände sind Booleans. Animationswerte werden auch von `argvus-config` validiert: Geschwindigkeiten sind Zahlen von `1` bis `10` (Dezimalzahlen zulässig), Pop-in-Prozentsätze sind ganze Zahlen von `1` bis `100`, Kurven müssen Namen von der Seite „Animationen“ sein, Stile müssen Namen von der Seite „Animationen“ sein und vom eigenen Blatt der Animation akzeptiert werden (z. B. wird `popin` abgelehnt). `animation_workspace_in_style`, `gnomed` wird nur zum Öffnen und Schließen von Fenstern akzeptiert, und die Folienvarianten werden auf den Fenster- und Ebenenblättern abgelehnt. Die Bézier-Y-Punkte bleiben zwischen `-1` und `2`, der Bereich Hyprland 0,56,2 analysiert und die anderen Kurvenparameter bleiben innerhalb ihrer dokumentierten Bereiche. Die Blätter `layers` und `switching` sind nicht mehr vorhanden, daher verlieren Konfigurationen, die vor ihrer Entfernung gespeichert wurden, beim Laden die Schlüssel `animation_layers_enabled`, `animation_layers_speed`, `animation_layers_curve` und `animation_switching_*`. Stile außerhalb des aktuellen Satzes werden gelöscht und Bézier-Y-Punkte außerhalb von `-1` bis `2` werden geklemmt, sodass die gepackten Standardwerte wieder gelten was auch immer entfernt wurde.

Die Energieeinstellungen verwenden `power.screen_off_minutes`, `power.lock_minutes`, `power.keep_awake` und die Richtlinien für Deckel/Stromtaste. Die projizierten Formen sind `data/generated/hypr/hypridle.conf` und der Marker `data/power/keep-awake`; `.keep-awake` ist nur ein Migrationsinput. `session.language` ist außer Betrieb: bestehende Werte werden ignoriert, neue Schreibvorgänge werden abgelehnt, und es wird keine Sprachkompatibilitätsdatei projiziert. Setzen Sie `LANG` stattdessen im Benutzer `locale.conf`. Kalenderpräferenzen verwenden `calendar` und werden in den Taskleisten-Kalender-Benutzer-TOML projiziert. Einstellungen für Wechselgeräte verwenden `removable_devices`; die Anwendung für Wechselgeräte behandelt diesen Abschnitt als Benutzerüberschreibung gegenüber den ausgelieferten Standardwerten.

`migrate` importiert die alten Stammdateien und -abschnitte einmal und bewahrt einen bestehenden kanonischen Wert. Alte `defaults.json`, `/defaults`, `/hyprland/keybindings`, `keybindings.toml`, Punktdateien, `generated/` und Komponentendateien werden unter `data/` verschoben; wiederholtes Neuladen erstellt das alte Stammverzeichnislayout nicht erneut. Die erneute Auswahl des aktuellen Themas ist idempotent: Wenn der kanonische Zustand und die Projektionen bereits übereinstimmen, meldet der Projektionsplan keine geänderten Abschnitte und die Sitzung überspringt Splash- und Dienstneustarts.

Power folgt demselben Projektionsvertrag: Das Control Center schreibt `/power` in `config.json`, `argvus config project` rendert `data/hypr/hypridle.conf` atomar, und erst dann darf `argvus sessionctl` `argvus-hypridle.service` neu starten. Das Entfernen dieser Datei ist mit `argvus config project` wiederherstellbar; ein Wert von `0` lässt das entsprechende Timeout weg und bedeutet Nie.

Das Erscheinungsbild des Sperrbildschirms wird von Rust aus dem kanonischen Thema, Akzent, Effekten, Hintergrundbild und Schriftzustand in `data/hypr/hyprlock.conf` projiziert. Der historische `hyprlock-theme.sh` Einstiegspunkt dient nur der Kompatibilität und delegiert an `argvus config lock apply`; `--invalidate` invalidiert nur den abgeleiteten unscharfen Hintergrundbild-Cache.

Änderungen des Energiezeitlimits verwenden `/power/lock_minutes` und `/power/screen_off_minutes` als ihre Quelle der Wahrheit. `argvus config project` verwendet den bestehenden Hypridle-Renderer wieder, und der `idle-timeout.sh` Einstiegspunkt ist nur ein Kompatibilitätsadapter; ein Neustart des Dienstes wird weiterhin an `argvus sessionctl` delegiert.

Das Erscheinungsbild von Benachrichtigungen wird aus dem kanonischen Thema, Akzent und Effekten in `data/notifications/dunstrc` abgeleitet. Die historische Benachrichtigungsdatei `theme.sh` bietet nur Kompatibilitätsfunktionen, die `argvus config notifications` aufrufen; sie analysiert oder verändert die Dunst-Konfiguration nicht.

Hardware-orientierte Adapter wie Display-Erkennung und Bluetooth-Steuerung werden nun von der Rust-Display-Domäne importiert: `nwg-displays` ist ein externer Importeur, während `/displays` und `data/generated/hypr/monitors.lua` jeweils kanonischer und abgeleiteter Zustand bleiben. Die Display-Tools ändern das Layout mit `argvus config monitor-set`, `monitor-primary`, `monitor-workspaces` und `monitor-remove`; die Display-Befehle sind `argvus config display status|list-modes|set|apply|apply-nwg|migrate-nwg|nwg-import|nwg-apply`. Die Hyprland-Datei wird von `/displays` projiziert; kein Shell-Skript schreibt sie mehr. Auswahl des Wallpapers, Taskleiste-Nutzungsgruppenmodus und Standard-App-Suche verwenden ebenfalls `argvus-config`-APIs; ihre historischen Shell-Einstiegspunkte sind UI- oder Kompatibilitäts-Wrapper. Keep Awake ist eine persistente `/power/keep_awake`-Voreinstellung, die auf den `data/power/keep-awake`-Marker projiziert wird, während lock-DPMS ein Laufzeitzustand der Sitzung ist unter
`$XDG_CACHE_HOME/argvus/lock-dpms`. Das Tastaturlayout wird jetzt vom Rust-`argvus-keyboard-layout`-Helfer verwaltet: Die konfigurierte Reihenfolge stammt aus der generierten Hyprland-Eingabeprojektion oder dem Fallback der Gebietsschemaeinstellungen, während das aktive Layout den zur Laufzeit entdeckten Zustand der Sitzung aus Hyprland beibehält. Der Zykluseinstiegspunkt und der Tastaturlayout-Daemon sind nur Kompatibilitäts-Hüllen; der Benutzerdienst führt den Rust-Daemon direkt aus, und `argvus sessionctl keyboard-layout cycle` ist der stabile Befehl für Aufrufer.

Der Bluetooth-Laufzeitstatus folgt der Grenze des externen Adapters, ist jedoch nicht Teil von `argvus-config`: `argvus bluetooth` ist für typisierte Analyse und Validierung rund um `bluetoothctl`, begrenzte Wiederholungen, Systemdienstprüfungen und RFKill-Status verantwortlich. `argvus bluetoothctl` ist der Einstiegspunkt. Adapterleistung, gekoppelte Geräte und verbundene Geräte bleiben BlueZ-Laufzeitstatus und nicht kanonischer JSON-Status; die Validierung von hardwareaktivem Bluetooth ist umgebungsabhängig.
## Laufzeit-Identität und Abgleich

Verwenden Sie `argvus config status --verbose`, um das genaue Binärprogramm, die Build-Identität, das Konfigurations-Root, das generierte Root, das Asset-Root, die Manifestquelle und die aktive Generation zu prüfen. Es meldet `synchronized`, `stale`, `incomplete`, `legacy` und `pending-transaction` Projektionszustände.

`argvus config --version` meldet die Paketversion, die Build-Revision, wenn sie von der Build-Umgebung bereitgestellt wird, und das Profil. Entwicklungstests müssen ihre Binärdateien und Assets explizit einfügen; sie dürfen nicht stillschweigend `argvus config` oder `~/.config/argvus` verwenden. `apply` normalisiert sicher alte kombinierte Bezeichner wie `argvus-dark-spaced` in separate Theme- und Variantenwerte, bevor Projektionen neu aufgebaut werden. Die Identität der verpackten Assets wird in dieser Version nicht in einem Manifest aufgezeichnet.

## Nachtlicht

Öffnen Sie **Desktop & Eingabe > Anzeigen > Nachtlicht** oder führen Sie `argvus control-center displays night-light` aus. Konfigurieren Sie Temperatur, manuelle Aktivierung und feste lokale Zeitpläne über `argvus config display night-light`. Siehe [Nachtlicht](/docs/user-guide/night-light/) für Befehle, Status und Laufzeitanforderungen.

### Wackeleinstellungen (Hyprland 0.57+)

`effects.wobble_enabled`, `wobble_stiffness`, `wobble_damping`, `wobble_mass`, `wobble_intensity` und `wobble_mesh` projizieren direkt in `hypr/state.lua`. Verwenden Sie `argvus effects-toggle wobble-settings get`, um sie zu lesen, oder `argvus effects-toggle wobble-settings set enabled=enabled stiffness=100 damping=10 mass=1 intensity=1 mesh=8`, um zu speichern und eine erneute Ladeanforderung zu stellen. `enabled=disabled` schaltet die Wirkung aus.

Das Kontrollzentrum teilt den Animationsentwurf: `animations-settings get` gibt auch `wobble_*`-Einträge zurück, und `animations-settings set` akzeptiert diese vollständigen Namen zusammen mit seinen bestehenden Animationsargumenten. Dies speichert den gesamten Entwurf in einem validierten Patch und fordert eine erneute Ladeaktion an. Zum Beispiel:

```sh
argvus effects-toggle animations-settings set enabled=enabled wobble_enabled=true wobble_intensity=0 wobble_mesh=8
```
Die globale `enabled` steuert Animationen; `wobble_enabled` ist ihre unabhängige Einstellung. Das Deaktivieren globaler Animationen unterdrückt Wackeln in Lua, ohne diese Einstellung zu löschen. Ungültige Werte lehnen den gesamten Patch ab. Fehlende Schlüssel erhalten die ARGVUS-Standardeinstellungen, ohne vorhandene Auswahlmöglichkeiten zu ersetzen. Siehe [Appearance](/docs/user-guide/appearance/#window-wobble) für Bereiche und Standardeinstellungen.

### Benutzerdefinierte Tastenkombinationen

`/keyboard_shortcuts` behält seine vorhandenen Zeichenketten- oder Nullwerte für offizielle Manifestaktionen bei. Benutzerdefinierte globale Tastenkombinationen verwenden ein separates `/hyprland/custom_shortcuts`-Objekt, das in `config/hyprland.json` gespeichert wird. Seine stabilen `custom_`-IDs sind unabhängig von Beschreibungen und Lokalisierungen. Jeder Eintrag hat `description`, `keys`, `enabled` und ein typisiertes `action`:

```json
{
  "custom_editor": {
    "description": "Open my editor",
    "keys": "SUPER + CTRL + E",
    "enabled": true,
    "action": { "type": "command", "command": "editor --new-window" }
  },
  "custom_float": {
    "description": "Toggle floating",
    "keys": "SUPER + CTRL + F",
    "enabled": false,
    "action": {
      "type": "dispatcher",
      "dispatcher": "window.float",
      "arguments": { "action": "toggle" }
    }
  }
}
```
Das Beispiel ist der Wert von `/hyprland/custom_shortcuts`, kein vollständiges Konfigurationsdokument. Dispatcher-Namen stammen von den unterstützten Lua-Dispatcher-Fabriken; Argumente sind ein optionales JSON-Objekt oder `null`, niemals ausgewerteter Code. Benutzerdefinierte Shortcuts haben keine Untermaps, Mausbindungen oder erweiterten Flags. Die Projektion schreibt `data/generated/hypr/custom-shortcuts.lua` als escape-te Tabelle, einschließlich einer leeren Tabelle, wenn der letzte Datensatz entfernt wird. Sowohl die vollständige Projektion als auch `project --only keyboard_shortcuts` erzeugen sie. Hyprland registriert aktivierte Einträge mit `hl.bind`; optionale Datei- und individuelle Registrierungsfehler werden erfasst und protokolliert.

Das Control Center erstellt/bearbeitet Datensätze und deaktiviert bestätigte Konflikte mithilfe einer `argvus config patch`-Transaktion. Es speichert vor der Anwendung über den vorhandenen Konfigurationsdienst. Ein fehlgeschlagenes Anwenden macht ein erfolgreiches Speichern nicht rückgängig. Das Wiederherstellen der offiziellen Standardeinstellungen bewahrt benutzerdefinierte Datensätze und wird verweigert, wenn es mit einer aktiven benutzerdefinierten Verknüpfung kollidieren würde. Benutzerdarstellungen bleiben in allen Sprachen wörtlicher Text in der effektiven generierten Kurzübersicht.

## Globaler Fenstermodus

Öffnen Sie **Control Center → Desktop & Eingabe → Hyprland → Fenster → Modus** und wählen Sie **Fenster kacheln (Standard)** oder **Schwebende Fenster**. Derselbe Selektor ist im Fenster-Modus-Symbol in der Taskleiste verfügbar.

Die Einstellung gilt für jeden Monitor und Arbeitsbereich und wird nach dem Abmelden gespeichert. Floating konvertiert bestehende Anwendungsfenster und neu geöffnete Fenster. Tile stellt nur Fenster wieder her, die durch diese Richtlinie konvertiert wurden; Dialoge und andere Fenster, die bereits im Floating-Modus waren, bleiben floating. Ein Fenster, das explizit angeheftet wird, während Floating aktiv ist, bleibt ebenfalls floating, um seine Anheftung zu bewahren. Das gekachelte Layout wird neu berechnet; der genaue vorherige Aufteilungsbaum wird nicht wiederhergestellt.

`SUPER + SHIFT + Space` behält sein normales pro-Fenster-Verhalten in Tile bei. Im Floating-Modus wechselt es ein Fenster nicht zurück zum Kacheln. Panels, die Taskleiste und Benachrichtigungen sind Layer-Shell-Oberflächen und bleiben unberührt. Vollbild- und Maximierungsfunktionen verwenden weiterhin das normale Verhalten von Hyprland.

Die kanonische Präferenz ist `/hyprland/window_mode` in `$XDG_CONFIG_HOME/argvus/config/hyprland.json`. Ihre erzeugte Projektion ist `data/generated/hypr/window-mode.lua`; nicht bearbeiten. Das Register für die pro-Fenster-Wiederherstellung befindet sich unter `$XDG_RUNTIME_DIR/argvus`, ist auf die Kompositor-Instanz beschränkt und überlebt Konfigurationsneuladungen. Es wird in einer neuen Sitzung nicht wiederverwendet.

Wenn die Anwendung aussteht oder fehlschlägt, unterscheidet die Schnittstelle dies von der gespeicherten Präferenz. Wählen Sie die Option erneut, um einen Versuch zu wiederholen. Ohne einen Kompositor wird die Präferenz für das nächste Login gespeichert.

```sh
argvus config window-mode get
argvus config window-mode status --json
argvus config window-mode set floating
argvus config window-mode set tile
argvus config window-mode toggle
```
## Fensterbuttons und Anwendungsleiste

Der Schwebe-Modus fügt einem normalen Anwendungsfenster eine ARGVUS-Titelleiste mit **Minimieren**, **Maximieren / Wiederherstellen** und **Schließen** hinzu. Ein Doppelklick auf die Titelleiste wechselt zwischen Maximierung und Normalgröße. Die zusätzliche Dekoration ist im Kachel- und Vollbildmodus verborgen. Anwendungen, die ihre eigenen Steuerelemente zeichnen, können beide Leisten anzeigen; einheitliche ARGVUS-Steuerelemente sind beabsichtigt.

Eine zweite Waybar erscheint auf jedem Monitor, am Rand gegenüber der Haupttaskleiste. Sie listet nur die Fenster auf, die zum aktuellen Arbeitsbereich dieses Monitors gehören, einschließlich minimierter Fenster aus diesem Arbeitsbereich. Ein geöffneter spezieller Arbeitsbereich hat Vorrang. Angeheftete Fenster bleiben sichtbar; ausgeblendete Gruppentabs erzeugen keine zusätzlichen Einträge, und eine minimierte Gruppe hat einen Wiederherstellungseintrag. Tabs innerhalb einer Anwendung werden nicht gesondert aufgeführt. Minimierte Einträge behalten ihre ursprüngliche Monitor-/Arbeitsbereichszuordnung bei. Klicken Sie auf ein inaktives Fenster, um es zu aktivieren, auf ein aktives Fenster, um es zu minimieren, oder auf ein minimiertes Fenster, um es wiederherzustellen; dessen Kontextmenü bietet Wiederherstellen, Minimieren, Maximieren/Wiederherstellen und Schließen. Unbekannte Anwendungssymbole fallen auf ein Fenstersymbol zurück. Das Aktivieren oder Wiederherstellen eines Eintrags bewahrt die Zeigerposition. Die sekundäre Leiste verwendet das effektive primäre CSS (Farben, Schriftarten, Rahmen und Rundungen), mit ihrer Höhe und den gespiegelten Rändern auf der gegenüberliegenden Kante.

`SUPER + Alt + Up` platziert die Hauptleiste oben und die Fensterleiste unten. `SUPER + Alt + Down` macht das Gegenteil. Wird eine Richtung wiederholt, bleiben die Positionen unverändert. Kachel hat nur die Hauptleiste. Die Rückkehr zu Kachel stellt alle minimierten Fenster wieder her, bevor die Fensterleiste entfernt wird, und bewahrt dann die ursprüngliche Kachel-/Schwebe-Strategie.

Die Minimierung verwendet den reservierten Arbeitsbereich `special:argvus-minimized`. Verwenden Sie diesen Arbeitsbereich nicht manuell. Der Prozess `argvus windows service` zeichnet Identität, ursprünglichen Arbeitsbereich/Monitor, Geometrie, Vollbild/Maximierung und Pinnstatus in einem compositorspezifischen Laufzeit-Register unter `$XDG_RUNTIME_DIR/argvus` auf. Die Aufzeichnungen überleben einen Dienstneustart oder das Neuladen der Konfiguration, überschreiten jedoch nicht die Compositor-Sitzungen. Das Schließen eines Fensters entfernt seinen Eintrag. Verschwindet ein Monitor, wird bei der Wiederherstellung ein verfügbarer Ausgang verwendet. Eine fehlende oder fehlgeschlagene Wiederherstellungsleiste verhindert neue Minimierungen; eine fehlgeschlagene Wiederherstellung behält den Eintrag und die Wiederherstellungsleiste bei.

Die Sitzung besitzt `argvus-windows.service` und `argvus-window-bar.service`. Letzterer hält sein Waybar-Kind im Tile-Modus inaktiv. Das Hyprbars-Plugin wird von `argvus-hyprland` gebündelt, gebaut für die Hyprland 0.57.0 ABI auf x86_64; es wird nicht im Home-Verzeichnis des Benutzers kompiliert. Bauen Sie das Plugin bei einem Update der Compositor-ABI neu.Befehle: `argvus windows list --json`, `watch --json`, `position top|bottom`, `restore-all` und `minimize|restore|activate|maximize|close ADDRESS`. Die Fensteradressen stammen aus der Liste. `position top --persist` speichert/projektiert die Voreinstellung, ohne die Hauptleiste neu zu starten. Konfigurations- und Laufzeitfehlerdetails werden im Journal geschrieben; sichtbare Aktionsfehler zeigen ebenfalls eine übersetzte Benachrichtigung an.

In **Spaced + Floating** respektieren Fenster äußere Abstände und die von beiden Leisten reservierten Bereiche. Die Begrenzungen beinhalten Rahmen und Titel, der innerhalb desselben Fensterrahmens sitzt. Maximieren über ARGVUS-Steuerungen füllt diesen Bereich mit Rand; Wiederherstellen bringt die vorherige Geometrie zurück, auch nach einem Dienstneustart. Vollbild nutzt weiterhin die gesamte Ausgabe. Schwebende Fenster können sich überlappen; innere Lücken im Kachelmodus trennen sie nicht.

Die inneren Ränder, die von den Balken reserviert werden, werden in den äußeren Fensterabständen nicht doppelt gezählt. Bei symmetrischem Abstand ist der sichtbare Abstand zwischen Fenster und Balken oben und unten gleich, auch nach dem Tausch der Balken.

`SUPER + Alt + Space` schaltet den globalen Kachel-/Schwebemodus um und verwendet die gleiche Einstellung wie die Taskleiste und das Control Center. Der entsprechende Befehl ist `argvus config window-mode toggle`. Gleichzeitige Aufrufe werden seriell ausgeführt; das Umschalten verwendet die gespeicherte Einstellung, selbst wenn die Anwendung ausstehend ist. Die Tastenkombination kann in den Tastenkombinationen geändert oder deaktiviert werden; `SUPER + Shift + Space` behält seine Einzel-Fenster-Aktion bei.
