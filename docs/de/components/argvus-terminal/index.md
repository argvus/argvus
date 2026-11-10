---
title: Terminal
description: Öffne das ARGVUS-Terminal und die Terminal-Anwendungsprofile.
---
Öffnen Sie das konfigurierte Terminal mit:

```sh
argvus --terminal
```
`argvus terminal` (ein Unterbefehl von `argvus`) stellt die Terminalintegration und die Kitty-Konfiguration bereit. `argvus-app-profiles` liefert Profile für Terminalanwendungen wie Superfile und Yazi; es stellt jedoch nicht den `argvus`-Dispatcher bereit.

`argvus-config` projiziert das Thema in `data/generated/terminal/`, und `argvus --yazi` und `argvus --spf` lesen ihre Konfiguration aus den projizierten `data/generated/yazi/`- und `data/generated/superfile/`-Bäumen anstatt aus Kopien, die in Ihrem Home-Verzeichnis materialisiert wurden. Eine vollständige native Überschreibung im eigenen Konfigurationsverzeichnis der Anwendung hat jedoch weiterhin Vorrang, sodass Sie den projizierten Baum durch Ihre eigene Konfiguration ersetzen können.

Themenänderungen regenerieren die von Kitty abgeleiteten Dateien unter `$XDG_CACHE_HOME/argvus/argvus-terminal`. Der Generator serialisiert Updates und ersetzt jede Datei atomar. Wenn das Terminal immer noch nicht geöffnet wird, überprüfen Sie das aktive Thema und regenerieren Sie den Cache mit:

```sh
argvus config get /appearance/theme --raw
argvus terminal --apply
argvus terminal --print-config
```
Das Terminal liest zunächst das verwaltete Thema und den Akzent von `$XDG_CONFIG_HOME/argvus/config.json`. Die veralteten Dateien `.active-theme` und `.accent-color` bleiben als Fallbacks für ältere Installationen bestehen.

## Transparenz und Unschärfe

Unter **Kontrollzentrum → Erscheinungsbild → Terminal** können **Transparenz** und **Unschärfe** unabhängig von `0%` bis `100%` konfiguriert werden. Änderungen werden mit der **Übernehmen**-Schaltfläche übernommen und erzeugen das Kitty-Profil unter `$XDG_CACHE_HOME/argvus/argvus-terminal` neu; bestehende Kitty-Fenster erhalten die aktualisierte Konfiguration durch ein Neuladen.

`0%`-Transparenz hält den Hintergrund undurchsichtig und `100%` macht ihn vollständig transparent. Unschärfe verwendet den von Kitty unterstützten Hintergrund-Unscharf-Radius und hat nur eine sichtbare Wirkung, wenn das Fenster transparent ist.

## Benachrichtigung bei lang laufenden Befehlen

Befehle, die 30 Sekunden oder länger laufen, senden eine Benachrichtigung, wenn sie beendet sind, solange das Terminalfenster nicht im Fokus ist. Die Benachrichtigung verwendet Kittys `notify_on_cmd_finish` mit der bereits aktivierten Shell-Integration, sodass `~/.zshrc` und `~/.bashrc` nicht geändert werden müssen. Die 30-Sekunden-Grenze ist in der Kitty-Konfiguration festgelegt.

## TUI-App-Modus

ARGVUS Control Center und Taskleisten-TUI-Popups verwenden `argvus tui-terminal` mit einem dedizierten Kitty-App-Profil. Es übernimmt das aktive ARGVUS-Thema und die Schriftart, weist eine stabile Fensterklasse zu und beendet sich zusammen mit seinem Kind anstelle eines Shell-Öffnens. Das Control Center blendet Tabs aus und hat unabhängige Transparenz- und Unschärfewerte; Taskleistenmonitor- und Temperatur-Popups verwenden eine untere Tab-Leiste für ihren lokalisierten `Press q to close`-Titel. Der Launcher akzeptiert `--title TITLE`, um einen festen Kitty-Fenster- und Tab-Titel zu setzen. Das normale Terminal behält seine Tabs und Einstellungen.

Wenn LANG in der Launcher-Umgebung fehlt, liefert `argvus terminal` das persönliche Gebietsschema von `$XDG_CONFIG_HOME/locale.conf` (standardmäßig Englisch), bevor Kitty die Tastaturkomposition initialisiert. Bestehende Überschreibungen der Gebietsschema-Umgebung werden beibehalten. Starter Kitty nach der Aktualisierung neu: Das Exportieren von LANG innerhalb einer bestehenden Shell initialisiert die Kompositionstabelle von Kitty nicht neu.

Leere LC_ALL und LC_CTYPE, die von der Sitzungsinitialisierung übernommen werden, werden aus der Kitty-Kindumgebung entfernt. Andernfalls behandelt Kitty ein leeres Gebietsschema als explizit und kann seine Kompositionstabelle nicht laden, selbst mit einem gültigen LANG. Nichtleere Überschreibungen werden beibehalten. Dies gilt auch für neue Profile und erfordert das erneute Öffnen von Kitty nach der Aktualisierung des Launchers.

Das dedizierte Kontrollzentrum-Fenster (`argvus-control-center`) erhält die pro Start festgelegte Kitty-Überschreibung `confirm_os_window_close=0`, sodass die Schließen-Aktion des Compositors nicht nach einer Bestätigung fragt. Die Überschreibung wird nicht in das gemeinsame Terminalprofil geschrieben; andere TUIs und normale Terminals behalten ihre bestehende Richtlinie bei.
