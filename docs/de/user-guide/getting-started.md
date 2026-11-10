---
title: Erste Konfiguration
description: Ein praktischer Pfad für die Erstkonfiguration von ARGVUS.
---
Nach der ersten grafischen Sitzung müssen Sie nicht jeden Teil von ARGVUS konfigurieren. Beginnen Sie mit den Einstellungen, die beeinflussen, wie sich der Desktop anfühlt, und konfigurieren Sie dann Hardware und Dienste nach Bedarf.

## Eine nützliche Reihenfolge

1. Lesen Sie die [Desktop-Übersicht](/docs/user-guide/desktop/), um Fenster, Arbeitsbereiche, die Taskleiste, die Systemsteuerung und den Launcher zu identifizieren.
2. Öffnen Sie das [Kontrollzentrum](/docs/components/argvus-control-center/) und wählen Sie ein Thema, einen Modus, eine Akzentfarbe und ein Hintergrundbild unter **Darstellung**.
3. Passen Sie [Fenster und Layout](/docs/argvus-hyprland/windows-and-layout/) an, wenn die Standardabstände, -ränder oder die Taskleistenabstände nicht zu Ihrem Bildschirm passen.
4. Konfigurieren Sie die [Taskleiste](/docs/argvus-taskbar/taskbar/) und entscheiden Sie, welche Systemsteuerungskarten sichtbar sein sollen.
5. Überprüfen Sie [Tastenkombinationen](/docs/argvus-hyprland/keyboard-shortcuts/) und ändern Sie nur die Belegungen, die Sie tatsächlich nutzen.
6. Konfigurieren Sie [Maus und Touchpad](/docs/argvus-hyprland/input/), Bildschirme, Netzwerk und Energieeinstellungen über die entsprechenden Einstellungsseiten.

Die ersten vier Schritte sind optionale Personalisierungen. Die verbleibenden Schritte sind normalerweise nur erforderlich, wenn Ihre Hardware, Sprache oder Arbeitsweise dies erfordert.

## Zwei Orte zum Ändern von Einstellungen

Verwenden Sie **Systemsteuerung** für Einstellungen, die beschreiben, wie ARGVUS konfiguriert werden soll: Erscheinungsbild, Layout, Schriftarten, Eingabe, Tastenkürzel, Sprache, Region und Standardanwendungen. Verwenden Sie **Kontrollfeld** für Status- und häufige Sitzungsaktionen wie Lautstärke, Helligkeit, Netzwerkstatus, Benachrichtigungen, Energie- und Sitzungssteuerung.

Siehe [Systemsteuerung vs Kontrollfeld](/docs/components/argvus-control-center/) für die vollständige Unterscheidung und [Wo Dinge konfiguriert werden](/docs/user-guide/where-to-configure/) für ein schnelles Verzeichnis.

## Änderungen, Persistenz und Wiederherstellung

Die meisten Änderungen in der Systemsteuerung werden angewendet, während die Sitzung läuft, und als ARGVUS-Benutzerstatus gespeichert. Änderungen am Erscheinungsbild werden in der kanonischen Konfiguration festgeschrieben, und `argvus-config` projiziert die von Hyprland und der Shell verwendeten Dateien daraus. Ein Kontrollfeld-Umschalter kann eine direkte Aktion oder eine Sitzungsstatusänderung anstelle einer dauerhaften Konfiguration sein.

Wenn eine Seite **Standardeinstellungen zurücksetzen** anbietet, verwenden Sie diese Seitenaktion, anstatt generierte Dateien zu löschen. Das Erscheinungsbild hat separate Zurücksetz-Aktionen für Akzente, Hintergrundbilder und Layoutwerte; die Tastenkombinationsseite hat ihre eigene Wiederherstellungsaktion. Es gibt keine einzige universelle Zurücksetzen-Schaltfläche für alle Desktop-Einstellungen.

## Nächste Schritte

- [Erscheinungsbild](/docs/user-guide/appearance/) — Themes, Hintergrundbilder, Effekte und Layout.
- [Desktop-Layout](/docs/argvus-hyprland/windows-and-layout/) — Lücken, Ränder und Panel-Abstände verstehen.
- [Eingabegeräte](/docs/argvus-hyprland/input/) — Maus- und Touchpad-Steuerungen.
- [Fehlerbehebung bei Thema und Hintergrundbild](/docs/user-guide/troubleshooting/theme-and-wallpaper/) — Wiederherstellung, wenn eine visuelle Änderung nicht wie erwartet ist.
