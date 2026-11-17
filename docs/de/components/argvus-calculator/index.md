---
title: ARGVUS Rechner
description: ARGVUS Rechner — GTK4 / ARGVUS
---

# ARGVUS Rechner

`argvus calculator` · launcher: **ARGVUS Rechner**

- GTK4 / Relm4; ARGVUS theme (`time`), IBM Plex Mono.
- `+`, `-`, `*`, `/`, `%`, `±`; `50% = 0.5`.
- **Wissenschaftlich**: `sqrt`, `log`, `ln`, `exp`, `sin`, `cos`, `tan`, `asin`, `acos`, `atan`, `pi`, `e`, `^`, `(`, `)`.
- **Bogenmaß**: Nicht markiert: Grad.
- `-2^2 = -4`; `2^3^2 = 512`; `2*pi`; `1.5 = 1,5`.
- Enter: `=`; Escape: **Löschen**; Backspace: **Zeichen löschen**; Tab / Shift+Tab.
- **Ergebnis kopieren**; **Verlauf**: 100; **Verlauf löschen**.
- IEEE 754 / f64; 15 digits; 4096 bytes; 64 levels.

## Verlauf

Der Verlauf bleibt nach dem Schließen erhalten (bis zu 100 Ergebnisse). Löschen entfernt auch gespeicherte Einträge. Einstellungen: ↑/↓ zum Navigieren, Enter oder Klick zum Aktivieren, Escape zum Zurückkehren.

`$XDG_DATA_HOME/argvus/calculator/history.json` (`~/.local/share/argvus/calculator/history.json`)

Öffnet sich immer als kleines, zentriertes schwebendes Fenster (370 × 420); der wissenschaftliche Modus kann es vergrößern.

Die Hauptansicht enthält nur Ausdruck, Ergebnis und Tastenfeld. Das Menü bietet wissenschaftlichen Modus, Winkeleinheiten, Kopieren und eine eigene Verlaufsseite. Zurück oder Escape kehrt zur Berechnung zurück; ein gewähltes Ergebnis wird wiederverwendet. Der Basismodus stellt die kompakte Größe wieder her und erhält den Ausdruck.

`SUPER + C` → `argvus calculator`

`.desktop`: `/usr/share/applications/argvus-calculator.desktop` · `Name=ARGVUS Calculator` · `Exec=argvus calculator`

ARGVUS: `focus_on_activate = true` (`org.argvus.*`, `sh.argvus.*`, `com.argvus.*`, `argvus-*`, `argvus.*`).

Beim Vergrößern des Fensters wachsen die Tasten. Jeder neue Start verwendet die Standardgröße. Unterseiten haben oben einen Zurück-Pfeil.

`s Settings` → `settings` · Escape / ←

Min: **370 × 420** / **440 × 640** · `0–9`: **16 px**.

`2+2` → Enter / `=` → `4` (input) · `4*3` → Enter → `12`.

Der gewählte Basis- oder Wissenschaftsmodus wird sofort gespeichert und beim nächsten Start wiederhergestellt.

`$XDG_CONFIG_HOME/argvus/calculator/settings.json` (`~/.config/argvus/calculator/settings.json`).
