---
title: Häufig gestellte Fragen
description: Häufige Fragen zu ARGVUS, seiner Architektur und der täglichen Nutzung.
---
Diese Seite beantwortet häufige Fragen darüber, was ARGVUS ist, wie es verteilt wird und wo es konfiguriert werden kann. Für detaillierte Verfahren folgen Sie den Links in jeder Antwort.

## Was ist ARGVUS?

ARGVUS ist eine modulare Desktop-Umgebung für Wayland und Arch Linux, aufgebaut um Hyprland. Es ist keine von Hyprland unabhängige Desktop-Umgebung; es ist eine Desktop-Erfahrung, die auf Hyprland aufgebaut und auf Arch Linux zugeschnitten ist. ARGVUS koordiniert die grafische Sitzung, Shell-Oberflächen, Einstellungsanwendungen, Erscheinungsbild, Desktop-Dienste und integrierte Tools durch eine Sammlung von Paketen.

Es ist mehr als eine Hyprland-Konfiguration oder eine Sammlung persönlicher Dotfiles: Es bietet einen unterstützten Sitzungseinstiegspunkt, gemeinsame Konfiguration und Erscheinungsbildstatus sowie definierte Zuständigkeiten für Komponenten wie die Taskleiste, das Kontrollzentrum, Benachrichtigungen, Netzwerke und Bildschirme. Siehe die [Einführung](/docs/introduction/)] für einen Überblick darüber, wie die Teile zusammenpassen.

## Ist ARGVUS kostenlos?

Ja. ARGVUS kann kostenlos heruntergeladen und genutzt werden, und seine Kernprojektpakete werden als freie Software unter der [GNU General Public License Version 3](https://www.gnu.org/licenses/gpl-3.0.html)] verteilt. Dies gibt den Nutzern die Freiheit, die betreffende Software gemäß den Lizenzbedingungen auszuführen, zu studieren, zu modifizieren und weiterzuverbreiten.

ARGVUS wird als signierte Arch-Linux-Pakete verteilt. Die Software selbst hat keinen Kaufpreis, das Herunterladen von Paketen kann jedoch normale Internet- oder Hostingkosten verursachen. Von Drittanbietern bereitgestellte Komponenten und Anwendungen können eigene Lizenzen haben.

## Ist ARGVUS eine Linux-Distribution?

Nein. ARGVUS ist die Desktop-Umgebungsebene, die auf Arch Linux installiert wird. Arch Linux stellt das Betriebssystem, den Kernel, den Paketmanager und Basispakete bereit; Hyprland stellt die Wayland-Komposition bereit; ARGVUS koordiniert die Desktop-Sitzung und ihre integrierten Komponenten.

Siehe [installation](/docs/getting-started/installation/)] für das unterstützte Installationsmodell und [What ARGVUS is—and is not](/docs/introduction/#what-argvus-isand-is-not)] für die architektonische Unterscheidung.

## Warum eine Desktop-Umgebung statt einer Distribution?

ARGVUS als Desktop-Umgebung zu verwenden, gibt den Benutzern mehr Kontrolle über den Rest ihres Systems. Eine Distribution wählt normalerweise eine vollständige Basis aus und pflegt diese, einschließlich Kernel, Paketwahl, Repositories, Voreinstellungen und Systemrichtlinien. ARGVUS konzentriert sich auf das Desktop-Erlebnis, während diese breiteren Entscheidungen dem Benutzer und der zugrunde liegenden Arch-Linux-Installation überlassen werden.

Dieses Modell hält das Projekt auch modular: Sitzungsverwaltung, Hyprland-Konfiguration, Erscheinungsbild, Taskleiste, Einstellungen und Anbieter können separat entwickelt und verpackt werden, während sie zusammen als ein Desktop funktionieren.

## Welche Systeme werden unterstützt?

Die derzeit unterstützte Basis ist Arch Linux. ARGVUS wird für Arch Linux verpackt und hängt von den Wayland-, Hyprland- und Systemkomponenten ab, die in den [Installationsanforderungen](/docs/getting-started/installation/requirements/) beschrieben sind.

Andere Distributionen können einige der gleichen Upstream-Komponenten bereitstellen, aber das macht das komplette ARGVUS-Paketset oder die Sitzungsintegration dort nicht unterstützt.

## Wie installiere ich ARGVUS?

Folgen Sie der [Installationsanleitung](/docs/getting-started/installation/)]. Sie erklärt die Anforderungen, wie man das signierte ARGVUS-Paket-Repository konfiguriert, wie man das Paketset installiert und wie man die erste Sitzung startet.

ARGVUS wird als koordiniertes Set von Arch-Paketen geliefert. Das `argvus`-Paket ist das Hauptinstallationsziel und zieht die Desktop-Komponenten, die es koordiniert, mit ein.

## Wie ändere ich die ARGVUS-Einstellungen?

Verwenden Sie **Control Center** für eine dauerhafte Desktop-Konfiguration, einschließlich Themen, Hintergrundbilder, Effekte, Layout, Schriftarten, Tastatur, Eingabe, Sprache, Region und Standardanwendungen. Die effizientesten Möglichkeiten, es zu öffnen, sind die `SUPER + Alt + C`-Verknüpfung oder der Launcher mit `SUPER + D`: Geben Sie **ARGVUS Control Center** ein. Sie können auch `argvus` eingeben, um die ARGVUS-Anwendungen aufzulisten.

Als zusätzliche Option können Sie es auch über ein Terminal öffnen mit:

```sh
argvus --control-center
```
Sie können auch einen fokussierten Bereich mit `argvus control-center` öffnen, zum Beispiel `argvus control-center appearance themes`. Führen Sie `argvus control-center --help` aus, um die in der installierten Version verfügbaren Routen zu sehen.

Verwenden Sie **Systemsteuerung** für den aktuellen Status und häufige Aktionen wie Lautstärke, Helligkeit, Netzwerk, Benachrichtigungen, Stromversorgung und Sitzungssteuerung. Es ist keine zweite Konfigurationsdatenbank. Siehe [Wo Dinge zu konfigurieren sind](/docs/user-guide/where-to-configure/)] und [Kontrollzentrum](/docs/components/argvus-control-center/)] für den Unterschied.

## Wo werden ARGVUS-Einstellungen gespeichert?

Benutzergeführte Konfiguration wird unter `$XDG_CONFIG_HOME/argvus` (normalerweise `~/.config/argvus`) gespeichert. Die kanonische logische Konfiguration wird dort gepflegt, während die abgeleiteten Verbraucherdateien für Hyprland, Waybar und das Telemetrie-Widget, GTK, Qt6ct, Quickshell, Dunst, Hyprlock, Yazi, Superfile, Terminals, Schriftarten, Effekte, Kalender, Eingabe, Tastatur, Energie und Wechselgeräte unter `~/.config/argvus/data/generated/` liegen. Dieser Baum wird nur von `argvus-config` beschrieben und aus der kanonischen Konfiguration neu erstellt; `argvus-appearance` übernimmt Designänderungen und stimmt dann nur die externen Verbraucher ab, die die kanonische Konfiguration nicht selbst lesen können. Rofi ist die Ausnahme: `argvus launcher` verpackt seine Konfiguration und Themen, und die vom Benutzer bearbeiteten Kopien liegen unter `~/.config/argvus/data/rofi/`.

Bearbeiten Sie generierte Dateien nicht als dauerhafte Konfigurationsmethode. Verwenden Sie das Control Center oder den dokumentierten nativen Überschreibungspfad für die Komponente, die Sie anpassen möchten. Siehe [Konfigurationsdateien](/docs/reference/configuration-files/)] und [Konfiguration und Zustand](/docs/developer-guide/architecture/configuration-and-state/)].

## Was soll ich tun, wenn eine Änderung nicht angewendet wird?

Bestätigen Sie zuerst, dass die Änderung auf der entsprechenden Seite gespeichert oder angewendet wurde. Prüfen Sie dann, ob die betroffene Komponente eine Sitzungs- oder Service-Neuladung benötigt. Eine übermittelte Änderung wird durch `argvus-config` in die Verbraucherdateien projiziert, aber die Projektion garantiert nicht, dass jeder laufende Verbraucher sie neu geladen hat. `argvus sessionctl reload` wendet eine bereits übermittelte Generation aus dem Projektionsmanifest an, anstatt erneut zu projizieren, und überspringt das grafische Neuladen, wenn sich der kanonische Zustand nicht geändert hat.

Verwenden Sie den entsprechenden [Fehlerbehebungsleitfaden](/docs/user-guide/troubleshooting/) für Installation, Sitzungstart, Grafik, Displays, Hyprland, Themes und Hintergründe. Beim Melden eines Problems geben Sie die Komponente, die geänderte Einstellung, den verwendeten Befehl oder die Seite sowie die Ausgabe des relevanten Status- oder Diagnosebefehls an.

## Wo kann ich mehr erfahren?

- [Erste Konfiguration](/docs/user-guide/getting-started/) für den empfohlenen Ablauf der ersten Sitzung.
- [Benutzerhandbuch](/docs/user-guide/) für funktionsspezifische Anweisungen.
- [Referenz](/docs/reference/) für Befehle, Pfade, Dienste und Konfigurationsverträge.
- [Entwicklerhandbuch](/docs/developer-guide/) für Architektur, Paketverantwortung und Beitrag-Workflows.
