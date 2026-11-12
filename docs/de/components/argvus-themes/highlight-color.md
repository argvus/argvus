---
title: Anpassen der Hervorhebungsfarbe
description: Ändern Sie die Hervorhebungsfarbe von ARGVUS unabhängig vom ausgewählten Thema und kehren Sie zum Standard des Themas zurück.
---
Die **Hervorhebungsfarbe** ist der Akzent, der für Fokusrahmen, Auswahl und aktive Zustände auf dem Desktop verwendet wird. Jedes Thema hat eine Standard-Hervorhebungsfarbe, aber Sie können sie durch jede beliebige Farbe ersetzen, ohne das Thema zu ändern.

## Hervorhebungsfarbe ändern

1. Öffnen Sie **Systemsteuerung → Erscheinungsbild → Hervorhebungsfarbe**.
2. Wählen Sie **Hervorhebungsfarbe bearbeiten**. Der aktuelle Wert wird neben der Option angezeigt.
3. Geben Sie eine sechsstellige RGB-Farbe ein, mit oder ohne führendes `#`. Zum Beispiel werden `3590BD` und `#3590BD` beide akzeptiert.
4. Drücken Sie `Enter`, um anzuwenden.

Sie können auch eine Farbe in das Feld einfügen. ARGVUS akzeptiert den eingefügten Text nur, wenn es sich um einen gültigen sechsstelligen RGB-Wert handelt. Kurzformen wie `#123` und Werte mit nicht-hexadezimalen Zeichen werden mit einer Fehlermeldung abgelehnt, und die Farbe wird nicht geändert.

Die neue Farbe wird sofort auf den Desktop angewendet und in Ihrer ARGVUS-Konfiguration gespeichert.

## Zurück zur Standardfarbe des Themas

Um zur Farbe zurückzukehren, die vom ausgewählten Thema definiert ist, öffnen Sie **Systemsteuerung → Darstellung → Hervorhebungsfarbe** und wählen Sie **Auf Standardfarbe des Themas zurücksetzen** aus.

## Wie die Hervorhebungsfarbe mit Themen interagiert

- Eine von Ihnen gewählte Hervorhebungsfarbe ist eine unabhängige Überschreibung. Sie bleibt bestehen, wenn Sie zu einem anderen Thema wechseln, sodass die von Ihnen gewählte Farbe gleich bleibt.
- Ein Thema, das Sie nach **Auf Standardfarbe des Themas zurücksetzen** anwenden, übernimmt die Hervorhebungsfarbe wieder und verwendet seine eigene Standardfarbe.
- Das Exportieren des aktiven Themas speichert die Hervorhebungsfarbe, die beim Export als Akzent der Palette des neuen Themas aktiv ist. Siehe [Importieren und Exportieren von Themen](/docs/components/argvus-themes/import-and-export/).

## Wo der Wert gespeichert ist

Die Hervorhebungsfarbe ist Teil Ihrer ARGVUS-Konfiguration, in `$XDG_CONFIG_HOME/argvus/config.json` (normalerweise `~/.config/argvus/config.json`). Bearbeiten Sie die generierten Dateien unter `$XDG_CONFIG_HOME/argvus/data/generated` nicht: ARGVUS überschreibt sie aus der Konfiguration.
