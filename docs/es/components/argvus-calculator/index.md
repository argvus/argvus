---
title: Calculadora ARGVUS
description: Calculadora ARGVUS — GTK4 / ARGVUS
---

# Calculadora ARGVUS

`argvus calculator` · launcher: **Calculadora ARGVUS**

- GTK4 / Relm4; ARGVUS theme (`time`), IBM Plex Mono.
- `+`, `-`, `*`, `/`, `%`, `±`; `50% = 0.5`.
- **Científica**: `sqrt`, `log`, `ln`, `exp`, `sin`, `cos`, `tan`, `asin`, `acos`, `atan`, `pi`, `e`, `^`, `(`, `)`.
- **Radianes**: Sin marcar: grados.
- `-2^2 = -4`; `2^3^2 = 512`; `2*pi`; `1.5 = 1,5`.
- Enter: `=`; Escape: **Borrar**; Backspace: **Borrar carácter**; Tab / Shift+Tab.
- **Copiar resultado**; **Historial**: 100; **Borrar historial**.
- IEEE 754 / f64; 15 digits; 4096 bytes; 64 levels.

## Historial

El historial se guarda entre aperturas (hasta 100 resultados). Limpiarlo borra también los registros guardados. Configuración: ↑/↓ para navegar, Enter o clic para activar y Escape para volver.

`$XDG_DATA_HOME/argvus/calculator/history.json` (`~/.local/share/argvus/calculator/history.json`)

Siempre se abre en una ventana flotante pequeña y centrada (370 × 420); el modo científico puede ampliarla.

La pantalla principal contiene solo expresión, resultado y teclado. Use el menú para el modo científico, unidades angulares, copiar y abrir el historial separado. Volver o Escape regresa al cálculo; seleccionar un resultado lo reutiliza. El modo básico restaura la ventana compacta y conserva la expresión.

`SUPER + C` → `argvus calculator`

`.desktop`: `/usr/share/applications/argvus-calculator.desktop` · `Name=ARGVUS Calculator` · `Exec=argvus calculator`

ARGVUS: `focus_on_activate = true` (`org.argvus.*`, `sh.argvus.*`, `com.argvus.*`, `argvus-*`, `argvus.*`).

Al redimensionar la ventana aumentan los botones. Cada nueva apertura usa el tamaño predeterminado. Las pantallas secundarias tienen una flecha para volver arriba.

`s Settings` → `settings` · Escape / ←

Min: **370 × 420** / **440 × 640** · `0–9`: **16 px**.

`2+2` → Enter / `=` → `4` (input) · `4*3` → Enter → `12`.

El modo básico o científico se guarda inmediatamente y se restaura al abrir de nuevo.

`$XDG_CONFIG_HOME/argvus/calculator/settings.json` (`~/.config/argvus/calculator/settings.json`).
