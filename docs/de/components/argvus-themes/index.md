---
title: Themen
description: Installieren, auswählen, anpassen, importieren und exportieren Sie ARGVUS-Themes.
---
Ein Thema in ARGVUS legt die Farben, Oberflächen, Standard-Hintergrundbilder und Layout-Geometrie fest, die auf dem gesamten Desktop verwendet werden: Hyprland, Waybar, das Kontrollpanel, die Taskleiste, der Sperrbildschirm, Benachrichtigungen und ARGVUS-Anwendungen. Dieses Handbuch behandelt alles, was Sie als Benutzer benötigen, um mit Themen zu arbeiten.

## Was in einem Thema enthalten ist

Jedes offizielle Thema ist eine vollständige visuelle Identität mit zwei Modi:

- **Sticky** — ein kompaktes Layout mit kleinen Fensterabständen, keinen äußeren Rändern und einer Taskleiste nahe am Bildschirmrand.
- **Spaced** — ein großzügiges Layout mit größeren Abständen, abgerundeten Ecken und Rändern um die Taskleiste und Shell-Oberflächen.

Die Auswahl eines Themas ändert nicht seine Identität: die Palette, die Standard-Hervorhebungsfarbe und die Hintergrundbildzuordnung bleiben zwischen den Modi gleich. Der Modus ist eine separate Einstellung, siehe [Offizielle Themen](/docs/components/argvus-themes/official-themes/)].

## Seiten in diesem Handbuch

- [Themen installieren](/docs/components/argvus-themes/installing-themes/) — wo die offiziellen Themen herkommen und wie man eines oder alle mit `pacman` installiert.
- [Offizielle Themen](/docs/components/argvus-themes/official-themes/) — der Katalog, die standardmäßige Hervorhebungsfarbe jedes Themas und wie man ein Thema und dessen Modus auswählt.
- [Hervorhebungsfarbe anpassen](/docs/components/argvus-themes/highlight-color/) — die Akzentfarbe unabhängig vom Thema ändern.
- [Themen importieren und exportieren](/docs/components/argvus-themes/import-and-export/) — das aktive Thema mit deiner Hervorhebungsfarbe und deinem Hintergrundbild als eigenes Thema exportieren und auf einem anderen Gerät importieren.

## Schnelle Referenz

| Aufgabe | Wo |
| --- | --- |
| Ein Thema auswählen | **Kontrollzentrum → Erscheinungsbild → Themen → Offizielle Themen** |
| Sticky und Spaced wechseln | **Kontrollzentrum → Erscheinungsbild → Modus** |
| Die Hervorhebungsfarbe ändern | **Kontrollzentrum → Erscheinungsbild → Hervorhebungsfarbe** |
| Das aktive Thema exportieren | **Kontrollzentrum → Erscheinungsbild → Themen → Thema exportieren** |
| Ein Thema importieren | **Kontrollzentrum → Erscheinungsbild → Themen → Thema importieren** |
| Den Themenauswähler über die Tastatur öffnen | `SUPER + SHIFT + T` |

## Woher die Themen stammen

ARGVUS liefert zwei eingebaute Themen, **ARGVUS Dark** und **ARGVUS Light**, mit dem `argvus-appearance`-Paket. Jedes andere offizielle Thema ist ein separates `argvus-theme-<name>`-Paket. Die Installation eines Themenpakets fügt es dem Themenauswähler hinzu; ein Neustart der Sitzung ist nicht erforderlich. Eigene Themen, exportiert oder importiert, werden separat unter **Eigene Themen** aufgelistet.

## Tastaturhinweise

Die Themen-Seiten des Kontrollzentrums zeigen ihre Tasten in der Fußzeile an:

- `↑` / `↓` — navigieren.
- `Enter` — anwenden oder öffnen.
- `e` — das aktive Thema exportieren, auf einer Themen-Seite.
- `i` — ein Thema importieren, auf einer Themen-Seite.
- `d` — ein benutzerdefiniertes Thema löschen.
- `Esc` — zurückgehen.

## Fehlerbehebung

- **Ein installiertes Thema wird nicht angezeigt.** Bestätigen Sie, dass das Paket mit `pacman -Q argvus-theme-<name>` installiert ist, und öffnen Sie dann das Kontrollzentrum erneut. Das Paket muss mit `argvus-appearance` vorhanden installiert sein.
- **Die Hervorhebungsfarbe hat sich nach dem Themenwechsel geändert.** Eine benutzerdefinierte Hervorhebungsfarbe überdauert Themenwechsel. Verwenden Sie **Auf Standardfarbe des Themas zurücksetzen**, um dem Thema seine eigene Farbe zurückzugeben.
- **Ein Themenarchiv wird nicht für den Import angezeigt.** Es muss direkt in Ihrem Heimatverzeichnis liegen und `<id>/theme.toml` enthalten. Siehe [Themen importieren und exportieren](/docs/components/argvus-themes/import-and-export/).
