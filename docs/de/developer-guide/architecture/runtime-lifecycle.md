---
title: Laufzeit-Lebenszyklus
description: Sitzungsstart und Benutzer-Service-Eigentümerschaft.
---
`argvus-session` importiert die Umgebung, startet Hyprland über `argvus start` und startet `argvus-session.target`, nachdem der Compositor bereit ist. Das Ziel zieht Shell-, Benachrichtigungs-, Hintergrundbild-, Leerlauf- und Zwischenablagedienste heran. `argvus sessionctl` koordiniert Start, Stopp, Neustart, Laden, Status und Protokolle.

Die Sitzung stoppt das Ziel, wenn Hyprland beendet wird. Einzelne Projekte bleiben für ihre eigenen Binärdateien und Konfigurationen verantwortlich.

## Start

Der grafische Einstiegspunkt ist `/usr/share/wayland-sessions/argvus.desktop`, der `argvus session` aufruft. Die Sitzung importiert die ARGVUS- und XDG-Konfigurationsumgebung, synchronisiert sie mit `systemd --user` und der D-Bus-Aktivierung und delegiert den Compositor-Start an `argvus start`.

`argvus start` überprüft die Hyprland-Konfiguration, wendet das bereitgestellte Fallback an, wenn eine standardmäßig generierte Datei gefunden wird oder eine Benutzerkonfiguration ungültig ist, erkennt die Virtualisierungskompatibilität und startet Hyprland. Die Bereitstellungsbrücke des Compositors ruft dann `argvus sessionctl ready` auf, das das ARGVUS-Ziel mit der aktuellen Wayland- und Hyprland-Umgebung startet.

Das Ziel verwendet eine einmalige Vorbereitungseinheit, bevor lang laufende Komponenten gestartet werden. Die Vorbereitung wendet die generierte Konfiguration, das Themen- und Akzentstatus, den Monitorstatus, die Anwendungsintegration und die Sitzungsumgebung an. Persistente Dienste werden von `systemd --user` verwaltet, nicht von ad-hoc Hintergrundprozessen.

`argvus start` reiht `argvus-config.service` (`--no-block`) direkt davor ein, Hyprland zu starten: Die Projektion benötigt keinen Compositor, daher überschneidet sie den Compositor-Start. `argvus sessionctl ready` startet dann das Ziel, das feststellt, dass die Einheit aktiv ist, oder auf seinen Job wartet, sodass die kanonische Konfiguration noch vor der Ausführung der Vorbereitung projiziert wird. Sobald das Ziel gestartet ist, wartet `sessionctl ready` bis zu 1,5 Sekunden auf das Hintergrundbild (eine Hintergrundlage-Oberfläche) und die Taskleiste, bevor es die Ladeüberlagerung entfernt, weil ein einfacher Dienst als gestartet zählt, bevor er zeichnet; eine Oberfläche, deren Einheit nicht läuft, wird nicht abgewartet, und das Warten wird als `session.desktop-surfaces` protokolliert, mit `session.wallpaper-surface` und `session.taskbar-surface` für die Zeit, die jede Oberfläche benötigte. Ein schwerwiegender Projektions- oder Zielstartfehler wird protokolliert und stoppt die unabhängige Ladeüberlagerung, um einen scheinbar endlosen Spinner zu verhindern. Optional Visuelle Benutzer müssen eine Fallback-Lösung finden oder einen verschlechterten Fehler melden, ohne das Sitzungziel zu ersetzen.

### Anmeldekosten

Eine Anmeldung ohne Änderung der Konfiguration wiederholt keine aufwendigen Arbeiten:

- Die Projektion führt den Themenwechsel und den Effektadapter nur aus, wenn sich die Abschnitte Aussehen oder Effekte geändert haben oder wenn deren reale Ausgaben (die materialisierten Benutzerkopien und `data/state/effects/<theme>.conf`) fehlen.
- Die Vorbereitung wendet das Startthema und den darauf liegenden Akzent nur an, wenn der Stempel in `$XDG_CACHE_HOME/argvus/session/theme-startup.stamp` nicht mehr mit den gepackten Themendateien und dem Akzentmarker übereinstimmt. Der Stempel erstellt Fingerabdrücke der Dateiinhalte, sodass das Neuinstallieren identischer Dateien ihn nicht ungültig macht. Geänderte Theme-Eingaben oder ein neuer Akzent machen ihn einmal ungültig. Das Migrieren eines älteren stempelbasierten Zeitstempels baut ihn ebenfalls einmal neu auf. Wenn das Thema erneut angewendet wird, lädt die Vorbereitung auch Hyprland neu, das auf den vorherigen Dateien gestartet wurde.
- Die GTK-Einstellungen (`gsettings`) werden nur geschrieben, wenn das Startthema erneut angewendet wurde. Zwischen den Anmeldungen behält dconf sie, sodass eine stabile Anmeldung sie nicht erneut ausführt. 
- `argvus-dunst.service` ist `Type=dbus`: es gilt als gestartet, sobald Dunst `org.freedesktop.Notifications` besitzt, sodass das Sitzungziel nicht darauf wartet, dass der Pausenzustand wiederhergestellt wird. 
- Die Vorbereitung läuft `spaces-switch.sh` und `borders-switch.sh` (`--apply-static`) nebeneinander: sie schreiben getrennte Dateien (die JSONC-Ränder und effektive Geometrie der Leisten sowie die CSS-Dateien der Leisten und das Rofi-Thema). 
- Die Projektion schreibt eine generierte Datei nur dann neu, wenn sich ihr Inhalt geändert hat. Unveränderte Dateien bleiben unberührt, sodass eine stabile Anmeldung sie nicht erneut fsync-ed. 
- Der Anzeige-Schritt der Vorbereitung (`argvus config display session-prepare`) gilt nur für DPMS und deaktivierte Ausgänge: er projiziert weder erneut noch lädt er den Kompositor neu.
- `argvus-session-reload.service` führt `argvus sessionctl reload-desktop --login` aus, das Hyprland nur dann neu lädt, wenn das Projektionsmanifest `hyprland` unter den betroffenen Verbrauchern aufführt. Die Desktop-Komponenten warten nicht darauf.

Der Sitzungsspinne (der Handoff-Helfer im Wayland-Modus) wird vom Kompositor-Handoff abgebildet, bis `argvus sessionctl ready` das Ziel gestartet hat, sodass ein einzelner Spinner den gesamten Startvorgang abdeckt. Er hat kein `PartOf=`, da das Erstellen des Ziels bei einer neuen Anmeldung es nicht herunterfahren darf; `sessionctl` stoppt ihn explizit, und `RuntimeMaxSec=30` beendet ihn, wenn kein Stopp eintrifft.

Jeder Vorbereitungs-, Neu-Lade- und Projektionsadapter-Schritt hängt seine Dauer als `<epoch-ms> timing <step> <N>ms` an das Sitzungsprotokoll an; siehe [Fehlerbehebung beim Sitzungsstart](/docs/user-guide/troubleshooting/session-startup/).

Pfadauflösung, Sitzungs-Vorbereitung und der Hyprland-Neuladeplan werden von Rust verwaltet. `argvus config session-prepare` und `argvus config session-reload-config` führen die Start- und Neuladeschritte aus; das veraltete `hypr-init.sh`-Skript wurde entfernt.

## Dienste und Herunterfahren

Das Ziel besitzt normalerweise `argvus-control-panel.service`, `argvus-taskbar.service`, `argvus-widget-telemetry.service`, wenn aktiviert, `argvus-wallpaper.service`, `argvus-dunst.service`, `argvus-hypridle.service`, Zwischenablagebeobachter, Tastaturlayout-Benachrichtigungen und PolicyKit-Integration. Die Overlay zum Laden der Sitzung wird von `argvus start` gestartet, nicht vom Ziel. Ein Ausfall in einem optionalen Dienst ersetzt nicht die gesamte Sitzung.

Die Dienste, die eine Wayland-Client-Verbindung benötigen, einschließlich des Kontrollpanels und des Leerlaufmanagers, warten auf eine verwendbare Hyprland-Laufzeit, bevor sie ihren Client starten. Dies verhindert, dass Quickshell oder hypridle gegen eine veraltete Socket während des Compositor-Übergangs gestartet werden; wenn Hyprland ersetzt wird, kann systemd die Komponente erneut starten, nachdem die neue Laufzeit verfügbar ist.

Ein Themenwechsel serialisiert sein Commit, Build, Projektion und das Neuladen der Verbraucher als eine Lebenszyklusoperation unter der Themen-Sperre. Es lädt `argvus-config.service` nicht neu noch startet es Dienste neu: Die Projektion, die ausgeführt wird, schreibt nur Dateien und überlässt das Thema dem Wechsel, solange die Sperre gehalten wird, und die laufenden Verbraucher laden sich an Ort und Stelle neu (siehe [Appearance und themes](/docs/developer-guide/subsystems/appearance-and-themes/)).

Wenn Hyprland beendet wird, stoppt `argvus start` `argvus-session.target` und die damit verbundenen Dienste. Dies verhindert, dass Taskleiste, Shell, Benachrichtigung, Leerlauf- und Zwischenablageprozesse beim Abmelden überleben.

## Neu laden

```sh
argvus sessionctl reload
```

Reload importiert die grafische Umgebung erneut und **wendet** dann eine festgeschriebene Generation an, wenn ein vorhergehender Domain-Befehl wie `apply-theme`, `accent`, `set`/`patch` oder eine Änderung im Widget-Telemetrie-Block die Änderung bereits projiziert hat. In diesem Fall stellt es die im Manifestplan aufgeführten Verbraucher wieder her und ruft `argvus config project` nicht erneut auf, da eine erneute Projektion einen identischen Baum wieder aufbauen und den Plan verwerfen würde. Projektion, Migration und `ensure` werden nur ausgeführt, wenn kein gültiger ausstehender Plan existiert — beim Sitzungsstart oder nachdem `argvus config recover` das vorherige Dokument wiederherstellen muss. Ein sauberes No-Op ist nur erlaubt, wenn sowohl die Projektionen als auch die verwaltete Laufzeit aktuell sind. Wenn der kanonische Zustand unverändert ist, aber eine Compositor-Socket, ein Dienst oder ein Verbraucher veraltet ist, gleicht Reload dies anstelle eines vorzeitigen Rücksprungs aus. Die normale Verteilung deckt Hyprland, Taskleiste, Widget-Telemetrie, Systemsteuerung, Benachrichtigungen, Hintergrundbild und Leerlauf ab. laut dem Projektionsmanifest bleiben unveränderte optionale Verbraucher unberührt. Jede Datei unter `data/generated/` wird zu diesem Zeitpunkt von `argvus-config` geschrieben, sodass die Sitzung ein Thema niemals selbst neu rendert. Der Reload führt dann `theme-adapters.sh` aus, das die Verbraucher abgleicht, die das kanonische Dokument oder die veröffentlichte Projektion nicht selbst lesen können. Diese Adapter lesen nur die festgeschriebene Generation: Sie schreiben ihre eigenen Verbraucherdateien und Caches und schreiben niemals das kanonische Dokument oder `data/generated/`, sodass eine einzelne Benutzeraktion immer noch eine Mutation, eine Projektion, eine Veröffentlichung und einen Reload kostet. Verwenden Sie `argvus sessionctl restart <component>` nur, wenn ein einzelner eigener Dienst einen ausdrücklich isolierten Neustart benötigt. Prüfen Sie die [services-Referenz](/docs/reference/services/) für genaue Einheitsnamen.

Projektions- und Nachladeoperationen werden mit einem Benutzer-Laufzeit-Blockierer serialisiert. Für eine temporäre Zeitspur setzen Sie `ARGVUS_SESSION_PERF=1`, bevor Sie das Nachladen aufrufen, und prüfen Sie `~/.local/state/argvus/session.log`; die Spur zeichnet Phasengrenzen auf, ohne das normale Nachladeverhalten zu ändern.
