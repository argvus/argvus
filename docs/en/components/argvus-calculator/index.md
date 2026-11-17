---
title: ARGVUS Calculator
description: ARGVUS Calculator — GTK4 / ARGVUS
---

# ARGVUS Calculator

`argvus calculator` · launcher: **ARGVUS Calculator**

- GTK4 / Relm4; ARGVUS theme (`time`), IBM Plex Mono.
- `+`, `-`, `*`, `/`, `%`, `±`; `50% = 0.5`.
- **Scientific**: `sqrt`, `log`, `ln`, `exp`, `sin`, `cos`, `tan`, `asin`, `acos`, `atan`, `pi`, `e`, `^`, `(`, `)`.
- **Radians**: Unchecked: degrees.
- `-2^2 = -4`; `2^3^2 = 512`; `2*pi`; `1.5 = 1,5`.
- Enter: `=`; Escape: **Clear**; Backspace: **Backspace**; Tab / Shift+Tab.
- **Copy result**; **History**: 100; **Clear history**.
- IEEE 754 / f64; 15 digits; 4096 bytes; 64 levels.

## History

History is saved between launches (up to 100 results). Clearing history also clears the saved records. Settings use selectable rows: ↑/↓ to navigate, Enter or click to activate, Escape to return.

`$XDG_DATA_HOME/argvus/calculator/history.json` (`~/.local/share/argvus/calculator/history.json`)

Always opens as a small, centered floating window (370 × 420); scientific mode can expand it.

The main screen contains only the expression, result and keypad. Use the menu for scientific mode, angle units, copying the result and the separate history screen. Back or Escape returns to calculation; selecting a history result reuses it. Returning to basic mode restores the compact window and preserves the expression.

`SUPER + C` → `argvus calculator`

`.desktop`: `/usr/share/applications/argvus-calculator.desktop` · `Name=ARGVUS Calculator` · `Exec=argvus calculator`

ARGVUS: `focus_on_activate = true` (`org.argvus.*`, `sh.argvus.*`, `com.argvus.*`, `argvus-*`, `argvus.*`).

Resize the window to enlarge the keypad buttons. Every new launch uses the default size. Secondary screens have a back arrow at the top.

`s Settings` → `settings` · Escape / ←

Min: **370 × 420** / **440 × 640** · `0–9`: **16 px**.

`2+2` → Enter / `=` → `4` (input) · `4*3` → Enter → `12`.

The selected basic or scientific mode is saved immediately and restored on the next launch.

`$XDG_CONFIG_HOME/argvus/calculator/settings.json` (`~/.config/argvus/calculator/settings.json`).
