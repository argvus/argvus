---
title: Repository-Rollen
description: Verantwortlichkeiten der ARGVUS-Quellrepositories nach der Konsolidierung des Arbeitsbereichs.
---
Die Repositories sind nach Verantwortungsbereich und nicht nach Navigationsseite gruppiert:

- Arbeitsbereich: `argvus` (Dispatcher, In-Process-Kommandos, GTK 4-Helfer, gemeinsame Crates, der Greeter in `crates/argvus-greeter` und die Daten der Kommandos);
- Sitzung und Integration: `argvus-session`, `argvus-hyprland`, `argvus-portal`, `argvus-lock`, `argvus-boot-splash` und der Greeter (`argvus-greeter`, gebaut aus `argvus/crates/argvus-greeter`);
- UI außerhalb des Arbeitsbereichs: Systemsteuerung, Taskleiste (`argvus-taskbar`), Launcher und Waybar;
- Anbieter: Display, Stromversorgung, Firewall, Benachrichtigungen;
- Assets und Verpackung: Erscheinungsbild, Hintergründe, Schriftarten, Symbole, Profile.

## Konsolidiert im Arbeitsbereich

Diese Repositories wurden in `de/argvus` zusammengeführt. Ihre Dokumentation befindet sich jetzt unter `docs/` des Arbeitsbereichs, und ihre Repositories werden nur noch aus historischen Gründen aufbewahrt:

| Ehemaliges Repository | Jetzt | Dokumentation |
| --- | --- | --- |
| `argvus-config` | `argvus config` | [Befehlszeile](/docs/reference/command-line/), [Konfigurationsdateien](/docs/reference/configuration-files/) |
| `argvus-i18n` | `argvus i18n` | [Internationalisierungskomponente](/docs/components/argvus-i18n/) |
| `argvus-keys`, `argvus-projects` | `argvus keys`, `argvus projects` | Befehlszeile |
| `argvus-terminal` | `argvus terminal` | [Terminalkomponente](/docs/components/argvus-terminal/) |
| `argvus-system-monitor` | `argvus system-monitor` | [Systemüberwachungskomponente](/docs/components/argvus-system-monitor/) |
| `argvus-network` | `argvus network`, `argvus bluetooth` | [Netzwerkkonponente](/docs/components/argvus-network/) |
| `argvus-widget-telemetry` | `argvus widget-telemetry` | Befehlszeile |
| `argvus-control-center` | `argvus control-center` (Terminalanwendung) | [Kontrollzentrumskomponente](/docs/components/argvus-control-center/) |
| `argvus-taskbar-calendar` | `argvus taskbar-calendar` (GTK 4-Helfer) | [Taskleisten-Kalenderkomponente](/docs/components/argvus-taskbar-calendar/) |
| `argvus-removable-devices` | `argvus removable-devices` (GTK 4-Helfer) | [Wechselbare Gerätekomponente](/docs/components/argvus-removable-devices/) |
| `argvus-loading-theme` | `argvus loading-theme` (`splash`-Helfer) | Befehlszeile |
| `argvus-tui-terminal` | `argvus tui-terminal` (Kiste im Arbeitsbereich) | Befehlszeile |
| `argvus-tui`, `argvus-theme` | Geteilte Kisten `crates/argvus-tui` und `crates/argvus-theme` | Entwicklern Hinweise |
| `argvus-greeter` | Quellcode in `crates/argvus-greeter`, geliefert im `argvus`-Paket | [Begrüßungskomponente](/docs/components/argvus-greeter/) |
| `argvus-themes` | Außer Dienst. Jedes offizielle Thema ist sein eigenes `argvus-theme-*`-Paket. | [Themen installieren](/docs/user-guide/appearance/) |

Die Entwicklernnotizen der konsolidierten Komponenten befinden sich in `docs/internal/<repository>/`.

Sprache ist nicht die Eigentumsgrenze: Jede Komponente behält ihre Laufzeithelfer mit ihrem Paket. Die eine strikte Regel, die alle Pakete durchschneidet: Nur `argvus-config` schreibt unterhalb von `data/generated/`. Eine Komponente, die auf eine Anwendung zugreifen muss, die `config.json` nicht lesen kann, fügt in ihrem eigenen Helfer einen Adapter hinzu und lässt den Orchestrator ihn nach dem kanonischen Commit aufrufen; sie schreibt niemals eine generierte Datei. Öffentliche CLIs gehören in `/usr/bin`; interne Helfer gehören unter `/usr/lib/argvus/<component>` oder in das Befehlsverzeichnis, wenn sie zur Laufzeit benötigt werden; architekturunabhängige Daten bleiben unter `/usr/share/argvus/<component>`.

Siehe die [package matrix](/docs/reference/package-matrix/) für die installierte Paketgrenze.
