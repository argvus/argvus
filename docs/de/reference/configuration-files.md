---
title: Konfigurationsdateien
description: Benutzer-, System-, paketstandard- und generierte ARGVUS-Konfiguration.
---

| Ebene | Speicherort | Bedeutung |
| --- | --- | --- |
| System | `/etc/argvus/` | Administrator-Konfiguration für Dienste und Anbieter. |
| Paket-Standards | `/usr/share/argvus/` | Nur-Lese-Komponentenstandards und Laufzeitressourcen. |
| Benutzer | `$XDG_CONFIG_HOME/argvus/config.json` | Kanonische portable Einstellungen. |
| Verwaltete Daten | `$XDG_CONFIG_HOME/argvus/data/` | Komponentendaten, Kompatibilitätsdateien und interne Metadaten. |
| Generiert | `$XDG_CONFIG_HOME/argvus/data/generated/` | Projektionen von `config.json`, nur geschrieben von `argvus-config`. Niemals bearbeiten, niemals anderswo als Konfiguration kopieren und niemals als Quelle wieder einlesen. |
| Laufzeit | `$XDG_RUNTIME_DIR/` und `$XDG_CACHE_HOME/` | Sockets, Sperren und vorübergehender Zustand. |

Der aktive Stil, Akzent, benutzerdefinierte Hintergrundbilder und Effekte sind Logischer Benutzerzustand. Das verwaltete logische Modell ist in [Canonical configuration](/docs/reference/argvus-config/) dokumentiert. Generierte Hyprland-, GTK-, Qt6ct-, Waybar-, Quickshell-, Dunst-, Hyprlock-, Yazi-, Superfile-, Terminal-, Schriftarten- und Wechseldatenträgerdateien sollten nicht als unabhängige Wahrheitsquellen betrachtet werden.

`argvus-config` spiegelt auch die Theme-Ebene in `data/waybar/argvus-taskbar.{jsonc,css}` und `data/waybar/argvus-widget-telemetry.{jsonc,css}`. Diese Kopien werden bei einer Erscheinungsänderung durch die verpackten Standardwerte ersetzt, anschließend werden ihre abgegrenzten verwalteten Blöcke neu geschrieben, daher sollten sie als abgeleitet und nicht als von Hand gepflegt behandelt werden. Die native Anwendungskonfiguration unter `$XDG_CONFIG_HOME/<app>` ist anders: Sie ist ein Adapterziel, das von den Erscheinungsorchestratoren geschrieben wird, niemals vom Projektor.

## Ausnahme zur Single-Writer-Regel

`argvus config` ist der einzige Verfasser von `data/generated/`. Eine Ausnahme gibt es für die Migration: wenn eine Benutzerkopie einer gepackten Datei erstellt wird, wird eine Datei, die eine ältere Version unter `data/generated/` hinterlassen hat, an ihren Benutzerort (`data/`) verschoben, damit der Benutzer seine Änderungen behält. Die Verschiebung erfolgt nur für Dateien, die die aktuelle Version nicht erzeugt. Verlassen Sie sich nicht auf diesen Pfad für neue Dateien: Fügen Sie eine generierte Datei in der Projektion hinzu, nicht hier.
