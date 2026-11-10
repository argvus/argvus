---
title: Architekturübersicht
description: Wie ARGVUS eine integrierte Desktop-Umgebung bildet.
---
ARGVUS hat fünf Laufzeitebenen:

1. Einstiegspunkte und Dispatch in `argvus`;
2. Sitzungslebenszyklus in `argvus-session`;
3. Compositor- und Shell-Oberflächen in `argvus-hyprland`, `argvus-waybar`, `argvus-taskbar` und `argvus-control-panel`;
4. Benutzeranwendungen wie Kontrollzentrum, Kalender und Systemmonitor;
5. Anbieter und Ressourcen für Anzeige, Netzwerk, Stromversorgung, Aussehen, Konten und Benachrichtigungen.

Geteilter Zustand verbindet diese Ebenen. `argvus-config` hält das kanonische Dokument und ist der einzige Schreiber von `data/generated/`, sodass eine Themen- oder Effektsänderung die Dateien, die von mehreren Paketen in einer Transaktion verwendet werden, neu generiert; Sitzungsdienste bieten Lebenszyklus-Verantwortung, ohne die Verantwortung für die Implementierung jeder Komponente zu übernehmen.

## Verantwortungsgrenzen

| Schicht | Hauptverantwortung | Beispiele |
| --- | --- | --- |
| Einstiegspunkte und Abruf | Desktop-Anwendungen starten und ARGVUS-Profile auflösen. | `argvus`, `argvus-app-profiles` |
| Sitzungslebenszyklus | Die Umgebung vorbereiten, Hyprland starten, die Benutzerumgebungsdienste synchronisieren und den eigenen Sitzungsabbruch durchführen. | `argvus-session`, `argvus sessionctl` |
| Compositor und Shell | Fensterverhalten und sichtbare Desktop-Oberflächen definieren. | `argvus-hyprland`, `argvus-waybar`, `argvus-taskbar`, `argvus-control-panel` |
| Konfiguration | Besitz von `config.json`, dem Pfadmodell und jeder generierten Verbraucherdatendatei. | `argvus-config` |
| Anwendungen | Einstellungen und fokussierte Desktop-Workflows bereitstellen. | Systemsteuerung, Kalender, Launcher, Systemmonitor und Wechselmedien |
| Anbieter und Ressourcen | Domänenoperationen und gemeinsame visuelle Ressourcen implementieren. | Anzeige, Netzwerk, Energie, Benachrichtigungen, Erscheinungsbild, Schriftarten und Hintergrundbilder |

Das `argvus`-Paket ist das Installationsziel und der Koordinator, aber es ist keine monolithische Laufzeitumgebung. Fokussierte Pakete behalten das Eigentum an ihren Binärdateien, der Konfiguration und den Diensten. Der Sitzungsmanager startet sie zum entsprechenden Zeitpunkt in der grafischen Sitzung, ohne deren Implementierung zu übernehmen.

Diese Unterscheidung ist beim Debuggen wichtig: ein Sitzungsfehler gehört zunächst zu `argvus-session`; ein Taskleistenfehler gehört zu `argvus-taskbar`/`argvus-waybar`; und ein Fehler bei einem generierten Thema oder Wallpaper gehört zum Erscheinungs- und Sitzungs-Konfigurationspfad.
