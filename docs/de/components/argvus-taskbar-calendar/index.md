---
title: Kalender
description: Verwenden Sie den ARGVUS-Taskleistenkalender.
---
`argvus-taskbar-calendar` ist das GTK4-Kalender-Popup, das vom Waybar-Datumsmodul geöffnet wird oder mit:

```sh
argvus --calendar
argvus taskbar-calendar toggle
argvus taskbar-calendar status
```
Es unterstützt lokale Ereignisse, Erinnerungen sowie den ICS-Import/Export. Sein Benutzerservice ist `argvus-taskbar-calendar.service`. Verwenden Sie `argvus taskbar-calendar --help` für die von der installierten Version unterstützten Optionen.

Der Kalender folgt dem aktiven ARGVUS-Thema, einschließlich der Varianten Sticky und Spaced. Themenänderungen werden über den generierten Benutzer-Cache angewendet; verwenden Sie `argvus taskbar-calendar reload`, wenn ein laufendes Popup ein explizites Aktualisieren benötigt.

Beim Öffnen über das Datumsmodul der Taskleiste ist das Popup am Rand der Leiste verankert: Es öffnet sich unter einer oberen Taskleiste und über einer unteren Taskleiste, ausgerichtet am rechten Rand des Datumsmoduls. Direkte Starts ohne Taskleisten-Geometrie verwenden den Zeiger oder Fallback des Monitors.

## Befehle

```bash
argvus taskbar-calendar                 open the calendar
argvus taskbar-calendar toggle          show or hide it (the Waybar date module uses this)
argvus taskbar-calendar show | hide
argvus taskbar-calendar config          open the system configuration for editing (sudo)
argvus taskbar-calendar config-path     print the configuration path
argvus taskbar-calendar reload
argvus taskbar-calendar import file.ics
argvus taskbar-calendar export --output calendar.ics
argvus taskbar-calendar sync
argvus taskbar-calendar status
argvus taskbar-calendar service
```
## Veranstaltungen und Erinnerungen

Veranstaltungen sind standardmäßig ausgeschaltet. Schalten Sie sie im Popup unter **Einstellungen > Veranstaltungen** ein; die Auswahl wird im Zustandsverzeichnis gespeichert und aktiviert außerdem `argvus-taskbar-calendar.service`, das die Erinnerungen anzeigt. Das Ausschalten stoppt den Dienst.
- **+ VERANSTALTUNG** (oder `n`) öffnet den Editor für den ausgewählten Tag. Veranstaltungen können ab heute erstellt und geändert werden; eine Veranstaltung eines vergangenen Tages kann weiterhin geöffnet und gelöscht werden.
- Eine neue Veranstaltung beginnt um 09:00 Uhr und dauert die in den Einstellungen festgelegte Standardlänge. Das Verschieben der Startzeit behält die Länge bei; eine Endzeit, die vor der Startzeit liegt, bedeutet, dass die Veranstaltung am nächsten Tag endet.
- Jede Veranstaltung zeigt beim Start eine Benachrichtigung mit Ton, unabhängig davon, was in den Kästchen **Erinnerung** und **Wiederholung** steht. Eine Veranstaltung, die nach ihrem Start gespeichert wird (z. B. eine ganztägige Veranstaltung, die während des Tages erstellt wird), benachrichtigt innerhalb von 30 Sekunden.
- **Erinnerung** fügt eine frühere Benachrichtigung hinzu, zur gewählten Zeit vor Beginn (zu Beginn, wenn diese Zeit bereits vergangen ist).
- Bei einer Ganztagesveranstaltung wird das Zeitfeld zu **UM DIEZEIT BENACHRICHTIGEN**: die Veranstaltung dauert den ganzen Tag und benachrichtigt zu dieser Zeit (00:00 benachrichtigt zu Beginn des Tages). Die Zeit wird beibehalten, wenn die Veranstaltung erneut geöffnet wird.
- Bei Ganztagesveranstaltungen benachrichtigt **Wiederholen** erneut im gewählten Intervall nach dieser Zeit.
- Das Bearbeiten einer Veranstaltung ohne Verschieben des Beginns wiederholt vorhandene, bereits angezeigte Erinnerungen nicht.
- Benachrichtigungen folgen der Kalendersprache (Einstellungen > Sprache).
- Ein Tag zeigt einen Punkt (`•`), wenn an diesem Tag eine Veranstaltung stattfindet, einschließlich jedes Tages einer über Nacht dauernden oder mehrtägigen Veranstaltung. Die Tage folgen der lokalen Zeitzone.
- Lokale Veranstaltungen werden automatisch 10 Minuten nach Ende entfernt. Wenn eine Veranstaltung bereits beendet ist, wenn sie gespeichert wird, bleibt sie mindestens 10 Minuten nach dem Speichern verfügbar.

Importierte `.ics`-Dateien behalten ihre Daten, Zeitzonen (`TZID`) und Alarme (`VALARM`).
Importierte Ereignisse benachrichtigen außerdem beim Beginn, neben ihren eigenen Alarmen. Wiederkehrende Ereignisse (`RRULE`)
werden gespeichert und exportiert, aber nur ihr erstes Vorkommen wird angezeigt und erinnert.
Nach einem Paket-Update beendet der Erinnerungsdienst seine Arbeit und systemd startet die neue Version
innerhalb von 30 Sekunden. `sync` meldet nur die konfigurierten entfernten Kalender; CalDAV-Synchronisation ist noch nicht implementiert.

## Pfade

- Konfiguration: `/etc/argvus/taskbar/calendar/config.toml` (systemweit; ein Standard wird mit dem Paket geliefert).
- Stile: `/etc/argvus/taskbar/calendar/style.css` und `theme.css`; die Farben des aktiven ARGVUS-Themes stammen aus dem Theme, das über den untenstehenden Cache aufgebaut wird.
- Daten: `$XDG_DATA_HOME/argvus-taskbar-calendar/` (oder `~/.local/share/argvus-taskbar-calendar/`).
- Zustand: `$XDG_STATE_HOME/argvus/taskbar/calendar/` (oder `~/.local/state/...`); die Ereignisumschaltung wird dort gespeichert.
- Cache: `$XDG_CACHE_HOME/argvus-taskbar-calendar/` (oder `~/.cache/argvus-taskbar-calendar/`).

Die Highlight-Farbe wird aus `~/.config/argvus/data/.accent-color` gelesen und zur Laufzeit
über den Farben des Themes angewendet.

Die automatische Sprachauswahl folgt dem Benutzer `locale.conf`, mit Englisch als Standard. Explizite Kalender-Sprachvorlieben überschreiben weiterhin die automatische Auswahl.

Die Taskleistenkalender, ARGVUS Focus, ARGVUS Clock und die Systemsteuerung schließen sich gegenseitig aus: Wenn eines geöffnet wird, werden die anderen geschlossen.

Fehler beim Lesen des Kalenders werden separat von einem leeren Terminplan angezeigt. Das Öffnen der Datenbank, das Laden täglicher Ereignisse und das Laden monatlicher Indikatoren haben jeweils eigene Meldungen. Wählen Sie einen Tag erneut aus, um einen neuen Versuch zu starten. Ein fehlgeschlagener Aktualisierungsversuch behält zuvor geladene Daten nur für denselben Tag oder Monat; gültige Ergebnisse der anderen Abfrage bleiben verfügbar.

## Gemeinsame Sprachkataloge

Die Benutzeroberfläche verwendet die gemeinsamen `argvus-i18n`-Kataloge für alle zehn Sprachen. Texte werden in `resources/locales/<locale>/taskbar-calendar.json` im Hauptarbeitsbereich gespeichert und durch `argvus` installiert. Englisch ist die Standardsprache; neue Sprachen erfordern kein lokales zweisprachiges Wörterbuch mehr.
