---
title: Calcolatrice ARGVUS
description: Calcolatrice ARGVUS — GTK4 / ARGVUS
---

# Calcolatrice ARGVUS

`argvus calculator` · launcher: **Calcolatrice ARGVUS**

- GTK4 / Relm4; ARGVUS theme (`time`), IBM Plex Mono.
- `+`, `-`, `*`, `/`, `%`, `±`; `50% = 0.5`.
- **Scientifica**: `sqrt`, `log`, `ln`, `exp`, `sin`, `cos`, `tan`, `asin`, `acos`, `atan`, `pi`, `e`, `^`, `(`, `)`.
- **Radianti**: Non selezionato: gradi.
- `-2^2 = -4`; `2^3^2 = 512`; `2*pi`; `1.5 = 1,5`.
- Enter: `=`; Escape: **Cancella**; Backspace: **Elimina carattere**; Tab / Shift+Tab.
- **Copia risultato**; **Cronologia**: 100; **Cancella cronologia**.
- IEEE 754 / f64; 15 digits; 4096 bytes; 64 levels.

## Cronologia

La cronologia resta dopo la chiusura (fino a 100 risultati). Cancellarla elimina anche i dati salvati. Impostazioni: ↑/↓ per navigare, Invio o clic per attivare, Escape per tornare.

`$XDG_DATA_HOME/argvus/calculator/history.json` (`~/.local/share/argvus/calculator/history.json`)

Si apre sempre in una piccola finestra flottante centrata (370 × 420); la modalità scientifica può ampliarla.

La schermata principale contiene solo espressione, risultato e tastiera. Il menu offre modalità scientifica, unità angolari, copia e cronologia separata. Indietro o Escape torna al calcolo; selezionare un risultato lo riutilizza. La modalità base ripristina la finestra compatta e conserva l’espressione.

`SUPER + C` → `argvus calculator`

`.desktop`: `/usr/share/applications/argvus-calculator.desktop` · `Name=ARGVUS Calculator` · `Exec=argvus calculator`

ARGVUS: `focus_on_activate = true` (`org.argvus.*`, `sh.argvus.*`, `com.argvus.*`, `argvus-*`, `argvus.*`).

Ridimensionando la finestra si ingrandiscono i tasti. Ogni nuova apertura usa le dimensioni predefinite. Le schermate secondarie hanno una freccia indietro in alto.

`s Settings` → `settings` · Escape / ←

Min: **370 × 420** / **440 × 640** · `0–9`: **16 px**.

`2+2` → Enter / `=` → `4` (input) · `4*3` → Enter → `12`.

La modalità base o scientifica scelta viene salvata subito e ripristinata al prossimo avvio.

`$XDG_CONFIG_HOME/argvus/calculator/settings.json` (`~/.config/argvus/calculator/settings.json`).
