---
title: Einführung
description: Was ARGVUS ist, wie sein Desktop funktioniert und wo seine Komponenten passen.
---
ARGVUS ist eine modulare Hyprland- und Wayland-Desktopumgebung für Arch Linux. Es handelt sich um eine integrierte Sammlung von Paketen und nicht um eine monolithische Anwendung: die Sitzung, der Compositor, die Shell-Oberflächen, Einstellungsanwendungen, Anbieter und visuelle Assets arbeiten durch gemeinsamen Zustand und explizite Laufzeitverträge zusammen.

Diese Seite ist die Karte der Dokumentation. Sie erklärt den gesamten Desktop auf hohem Niveau; die verlinkten Seiten enthalten die betrieblichen und Implementierungsdetails.

## Wie ARGVUS funktioniert

Die normale Sitzung folgt diesem Pfad:

```text
graphical login or TTY
        ↓
argvus-session → argvus start
        ↓
Hyprland + argvus-hyprland
        ↓
argvus-session.target (systemd --user)
        ↓
taskbar · control panel · notifications · wallpaper · idle · clipboard
```
`argvus-session` ist für den Lebenszyklus von Sitzungen und die Einrichtung der Umgebung verantwortlich. Es implementiert nicht jede Desktop-Funktion: Jede Komponente behält die Kontrolle über ihre eigenen Befehle, Konfigurationen und Service-Inhalte.

Für Installation und Start siehe [Installation](/docs/getting-started/installation/) und [Sessions](/docs/user-guide/sessions/). Für Service-Verantwortung siehe den Entwickler [runtime lifecycle](/docs/developer-guide/architecture/runtime-lifecycle/).

## Hauptschichten

### Einstiegspunkte und Infrastruktur

Das `argvus`-Paket stellt den `/usr/bin/argvus`-Dispatcher und den Hauptzugangspunkt des Pakets bereit. `argvus-session` stellt grafische und TTY-Einstiegspunkte, das Benutzerziel und `argvus sessionctl` bereit. Gemeinsame TUI- und Lokalisierungsfunktionen stammen vom `argvus`-Arbeitsbereich (die `argvus-tui`- und `argvus-i18n`-Crates) und dem `argvus i18n`-Befehl.

### Kompositor und Desktop-Shell

`argvus-hyprland` liefert die Hyprland-Konfiguration und Tastenkombinationen. `argvus-waybar` liefert das gepatchte Waybar-Binärprogramm, während `argvus-taskbar` die Taskleiste-Konfiguration und Aktionen besitzt. `argvus-control-panel` liefert das Quickshell-Panel, und `argvus widget-telemetry` liefert optionale System-Widgets, im Prozess in `argvus`.

Siehe [Desktop](/docs/user-guide/desktop/) für die Nutzung und [shell surfaces](/docs/developer-guide/subsystems/shell-surfaces/) für die Implementierungsverantwortung.

### Einstellungen und Anwendungen

`argvus control-center` ist die keyboard-first Einstellungsanwendung, die als Helfer in `argvus` geliefert wird. Die Kalender- und Wechseldatenträger-Werkzeuge sind GTK 4-Helfer in `argvus`; der Systemmonitor und das Terminal sind Unterbefehle von `argvus`; der Launcher ist `argvus-launcher`. Alle sind mit der Shell und dem Dispatcher integriert.

Siehe [Applications](/docs/user-guide/applications/) und das [Befehlsreferenz](/docs/reference/command-line/).

### Anbieter und Integrationen

Anzeige, Netzwerk, Bluetooth, Stromversorgung, Benachrichtigungen, Firewall und Wechselspeicher werden von spezialisierten Projekten bereitgestellt. Das Kontrollzentrum und die Systemsteuerung nutzen diese Anbieter; sie ersetzen sie nicht.

Login-, Sperr- und Startbildschirme sind ebenfalls separat: `argvus-greeter` verwaltet greetd, `argvus-lock` verwaltet Hyprlock, der in `argvus` bereitgestellte Handoff-Helfer zeichnet den Login-Spinner, das Ladeoverlay verwaltet Themenübergänge, und `argvus-boot-splash` verwaltet den Boot-Splash.

Siehe [Hardware](/docs/user-guide/hardware/), [Datenschutz und Sicherheit](/docs/user-guide/privacy-and-security/) und das Entwickler-[Komponentenreferenz](/docs/developer-guide/).

## Gemeinsamer Erscheinungszustand

Thema, Akzent, Hintergrundbild, Schriftart und Effekteinstellungen werden als logischer Zustand unter `$XDG_CONFIG_HOME/argvus` gespeichert (normalerweise `~/.config/argvus`). `argvus-config` ist die einzige Komponente, die `data/generated/` schreibt, wo es Verbraucherdateien für Hyprland, GTK, Qt6ct, Waybar, Quickshell, Dunst, Benachrichtigungen, den Sperrbildschirm, Yazi, Superfile und Terminals projiziert. Rofi ist die Ausnahme: `argvus launcher` paketiert seine Konfiguration und Themen als einfache Dateien, und die vom Benutzer bearbeiteten Kopien liegen unter `data/rofi/`. Erscheinungshilfen bestätigen die Zustandsänderung und stimmen dann externe Anwendungen ab; sie schreiben nie den erzeugten Baum.

Das erzeugte Verzeichnis ist nicht die Quelle der Wahrheit. Siehe [Appearance](/docs/user-guide/appearance/) und [Konfiguration und Zustand](/docs/developer-guide/architecture/configuration-and-state/).

Das visuelle System steuert auch das Layout. Der Sticky-Modus ist kompakt und quadratisch; der Spaced-Modus verwendet größere Fensterabstände, abgerundete Ecken und breitere Ränder um die Taskleiste und die Shell. Die Taskleiste ist standardmäßig an der oberen Kante platziert, unterstützt aber die Platzierung oben/unten über den gemeinsamen Layout-Zustand. Effekte steuern Transparenz, Unschärfe, Schatten und Animationen über den Kompositor und die Desktop-Oberflächen hinweg.

Siehe die [Thementabelle](/docs/user-guide/appearance/) für die aktuellen Familien und Farben, [Effekte](/docs/argvus-appearance/effects/) für den gemeinsamen Effektvertrag und [Taskleiste](/docs/argvus-taskbar/taskbar/) für die Platzierung von Leiste und Fenstern.

## Was ARGVUS ist – und was nicht

ARGVUS ist die Desktop-Umgebungsschicht über Arch Linux. Arch Linux liefert das Betriebssystem und die Pakete; Hyprland liefert die Wayland-Komposition; ARGVUS koordiniert die grafische Sitzung, Shell-Oberflächen, Einstellungen, Erscheinungsbild, Desktop-Dienste und integrierte Anwendungen.

Es ist daher mehr als eine Hyprland-Konfiguration oder eine Sammlung persönlicher Dotfiles. ARGVUS definiert einen unterstützten Einstiegspunkt für Sessions, verwaltet den Lebenszyklus seiner Desktop-Dienste, bietet ein gemeinsames Modell für Einstellungen und Erscheinungsbild und verbindet fokussierte Komponenten wie Netzwerk, Displays, Stromversorgung, Benachrichtigungen, Konten und Wechselmedien.

ARGVUS muss nicht jede Anwendung ersetzen. Es integriert ausgewählte Upstream-Anwendungen, wo dies die zuverlässigste Wahl ist, während das verhalten und die Konfiguration des gesamten Desktops kohärent bleiben.

Der Name vereint zwei Ideen. *ARGV*, von *Argumentvektor*, ist die Liste der Argumente, die ein Programm erhält, und spiegelt das Ziel wider, den Desktop über Befehle und Tastenkombinationen zu steuern. *US*, von *User System*, spiegelt das Ziel eines auf den Benutzer ausgerichteten Desktops wider. Für die vollständige Erklärung und die Aussprache-Audio siehe [Über ARGVUS](/about/).

Für die Implementierungsgrenzen und Paketverantwortlichkeiten siehe die Entwickler [Architekturübersicht](/docs/developer-guide/architecture/overview/) und [Repository-Rollen](/docs/developer-guide/architecture/repository-roles/).

## Weiter nach Ziel

- [Lesen Sie die FAQ](/docs/user-guide/faq/).
- [Installieren Sie ARGVUS](/docs/getting-started/installation/).
- [Starten Sie eine grafische oder TTY-Sitzung](/docs/user-guide/sessions/).
- [Konfigurieren Sie Themen und Hintergrundbilder](/docs/user-guide/appearance/).
- [Verwenden Sie die Taskleiste und die Systemsteuerung](/docs/user-guide/desktop/).
- [Verstehen Sie Befehle, Pfade und Dienste](/docs/reference/).
- [Studieren Sie die Architektur](/docs/developer-guide/architecture/overview/).
