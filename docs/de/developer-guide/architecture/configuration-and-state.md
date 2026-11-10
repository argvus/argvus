---
title: Konfiguration und Zustand
description: Geteilter Zustand, Überschreibungen und generierte Laufzeitdateien.
---
Der normale Benutzer root ist `$XDG_CONFIG_HOME/argvus` (normalerweise `~/.config/argvus`). Verwaltete logische Einstellungen werden in `config.json` gespeichert. Thema, Akzent, Hintergrundbild, Abstand und Rahmenpräferenzen sind dort kanonisch; die Dot-Files bleiben abgeleitete Kompatibilitätsprojektionen für aktuelle Hyprland Lua- und Shell-Verbraucher, während diese Verbraucher zu `data/` migrieren.

`argvus-config` ist der einzige Autor unterhalb von `generated/`. Jede von ARGVUS besessene Verbraucherdatei ist eine Projektion von `config.json`, einschließlich Hyprland, GTK, Qt6ct, Waybar, Quickshell, Dunst, Hyprlock, der Waybar-Taskleiste und Widget-Telemetrieprofile, der Yazi- und Superfile-Konfigurationsbäume, der Terminal- und Qt-Profilbäume, der fontverwalteten Blöcke und des Stylesheets für wechselbare Geräte. Komponenten lösen explizite Benutzerüberschreibungen, dann generierte Ausgaben und danach Paket-Standardeinstellungen auf. Rofi ist keine Projektion: `argvus-launcher` besitzt seine Dateien, und der Benutzerkopiert lebt unter `data/rofi/`. Normale Themen-/Reload-Projektionen befördern weder generierte noch gepackte Dateien in eine Benutzerüberschreibung. Dokumentationen und Werkzeuge sollten die logische Quelle identifizieren, anstatt Benutzer zu bitten, generierte Ausgaben zu bearbeiten.

`argvus-appearance` besitzt daher Themen- und Akzentorchestrierung, während `argvus-config` den kanonischen Zustand und die strukturierte Projektion besitzt. `argvus theme-switch` (die `argvus config theme-apply`-Transaktion) und `argvus config accent` validieren, sperren und committen über `argvus-config`, wenn verfügbar, und gleichen dann Kompatibilitätskonsumenten aus, die `config.json` nicht direkt lesen können. Diese Adapter sind die native Qt6ct-Datei, GTK `settings.ini`- und `gsettings`-Hinweise, die Terminal- und Systemmonitor-Anwendungen, foot, Superfiles eigenes Konfigurationsverzeichnis und der Pre-Authentication-Greeter. Ein Helfer, der einen neuen Adapter benötigt, muss ihn dort hinzufügen, niemals innerhalb von `generated/`.

Die Laufzeitverantwortung ist bewusst nach Aufgabenbereichen aufgeteilt: Rust besitzt den kanonischen Speicher, die Schema-Validierung, Migrationen, das Pfadmodell, Sperren, atomare Schreibvorgänge, Projektions-Hashes und strukturierte Renderer. Shell bleibt nur als kleine Kompatibilitätseinstiege oder Adapter für externe Nutzer wie `hyprctl` und themenbezogene Skripte im Besitz von Assets. Python ist für Tests, Audits und Entwicklungshilfen reserviert; es ist kein Teil der kritischen Desktop-Laufzeit.

`argvus config apply` ist die Grenze des Rebuilds (`project` bleibt die kompatible Schreibweise). Es regeneriert fehlende Ausgaben, selbst wenn das kanonische Dokument unverändert ist, rendert Projektoren in der Staging-Umgebung und tauscht den `data/`-Baum nur atomar aus, nachdem jede Projektion erfolgreich ist. Der Sitzungslebenszyklus bleibt im Besitz von `argvus sessionctl`, das dieses Manifest nutzt, um gezielte Neubeladungen zu planen.

Wenn ein Anrufer bereits eine Generation durchgeführt hat — `apply-theme`, eine Akzentänderung, eine `set`/`patch` oder eine Änderung des Widget-Telemetrie-Blocks — `reload` **wendet** diese Generation an, anstatt sie erneut zu projizieren. Es liest den Plan aus dem Manifest, stellt die Zielkonsumenten wieder her und projiziert nur, wenn das Manifest fehlt oder keine ausstehenden Arbeiten hat. Eine erneute Projektion würde zu diesem Zeitpunkt einen identischen Baum wiederaufbauen und den Plan verwerfen, den die Mutation gerade erstellt hat.

Das Manifest zeichnet auch die operative Generation, den Hash der effektiven Konfiguration, geänderte Domänen und die Liste der generierten Dateien auf. `data/internal/generation.json` ist ein abwerfbares lokales Projektion-Marker und niemals eine Benutzereinstellung. `argvus config status` vergleicht diese Aufzeichnungen mit der aufgelösten Konfiguration und meldet fehlende oder zusätzliche generierte Dateien.

Mutationen folgen derselben Motorgrenze: `UserConfig` wird in ein `NextUserConfig` geklont, validiert, in `EffectiveConfig` aufgelöst, projiziert und erst dann gespeichert. `.config-transaction.json` ist der operative Wiederherstellungszustand; `argvus config recover` reprojiziert deterministisch das vorherige kanonische Dokument nach einem Absturz zwischen dem Veröffentlichen von `data/` und `config.json`. Ein Nachladen wird erst nach dem Commit angefordert, und ein Fehlschlag beim Nachladen setzt keine kohärente gespeicherte Konfiguration zurück.

## Auflösungsmodell

```text
config.json logical model
    ↓
ARGVUS native compatibility state and per-app override
    ↓
ARGVUS generated runtime file
    ↓
/usr/share/argvus packaged default
    ↓
upstream default
```
Die genauen Kandidaten variieren je nach Komponente, aber das Prinzip bleibt konsistent: Eine Benutzerüberschreibung hat Vorrang, generierte Ausgaben sind abgeleiteter Zustand, und gepackte Dateien bleiben schreibgeschützte Eingaben. Die Sitzung exportiert `ARGVUS_CONFIG_HOME` und `ARGVUS_SYSTEM_CONFIG`, sodass Helfer und Benutzerdienste dieselben Wurzeln auflösen.

Die Wiederherstellung beim Neuladen folgt `recover → ensure → migrate → apply → runtime reload`, und der Projektschritt wird übersprungen, wenn das eingehende Manifest bereits eine festgeschriebene Generation beschreibt. `ensure` materialisiert `config.json` aus Schema-Standardeinstellungen nur, wenn das Profil fehlt, und füllt fehlende Felder, ohne explizite `false`, `0` oder benutzerdefinierte Werte zu ersetzen. Ein bereits fehlerhaftes Dokument bleibt ein Fehler, anstatt stillschweigend ersetzt zu werden.

Persistente Mutationen vergleichen die aufgelösten alten und neuen Konfigurationen über `ChangedDomains`. Nach dem Commit wird eine Reload-Anfrage über `argvus sessionctl` gesendet; ein Reload-Fehler wird gemeldet und aufgezeichnet, ohne die kommittierte Konfiguration zurückzusetzen. Der Sitzungsstart führt eine Wiederherstellung vor der Bootstrap-Materialisierung und -Projektion durch, sodass Verbraucher nicht gegen eine abgebrochene Transaktion starten.

| Zustand | Zweck |
| --- | --- |
| `config.json` | Kanonische verwaltete logische Präferenzen; die einzige Quelle für alles unter `generated/`. |
| `.active-theme`, `.accent-color`, `.accent-custom` | Legacy-Migrationsinputs; keine Laufzeitquellen und werden nicht regeneriert. |
| `.wallpaper-custom` | Legacy-Migrationsinput für `appearance.wallpaper`; keine Laufzeitquelle. |
| `.spaces`, `.borders` | Legacy-Layout-Migrationsinputs; kanonische Werte werden unter `layout` in `config.json` gespeichert. |
| `fonts.conf` | Kompatibilitätsprojektion von kanonischen Schriftartauswahlen. |
| `generated/effects` | Kompatibilitätsprojektion von kanonischen Effekten; wegwerfbar. |
| `$XDG_STATE_HOME/argvus/config-projection.json` | Projektionsmanifest: geänderte Abschnitte, betroffene Verbraucher und ob ein Neuladen erforderlich ist. Gelesen von `reload`; niemals eine Wahrheitquelle. |
| `$XDG_STATE_HOME/argvus/widget-telemetry-blocks` | Legacy-Telemetrie-Blockpräferenzen. Nur als Migrations-Fallback lesen; kanonische Werte befinden sich unter `/control_panel/widget_telemetry_blocks` und die Datei wird nicht mehr geschrieben. |

Eine Eigenschaft ist kanonisch, wenn sie von `config.json` gelesen und von dort projiziert wird. Eine Eigenschaft ist abgeleitet, wenn sie projiziert wird, aber auch als Migrationsinput verwendbar ist. Alles, was eine Komponente zur Laufzeit noch schreibt – ein Cache, eine native Konfigurationsdatei unter `$XDG_CONFIG_HOME/<app>`, eine Greeter-Statusdatei – ist ein Adapterziel und darf nicht wieder als Konfiguration gelesen werden.

Projection deckt Verbraucher für Hyprland, GTK/Qt, Waybar, Quickshell, Terminals, Benachrichtigungen, Sperrbildschirm und native Anwendungen ab. Hyprland liest aktive und inaktive Rahmenfarben aus dem ausgewählten Thema; ein explizites `appearance.accent_custom=true` ist der einzige Fall, der den aktiven/Gruppen-Rahmen mit dem Benutzerakzent überschreibt. Das Umschalten des Themas wendet diese Werte durch das Neuladen des Compositors erneut an, sodass die Farben des vorherigen Themas nicht beibehalten werden können. Wenn die Benutzerkonfiguration neu erstellt wird, sind Effekte standardmäßig auf aktivierte Animationen eingestellt. Das Löschen von `generated/` und das Ausführen von `argvus config apply` erstellt es Byte für Byte aus `config.json` erneut; das direkte Bearbeiten generierter Dateien ist nicht dauerhaft. 
Laufzeitdiagnosen müssen das vollständige Ausführungstuple identifizieren: Binärpfad, Version/Revision, Asset-Root, Konfigurations-Root, Daten-Root, generiertes Root, internes Root und Projektionsmanifest. `status --verbose` legt dieses Tuple offen und
vergleicht die veröffentlichte Generation/Hash mit der aktuell aufgelösten Konfiguration. Entwicklungs- und installierte-Paket-Tests sind separate Modi; keiner darf stillschweigend auf ein nicht verwandtes ausführbares Programm oder das Home-Verzeichnis des Entwicklers zurückfallen. `apply` ist auch die Abstimmungsgrenze für ältere kombinierte Themenkennungen: ein Wert wie `argvus-dark-spaced` wird zu `theme=argvus-dark` und `variant=spaced` vor der Auflösung. Die Identität des verpackten Assets wird in dieser Veröffentlichung nicht in einem Manifest aufgezeichnet, daher beschreibt keine Diagnosedatei diese.
