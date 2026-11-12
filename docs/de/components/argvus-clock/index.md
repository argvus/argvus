---
title: ARGVUS Uhr
description: Uhr, Weltuhren, Stoppuhr, Countdown-Timer und Alarme mit Ton.
---
ARGVUS Clock ist eine GTK 4-Anwendung mit der aktuellen Uhrzeit und den Zeitwerkzeugen: Weltuhren, eine Stoppuhr, ein Countdown-Timer und Alarme. Öffnen Sie sie über den Launcher, indem Sie auf die Uhrzeit in der Taskleiste klicken, mit **SUPER + Shift + C** oder mit:

```sh
argvus clock                     # open the popup near the pointer
argvus clock --toggle            # hide the popup if it is visible, show it otherwise
argvus clock --toggle --x 1600 --y 30  # open next to a taskbar click position
argvus clock --page alarms       # open on a page: clock, world, stopwatch, timer, alarms
argvus clock --background        # run without a window, to keep alarms active
```
ARGVUS Clock öffnet sich als Popup, ausgerichtet an der Taskleiste, mit einem 4-Pixel-Abstand. Die Taskleiste übermittelt die Root-Klick-Koordinaten, sodass das Popup den angeklickten Monitor verwendet; wird es ohne Koordinaten gestartet, verwendet es die Position des Zeigers. Bleibt der Zeiger 5 Sekunden lang außerhalb des Popups, schließt es sich automatisch; das Bewegen des Zeigers in das Popup bricht den Countdown ab. **Esc** auf einem Haupttab blendet es weiterhin sofort aus, und **Esc** auf einer Vollseite kehrt zum vorherigen Tab zurück. `argvus --clock` ist ein Alias. Ein Klick auf das Datum in der Taskleiste öffnet weiterhin den Kalender; ARGVUS Clock zeigt keine Ereignisse an.

ARGVUS Focus, ARGVUS Clock und das Kontrollpanel teilen sich dieselbe Bildschirmkante. Das Öffnen von Clock schließt zuerst Focus und das Kontrollpanel; das Öffnen einer dieser Oberflächen schließt Clock.
- **Uhr**: die Zeit in großen Ziffern (standardmäßig 24-Stunden-Format, 12-Stunden-Format in den Einstellungen, Sekunden optional), das lokalisierte Datum, die lokale Zeitzone und deren UTC-Versatz sowie der nächste Alarm.
- **Welt**: Städte, die durch die Suche nach einer Stadt oder einer IANA-Zeitzone hinzugefügt werden (Akzente und Groß-/Kleinschreibung werden ignoriert). Jede zeigt ihre Zeit, die Differenz zur lokalen Zeit und ob dort gestern, heute oder morgen ist. Die Sommerzeit wird aus der IANA-Zeitzonendatenbank übernommen; nichts wird mit festen Offsets berechnet. Städte können neu angeordnet und entfernt werden.
- **Stoppuhr**: starten, pausieren, fortsetzen, zurücksetzen und Runden (Rundenzeit und Gesamtzeit).
- **Timer**: Stunden, Minuten und Sekunden oder die Voreinstellungen 5, 10, 15, 25, 30 und 60 Minuten. Pausieren, fortsetzen, neu starten und abbrechen. Wenn er endet, klingelt er wie ein Alarm. Dieser Timer ist unabhängig von ARGVUS Focus-Sitzungen.
- **Alarme**: erstellen, bearbeiten, ein- und ausschalten sowie löschen von Alarmen. Jeder hat eine Uhrzeit, einen optionalen Namen, die Wochentage, an denen er sich wiederholt (keine bedeutet, er klingelt einmal), einen Ton und eine Schlummerdauer.

Tastatur: **Strg+1** bis **Strg+5** wechseln die Seiten, **Strg+N** erstellt einen Alarm, **Strg+,** öffnet die Einstellungen, **Esc** geht zurück, **Strg+W** schließt das Fenster und **Strg+Q** beendet das Programm.

## Alarmton

Alarme spielen einen echten Ton, bis Sie sie beantworten:

- Sounds: **Chime** (Standard) und **Beacon**, geliefert in `/usr/share/argvus/sounds/`.
- **Alarmlautstärke** (10 bis 100, Standard 80) und **Alarmton anhören** befinden sich in den Einstellungen; der Editor hat eine Testtaste für den gewählten Ton.
- Ein klingender Alarm zeigt eine Karte oben auf dem Bildschirm über anderen Fenstern und eine Benachrichtigung, beide mit **Schlummern** und **Verwerfen**. Mit Fokus auf der Karte bewirkt **Enter** Verwerfen und **S** Schlummern.
- Schlummern klingelt erneut nach der Schlummerdauer des Alarms (1 bis 60 Minuten, Standard 10).
- Ein unbeantworteter Alarm stoppt nach **Klingeln stoppen nach** (5, 10, 15 oder 30 Minuten, Standard 10) und hinterlässt eine Benachrichtigung.
- Zwei gleichzeitig klingelnde Alarme werden beide aufgelistet; der zweite stummt den ersten nie.
- Wenn der Ton nicht abgespielt werden kann, zeigen die Karte und das Fenster dies mit dem Fehler an, und der Fehler wird protokolliert. ARGVUS Clock meldet nie einen Ton, der nicht abgespielt wurde.

Die Wiedergabe verwendet die Medienunterstützung von GTK (GStreamer), sodass sie mit PipeWire, PulseAudio oder ALSA funktioniert. Das Paket hängt von `gst-plugins-base` (Ogg Vorbis) und `gst-plugins-good` (Audioausgabe) ab.

## Wenn Alarme klingeln

Alarme klingeln nur, solange ARGVUS Clock in Ihrer Sitzung läuft:

- Das Schließen des Fensters lässt ARGVUS Clock weiterlaufen, während Alarme, ein laufender Timer oder eine laufende Stoppuhr dies benötigen (**Weiterlaufen beim Schließen des Fensters**, standardmäßig aktiviert). Wenn nichts aussteht, beendet es sich.
- **Mit der Sitzung starten** aktiviert die Benutzereinheit `argvus-clock.service`, die `argvus clock --background` nach dem Anmelden ausführt. Standardmäßig ist sie ausgeschaltet.
- Alarme können nicht klingeln, während Sie abgemeldet sind, der Computer ausgeschaltet ist oder er sich im Ruhezustand befindet.
- Ein Alarm, der bis zu 5 Minuten zu spät bemerkt wird, klingelt trotzdem. Später als das (die Uhr lief nicht oder das System war im Schlafmodus oder ausgeschaltet) klingelt er nicht: er wird als **verpasst** in einer Benachrichtigung und auf der Alarmseite angezeigt. Ein wiederkehrender Alarm geht zu seinem nächsten Tag weiter; ein einmaliger Alarm schaltet sich aus.
- Jeder Alarm wird als bearbeitet gespeichert, bevor er klingelt, sodass das erneute Öffnen von ARGVUS Clock denselben Alarm nicht zweimal klingeln lässt und nur eine Instanz pro Sitzung läuft.
- Wiederkehrende Alarme werden in der lokalen Zeit an den gewählten Wochentagen berechnet, sodass sie ihre Zeit über Sommerzeitumstellungen hinweg behalten. Eine Zeit, die durch die Sommerzeitvorstellung übersprungen wird, klingelt in der ersten Minute nach der Lücke; eine Zeit, die zweimal vorkommt, klingelt beim ersten Mal.
- Ein Timer zählt während einer Unterbrechung weiter. Wenn er endete, während das System schlief, klingelt er beim Aufwachen und zeigt an, wie spät es ist. Nachdem ARGVUS Clock geschlossen wurde, kehrt ein laufender Timer oder Stoppuhr pausiert zurück.

## Pfade

| Pfad | Inhalt |
| --- | --- |
| `$XDG_CONFIG_HOME/argvus/data/clock/settings.json` | Einstellungen und Weltuhren |
| `$XDG_DATA_HOME/argvus/clock/alarms.json` | Alarme |
| `$XDG_STATE_HOME/argvus/clock/timers.json` | Timer- und Stoppuhr-Zwischenstopp |
| `/usr/lib/systemd/user/argvus-clock.service` | Mit der Sitzung starten |
| `/usr/share/argvus/sounds/` | Verpackte Sounds |

Dateien werden atomar geschrieben; eine Datei, die nicht gelesen werden kann, wird in `<name>.corrupt-<time>` umbenannt und Standardwerte werden verwendet. Die Farben folgen dem aktiven ARGVUS-Thema und der Highlight-Farbe (Theme-Consumer `time`). Zeichenketten stammen aus den Übersetzungsbereichen `clock` und `time`.

Der Kalender in der Taskleiste, ARGVUS Focus, ARGVUS Clock und die Systemsteuerung schließen sich gegenseitig aus: Das Öffnen eines Elements schließt die anderen.
