---
title: Erscheinung
description: Konfigurieren Sie die visuelle Sprache, die von ARGVUS-Komponenten geteilt wird.
---
Das Erscheinungssystem wendet einen Themenzustand auf Hyprland, GTK, Qt, Waybar, Quickshell, Rofi, Dunst, Hyprlock und ARGVUS-Anwendungen an.

## Themenfamilien

Die unten aufgeführten dreiundzwanzig Familien werden durch die aktuelle Themenlast definiert. Die Farbspalte ist die Standardfarbe für den aktiven Rand/die Akzentfarbe der Familie; Benutzer können eine andere gültige RGB-Akzentfarbe wählen.

| Familie | Standard-Hervorhebungsfarbe zurücksetzen | Basis-Hintergrund | Modi |
| --- | --- | --- | --- |
| One Dark | `#61AFEF` | `#282C34` | Sticky, Spaced |
| Dracula | `#BD93F9` | `#282A36` | Sticky, Spaced |
| ARGVUS Dark | `#3590bd` | `#111316` | Sticky, Spaced |
| Dark Silver | `#595959` | `#111316` | Sticky, Spaced |
| Dark Slate | `#7391a5` | `#2f3541` | Sticky, Spaced |
| Dark Universe | `#eeeeee` | `#000000` | Sticky, Spaced |
| ARGVUS Dark Gruvbox High | `#D79921` | `#282828` | Sticky, Spaced |
| ARGVUS Dark Gruvbox | `#D4BE98` | `#282828` | Sticky, Spaced |
| ARGVUS Licht | `#181818` | `#f7f7f7` | Klebrig, beabstandet |
| GitHub Light | `#0969DA` | `#FFFFFF` | Klebrig, beabstandet |
| Catppuccin Latte | `#1E66F5` | `#EFF1F5` | Klebrig, beabstandet |
| ARGVUS Light Gruvbox | `#458588` | `#FBF1C7` | Klebrig, beabstandet |
| Rosé-Kiefer | `#C4A7E7` | `#191724` | Klebrig, beabstandet |
| Tokio-Nacht | `#7AA2F7` | `#1A1B26` | Klebrig, beabstandet |
| Einsamkeit | `#798186` | `#101315` | Klebrig, beabstandet |
| Sonnenuntergang | `#E2BE8A` | `#0F0F0F` | Klebrig, beabstandet |
| Matrix | `#82FB9C` | `#0B0C16` | Klebrig, beabstandet |
| Monokai Dark | `#78DCE8` | `#2D2A2E` | Klebrig, beabstandet |
| Lady Mist | `#354B78` | `#131313` | Klebrig, beabstandet |
| Ein Licht | `#4078F2` | `#FAFAFA` | Klebrig, beabstandet |
| Everforest Licht | `#3A94C5` | `#FDF6E3` | Klebrig, beabstandet |
| Solarisiertes Licht | `#268BD2` | `#FDF6E3` | Klebrig, beabstandet |
| Nordlicht | `#5E81AC` | `#ECEFF4` | Klebrig, beabstandet |
| Aquarell | `#1A639C` | `#E9E3D3` | Klebrig, beabstandet |

Die im Tisch dargestellte Farbe ist der Wert, der in `config.json` geschrieben wird, wenn diese Familie ausgewählt wird oder wenn **Auf Standardthema zurücksetzen** verwendet wird. Ein manuell gewählter sechsstelliger RGB-Akzent gilt bis zum nächsten Themenwechsel, der ihn durch das Standardthema des gewählten Designs ersetzt.

**Sticky** ist das kompakte Layout: Kachel-Fenster verwenden kleine interne Abstände, die äußeren Abstände sind null und die Taskleiste sitzt nah am Bildschirmrand. **Spaced** fügt größere Abstände, abgerundete Ecken, Schatten und Ränder um die Taskleiste und die Shell-Oberflächen hinzu.

Vorhandene `float`-Layout-Werte und `-float`-Themen-IDs werden als alte Bezeichnungen für Spaced erkannt. Neue Layout-Werte und aufgelöste Themen-IDs verwenden `spaced` und `-spaced`.

Der Modus ändert die Geometrie und Oberflächenbehandlung; er erstellt keine sechste Farbenfamilie. Siehe [Themes](/docs/user-guide/appearance/)] für die Auswahl und [Desktop](/docs/user-guide/desktop/)] für Taskleisten- und Fensterplatzierung.

- [Themen](/docs/user-guide/appearance/)
- [Hintergründe](/docs/argvus-appearance/wallpapers/)
- [Schriften und Symbole](/docs/argvus-appearance/fonts-and-icons/)
- [Effekte](/docs/argvus-appearance/effects/)

Das Kontrollzentrum zeigt die benutzerseitigen Einstellungen an. Die Implementierung wird von `argvus-appearance`, `argvus-session`, den Shell-Komponenten und den Asset-Paketen geteilt. Siehe [Darstellungsarchitektur](/docs/developer-guide/subsystems/appearance-and-themes/).

## Fensterwackeln

Öffnen Sie **Desktop & Eingabe → Hyprland → Animationen → Fenster** und verwenden Sie die
**Wackeln**-Steuerelemente. Suchen Sie nach `wobble`, oder öffnen Sie die Seite direkt:

```sh
argvus control-center hyprland animations windows
```
Wackeln ist standardmäßig deaktiviert. Seine ARGVUS-Standardeinstellungen sind Steifigkeit `100`, Dämpfung `10`, Masse `1`, Intensität `1` und Mesh-Auflösung `8`. Steifigkeit und Masse akzeptieren `0.0001–1000`, Dämpfung `0–1000`, Intensität `0–2`, und Mesh eine Ganzzahl `2–32`. Eine höhere Mesh-Auflösung erhöht die Rendering-Kosten. Verwenden Sie die Pfeiltasten, um Werte anzupassen, oder Enter, um sie einzugeben. **Wackel-Standardeinstellungen wiederherstellen** ändert den Entwurf; **Anwenden** speichert alle ausstehenden Animationsbearbeitungen und lädt Hyprland einmal neu, ohne die Sitzung neu zu starten. Zurück innerhalb der Animationen zu gehen, behält den gemeinsamen Entwurf bei.

Der globale Animationsschalter unterdrückt das Wackeln, während die individuellen Einstellungen beibehalten werden. Die kanonischen Werte befinden sich unter `effects.wobble_*` im ARGVUS-Zustand; `hypr/state.lua` wird generiert und darf nicht manuell bearbeitet werden.

Dies erfordert Hyprland **0.57.0 oder neuer**, vorgeschrieben durch die Paketabhängigkeit von `argvus-hyprland`. Version 0.56 wird für diese Konfiguration nicht unterstützt. Das Intensitätsmaximum folgt dem [veröffentlichten 0.57.0 Quellcode ](https://github.com/hyprwm/Hyprland/blob/v0.57.0/src/config/values/ConfigValues.cpp), der es auf `2` begrenzt, auch wenn neuere Dokumentationen möglicherweise `3` aufführen.
