---
title: Offizielle Themen
description: Der offizielle ARGVUS-Themenkatalog, die standardmäßigen Hervorhebungsfarben und wie man ein Thema sowie dessen Sticky- oder Spaced-Modus auswählt.
---
Diese Seite listet die offiziellen Designs auf, die als Designpakete erhältlich sind, und erklärt, wie man eines auswählt. Zum Installieren siehe [Installing themes](/docs/components/argvus-themes/installing-themes/).

## Ein Design auswählen

1. Öffne **Systemsteuerung → Erscheinungsbild → Designs**.
2. Wähle **Offizielle Designs**.
3. Wähle **Dunkel** oder **Hell**.
4. Wähle die Designfamilie. Das Design wird sofort angewendet, sobald du es auswählst.

Das ausgewählte Design wird gleichzeitig auf den gesamten Desktop angewendet: die laufende Taskleiste, Panels, Benachrichtigungen, Fenster und der Hintergrund ändern sofort die Farbe, ohne Ladebildschirm und ohne Neustart. ARGVUS behält deine Hintergrundauswahl, außer das Design wird mit seiner eigenen Hintergrundzuordnung angewendet.

Du kannst auch `SUPER + SHIFT + T` drücken, um den Design-Selector über die Tastatur zu öffnen. Verwende die Pfeiltasten, um durch die Kategorien zu navigieren, `Enter` zum Auswählen und `Esc` zum Schließen.

## Haftend und Abstand

Jedes Thema funktioniert in zwei Modi. Der Modus ist vom Thema unabhängig: Ändert man das Thema, bleibt der aktuelle Modus erhalten, und ändert man den Modus, bleibt das aktuelle Thema erhalten.

| Modus | Fensterabstände | Außenabstand | Fensterabrundung | Taskleiste und Shell-Abstand |
| --- | ---: | ---: | ---: | ---: |
| Sticky | `2` | `0` | `0` | nahe am Rand |
| Spaced | `10` | `18` | `4` | `18` |

Um den Modus zu wechseln, öffnen Sie **Systemsteuerung → Erscheinungsbild → Modus** und wählen **Sticky** oder **Spaced**. Ihre manuellen Anpassungen dieser Layoutwerte bleiben bis zur nächsten expliziten Themenauswahl erhalten.

## Dunkle Themen

| Thema | Paket | Standard-Hervorhebung |
| --- | --- | --- |
| One Dark | `argvus-theme-one-dark` | `#61AFEF` |
| Dracula | `argvus-theme-dracula` | `#BD93F9` |
| Silver Dark | `argvus-theme-silver-dark` | `#595959` |
| Slate Dark | `argvus-theme-slate-dark` | `#7391A5` |
| Universe | `argvus-theme-universe` | `#EEEEEE` |
| Gruvbox High Dark | `argvus-theme-gruvbox-high-dark` | `#D79921` |
| Gruvbox Dark | `argvus-theme-gruvbox-dark` | `#D4BE98` |
| Rosé Pine | `argvus-theme-rose-pine` | `#C4A7E7` |
| Tokyo Night | `argvus-theme-tokyo-night` | `#7AA2F7` |
| Solitude | `argvus-theme-solitude` | `#798186` |
| Sunset | `argvus-theme-sunset` | `#E2BE8A` |
| Matrix | `argvus-theme-matrix` | `#82FB9C` |
| Monokai Dark | `argvus-theme-monokai-dark` | `#78DCE8` |
| Lady Mist | `argvus-theme-lady-mist` | `#354B78` |

## Helle Themen

| Thema | Paket | Standardhervorhebung |
| --- | --- | --- |
| GitHub Light | `argvus-theme-github-light` | `#0969DA` |
| Solarized Light | `argvus-theme-solarized-light` | `#268BD2` |
| Nord Light | `argvus-theme-nord-light` | `#5E81AC` |
| One Light | `argvus-theme-one-light` | `#4078F2` |
| Everforest Light | `argvus-theme-everforest-light` | `#3A94C5` |
| Catppuccin Latte | `argvus-theme-catppuccin-latte` | `#1E66F5` |
| Gruvbox Light | `argvus-theme-gruvbox-light` | `#458588` |
| Watercolor | `argvus-theme-watercolor` | `#1A639C` |
| Obsidian Light | `argvus-theme-obsidian-light` | `#334A78` |

## Eingebaute Themen

**ARGVUS Dark** und **ARGVUS Light** werden von `argvus-appearance` bereitgestellt und sind immer unter **Offizielle Themen** verfügbar. ARGVUS Dark verwendet die Standardhervorhebung `#3590BD`; ARGVUS Light verwendet `#181818`.

## Wofür ein Thema zuständig ist

Ein Thema definiert die Palette und Oberflächenbehandlung, die von den Desktop-Komponenten verwendet werden, die Standard-Hervorhebungsfarbe, die Standard-Wallpaper-Zuordnung sowie die Standardwerte für Transparenz und Unschärfe. Die Desktop-Komponenten lesen diesen Zustand, anstatt eine eigene Kopie zu speichern, sodass eine Themenänderung den Kompositor, die Taskleiste, die Systemsteuerung, den Sperrbildschirm, Benachrichtigungen und Anwendungen gleichzeitig erreicht.

Die Standard-Hervorhebungsfarbe ist nur der Ausgangspunkt. Sie können sie durch jede sechsstellige Farbe ersetzen, siehe [Anpassen der Hervorhebungsfarbe](/docs/components/argvus-themes/highlight-color/).
