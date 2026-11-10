---
title: Nachtlicht
description: Passen Sie die Farbtemperatur und feste lokale Zeitpläne auf Hyprland an.
---
Öffnen Sie **Systemsteuerung > Desktop & Eingabe > Anzeigen > Nachtmodus**, suchen Sie nach **Nachtmodus** oder führen Sie `argvus control-center displays night-light` aus.

Der Nachtmodus und sein Zeitplan sind anfangs deaktiviert. Wählen Sie eine Temperatur zwischen **1000 und 20000 Kelvin** (Standard **4500 K**). Niedrigere Werte ergeben wärmere Farben. Die Seite zeigt die gespeicherte manuelle Aktivierung, den aktuellen Zustand und den nächsten Übergang. Eingaben bearbeiten einen Wert; Esc bricht ab oder kehrt zu Anzeigen zurück. Ungültige Werte werden vor dem Speichern abgelehnt. Änderungen werden sofort gespeichert und angewendet. Wenn die Anwendung nach dem Speichern fehlschlägt, bleibt die gespeicherte Einstellung neben dem tatsächlichen Zustand und dem Fehler sichtbar.

## Zeitplan und Schnellaktionen

Der feste Zeitplan verwendet die aktuelle lokale Zeitzone. Der Beginn ist eingeschlossen und das Ende ausgeschlossen; **20:00–07:00** überschreitet Mitternacht. Anfang und Ende müssen unterschiedlich sein und `HH:MM` verwenden. Es werden keine Standort- oder Sonnenberechnungen verwendet.

Verwenden Sie die **Anzeige**-Karte des Kontrollpanels oder **SUPER + Shift + B** für schnelle Umschaltungen. Der Helligkeitsregler behält alle seine Helligkeitsaktionen bei und fügt den Nachtmodus hinzu, wenn das Backend verfügbar ist, auch auf einem Display ohne Helligkeits-Backend. Es gibt kein permanentes Taskleistenmodul.

Bei deaktivierter Planung ist die manuelle Aktivierung dauerhaft. Bei aktivierter Planung erstellt eine schnelle Umschaltung eine temporäre Überschreibung bis zur nächsten geplanten Änderung. Das Ändern des Plans oder der Temperatur löscht diese Überschreibung. Logout, eine neue Compositor-Instanz oder eine geänderte nächste Übergangszeit (einschließlich einer Zeitzonenänderung) machen sie ebenfalls ungültig.

## Befehle und Status

Alle Schnittstellen verwenden denselben öffentlichen Befehl:

```sh
argvus config display night-light status
argvus config display night-light enable
argvus config display night-light disable
argvus config display night-light toggle
argvus config display night-light temperature 4500
argvus config display night-light schedule on 20:00 07:00
argvus config display night-light schedule off 20:00 07:00
argvus config display night-light apply
argvus config display night-light apply-schedule
```
`status` gibt JSON-Schema Version 1 zurück: `available`, `unavailable_reason`, `settings` (aktiviert, Temperatur, Zeitplan_aktiviert, Start, Ende), `active`, `effective_temperature`, `mode` (`manual` oder `scheduled`), `manual_override`, `next_transition` (ISO 8601 mit Offset), `minimum_temperature` und `maximum_temperature`. Effektive Felder sind null, wenn sie nicht verfügbar sind; effektive Temperatur ist null, wenn die Identität aktiv ist. Verfügbarkeitsfehler werden im Status gemeldet; Aktionsfehler geben einen ungleich null mit einer Diagnose zurück.

Die kanonische Präferenz ist `/displays/night_light`, atomar von `argvus config` in `$XDG_CONFIG_HOME/argvus/config/displays.json` gespeichert. Volatile Überschreibungen und die Aktionssperre befinden sich unter `$XDG_RUNTIME_DIR/argvus/night-light/`. Generierte Monitor-Dateien sind unabhängig. Bearbeiten Sie generierte Dateien nicht, um diese Funktion zu konfigurieren.

## Laufzeit

Erfordert Hyprland und **hyprsunset >= 0.4.0**. Die Sitzung startet einmal den Upstream `hyprsunset.service`, wendet gespeicherten Zustand über `argvus-night-light-init.service` an und wertet Zeitpläne jede Minute über `argvus-night-light.timer` und `argvus-night-light.service` aus. Änderungen von Uhrzeit und Zeitzone lösen ebenfalls eine Auswertung aus. Mit ausgeschaltetem Zeitplan lässt der periodische Betrieb den manuellen Zustand unverändert. Wiederholte Anwendungen fragen die effektive Identität/Temperatur ab und überspringen unveränderte Schreibvorgänge. Kein Kompositor-Neustart ist erforderlich.

ARGVUS liefert ein systemd-Drop-in für den Upstream-Dienst: Es beginnt mit `--identity` und isoliert seine Konfigurationssuche auf `/usr/share/argvus/session/config/night-light`. Dies verhindert konkurrierende native Profile und lässt persönliche `hyprsunset.conf`-Dateien unberührt. Das Drop-in gilt für diesen Upstream-Dienst, solange das ARGVUS-Session-Paket installiert ist, einschließlich Starts außerhalb von ARGVUS. Bestehende systemd-Overrides des Benutzers können Vorrang haben; prüfen Sie `systemctl --user cat hyprsunset.service`, wenn Sie diese Situation diagnostizieren. Hyprsunset 0.4.0 lehnt das separate Pfad-Argument von `--config` ab, daher wird die Konfigurationssuche stattdessen mit service-lokalen XDG-Variablen isoliert.

Das Backend verwendet das offizielle Hyprsunset IPC (`temperature`, `identity`, `identity get`) mit begrenztem I/O und Rücklesen nach Änderungen. Es werden kein Shader, kein alternatives Compositor-Backend, keine Systemkonfigurationsschreibungen oder Privilegienerhöhungen verwendet. Nicht verfügbare Steuerelemente sind im Kontrollzentrum deaktiviert und in der Anzeige-Karte ausgeblendet.
