---
title: ARGVUS Fokus
description: Pomodoro- und Fokussitzungen mit Pausen, Benachrichtigungen und einer lokalen Sitzungsverlauf.
---
ARGVUS Focus ist eine GTK-4-Anwendung zum Arbeiten in Fokus-Sitzungen, getrennt durch Pausen. Sie öffnet sich als Popup neben der Taskleiste. Öffnen Sie sie über den Launcher, indem Sie auf den Taskleisten-Indikator klicken, mit **SUPER + Shift + F** oder mit:

```sh
argvus focus              # open the popup near the pointer
argvus focus --toggle     # hide the popup if it is visible, show it otherwise
argvus focus --toggle --x 1600 --y 30  # open next to a taskbar click position
argvus focus --start      # start or resume the current phase without opening the window
argvus focus --pause      # pause the running phase
argvus focus status --waybar         # print one Waybar JSON status and exit
argvus focus status --waybar --follow  # stream status changes for Waybar
```
Die Taskleiste übergibt die Root-Klick-Koordinaten, sodass das Popup den angeklickten Monitor verwendet und mit der Leiste ausgerichtet wird; ein Start ohne Koordinaten verwendet die Zeigerposition. Wenn der Zeiger 5 Sekunden lang außerhalb des Popups bleibt, schließt es sich automatisch; das Bewegen des Zeigers in das Popup bricht diesen Countdown ab. **Esc** auf der Hauptseite blendet es weiterhin sofort aus, und **Esc** auf einer Unterseite kehrt zur vorherigen Seite zurück. Der Taskleistenanzeiger zeigt im Leerlauf nur sein Symbol und die verbleibende `MM:SS` während einer Phase, mit der Phasenfarbe; eine pausierte Phase behält ihre verbleibende Zeit und fügt einen pausierten Zustand hinzu. Der Befehl `status` wird innerhalb von `argvus` ausgeführt, ohne GTK zu starten. `argvus --focus` ist ein Alias. Pro Sitzung läuft nur eine Focus-Instanz: Ein zweiter Start übergibt seine Optionen an die bereits laufende Instanz.

ARGVUS Focus, ARGVUS Clock und das Bedienfeld teilen sich denselben Bildschirmrand. Das Öffnen von Focus schließt zuerst Clock und das Bedienfeld; das Öffnen einer dieser Oberflächen schließt Focus.

## Sitzungen

- **Pomodoro**-Modus wechselt zwischen Fokus und kurzen Pausen und beendet jeden Zyklus mit einer langen Pause. Die Standardlängen betragen 25, 5 und 15 Minuten, mit einer langen Pause nach 4 Fokussitzungen.
- **Einzelsitzung**-Modus läuft eine einzelne Fokussitzung in ihrer eigenen Länge (standardmäßig 45 Minuten) ohne Pausen.
- Das Fenster zeigt die Phase (Fokus, kurze Pause, lange Pause, jeweils in eigener Farbe), die verbleibende Zeit im Fortschrittsring, die Position im Zyklus und die Summe für heute.
- **Fokus starten / Pause starten**, **Pause**, **Fortsetzen**, **Sitzung abbrechen** (beendet die Phase und zeichnet nichts auf) und **Jetzt beenden** (beendet die Fokussitzung frühzeitig und zeichnet die tatsächlich verbrachte Zeit auf). Während einer Pause bringt **Pause überspringen** zurück zum Fokus.
- Wenn eine Phase endet, wartet die nächste auf Sie, es sei denn, **Pausen automatisch starten** oder **Fokus automatisch nach einer Pause starten** ist aktiviert.

Tastatur: **Leertaste** startet oder pausiert, **Strg+,** öffnet die Einstellungen, **Strg+H** die Historie, **Esc** geht zurück, **Strg+W** versteckt das Fenster und **Strg+Q** beendet das Programm.

## Einstellungen

Presets (Klassisch 25/5/15, Erweitert 50/10/30, Tiefarbeit 90/20/30) füllen die Längen aus, die weiterhin bearbeitbar bleiben. Die Einstellungen beinhalten auch die Sitzungen vor einer langen Pause, die automatischen Starts, die Benachrichtigung am Ende einer Phase und die **Sitzungstöne**: eine kurze Glocke mit eigener Lautstärke und eine **Testton**-Taste. Diese Töne sind getrennt von den Alarmtönen von [ARGVUS Clock](/docs/components/argvus-clock/).

## Historie

Jede abgeschlossene Fokussitzung wird mit Start, Ende und verbrachter Zeit aufgezeichnet; Pausen und abgebrochene Sitzungen nicht. Die Historie-Seite zeigt die heutigen Sitzungen und Minuten, die Gesamtsummen und die letzten Sitzungen. Aufzeichnungen, die älter als 365 Tage sind, werden gelöscht. **Historie löschen** fragt nach Bestätigung.

## Fenster, Hintergrund und Suspendieren

- Das Schließen des Fensters während eines Zyklus versteckt es nur: Die Sitzung läuft weiter und benachrichtigt Sie, wenn die Phase endet. Ohne laufenden Vorgang beendet das Schließen das Programm.  
- Wird das System während einer laufenden Phase in den Ruhezustand versetzt, wird die Phase zum Zeitpunkt des Ruhezustands pausiert, und eine Nachricht zeigt an, wie lange das System geschlafen hat. Schlafen wird nicht als Fokuszeit gezählt.  
- Die Sitzung wird bei jeder Änderung und alle 30 Sekunden gespeichert. Wird ARGVUS Focus während einer Phase geschlossen (Abmeldung, Absturz, **Ctrl+Q**), wird sie beim nächsten Öffnen an dieser Stelle pausiert wieder angezeigt; sie gibt niemals vor, dass der Timer weiterlief, während er keine Benachrichtigung abgeben konnte.  
- Die Zeiten werden nach der Systemstart-Uhr gemessen, sodass eine Änderung der Wanduhr oder der Zeitzone eine Phase weder verkürzt noch verlängert.  

## Pfade

| Pfad | Inhalt |
| --- | --- |
| `$XDG_CONFIG_HOME/argvus/data/focus/settings.json` | Einstellungen |
| `$XDG_DATA_HOME/argvus/focus/history.json` | Abgeschlossene Sitzungen |
| `$XDG_STATE_HOME/argvus/focus/session.json` | Kontrollpunkt der aktuellen Sitzung |
| `/usr/share/argvus/sounds/argvus-bell.oga` | Sitzungsglocke |

Dateien werden atomar geschrieben. Eine Datei, die nicht gelesen werden kann, wird in `<name>.corrupt-<time>` umbenannt und die Standardeinstellungen werden verwendet; eine Nachricht teilt Ihnen mit, wohin sie gegangen ist.

Die Farben folgen dem aktiven ARGVUS-Thema und der Hervorhebungsfarbe (Themenverbraucher `time`). Zeichenketten stammen aus den Übersetzungsdomänen `focus` und `time`.

Die Taskleisten-Kalender, ARGVUS Focus, ARGVUS-Uhr und das Kontrollfeld sind gegenseitig exklusiv: Das Öffnen eines Elements schließt die anderen.
