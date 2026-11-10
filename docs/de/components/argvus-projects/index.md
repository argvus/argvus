---
title: Projekte
description: Der Projektwähler und die Projektkonfiguration über argvus Projekte.
---
`argvus projects` listet Ihre Projekte auf und öffnet sie. Es ist ein Befehl im Prozess des `argvus`-Pakets, und `SUPER + O` öffnet den Auswahlpicker.

## Befehle

```text
argvus projects                    pick a project (SUPER + O)
argvus projects open SELECTOR      open a project by position, full path or unique name
argvus projects add [--root] DIR   add a project, or a root holding projects
argvus projects remove DIR         remove a project or root
argvus projects list               print configured roots and paths
argvus projects pick [PROMPT]      print the path picked from the list, for other tools
```
## Was ist ein Projekt

Ein Projekt ist ein Verzeichnis, das in der Konfiguration (`/projects/paths`) aufgeführt ist, oder ein Unterverzeichnis eines in `/projects/roots` aufgeführten Root-Verzeichnisses. Es werden höchstens neun Projekte aufgeführt: die Nummern der `SUPER + CTRL + N`-Verknüpfungen. Das Öffnen eines Projekts fokussiert seinen Hyprland-Arbeitsbereich. Beim ersten Öffnen eines Projekts startet ARGVUS ein Terminal und den in `/projects/editor` festgelegten Editor darin.

Die Konfiguration wird über `argvus config` gelesen und geschrieben, sodass das Kontrollzentrum und die Shell dieselbe Liste sehen.

## Projektarbeitsbereiche und die Taskleiste

Projekte verwenden gewöhnliche benannte Hyprland-Arbeitsbereiche, keine speziellen Arbeitsbereiche. `SUPER + O` zeigt die Projektnummer, das Verzeichnis und die direkte `SUPER + CTRL + N`-Verknüpfung. Beide Einstiegspunkte fokussieren denselben Arbeitsbereich. Nur geöffnete Arbeitsbereiche erscheinen in der Taskleiste, mit einem `Project: name`-Label, Aktivzustand und einem Tooltip, der den vollständigen Pfad und die Verknüpfung enthält. Klicken Sie auf die Arbeitsbereichstaste, um zu ihm zurückzukehren. Die Taskleiste folgt dem Monitor des Arbeitsbereichs.

Der kanonische Verzeichnispfad bestimmt eine stabile Arbeitsbereichsidentität; das Ändern der Listenreihenfolge ändert die Verknüpfung, nicht den Arbeitsbereich. Projekte mit demselben Verzeichnisnamen bleiben getrennt, und symbolische Links auf dasselbe Verzeichnis werden dedupliziert. Terminal- und Editor-Starts richten sich explizit nach dem Arbeitsbereich des Projekts. Editor-Anwendungen, die einen bestehenden Prozess/Fenster wiederverwenden, können das eigene Arbeitsbereichsverhalten dieser Anwendung beibehalten.

Arbeitsbereiche, die vor dem Update geöffnet wurden, behalten ihre vorhandenen Fenster und nativen Namen. Ein Projekt nach dem Update zu öffnen, verwendet dessen neue, pfadbasierte Identität; schließen Sie alte Projektfenster, wenn es Ihnen passt. Es wird keine automatische Zuordnung oder Verschiebung versucht, da ein alter Name das Verzeichnis nicht zuverlässig identifiziert. Bei doppelten Verzeichnisnamen verwenden Sie den Auswähler, eine Zahl oder den vollständigen kanonischen Pfad anstelle eines mehrdeutigen Namens.

Das Öffnen eines Projekts erzeugt `$XDG_RUNTIME_DIR/argvus/project-workspaces.json` atomar aus der konfigurierten Projektliste und den Übersetzungen neu. Dies ist Sitzungs-Metadaten für Arbeitsbereichsbezeichnungen und Tooltips, nicht für die Konfiguration; bearbeiten Sie es nicht. Öffnen Sie ein Projekt nach einer Änderung der Liste oder Sprache erneut, um dessen Bezeichnungen zu aktualisieren. Aktualisieren Sie sowohl `argvus` als auch `argvus-waybar` sowie die `argvus-taskbar`-Konfiguration für diese Integration.
