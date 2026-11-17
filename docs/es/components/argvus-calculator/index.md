---
title: Calculadora ARGVUS
description: Básico · Científica · Programación · Financiero · Conversión de unidades
---

# Calculadora ARGVUS

`argvus calculator` · `SUPER + C` · launcher: **ARGVUS Calculator**

Elija un perfil en Configuración → Perfil con ↑/↓ y Enter, o pulse una fila. El perfil y las opciones se guardan inmediatamente y se restauran al abrir de nuevo. Cada perfil conserva su expresión durante la sesión. Enter o = sustituye la expresión por el resultado sin seleccionarlo y deja el cursor al final.

Las tasas son porcentajes por período; use el mismo período para tasa, cantidad y pago. Los pagos son al final del período. Se admite tasa cero. Interés simple y compuesto devuelven el total acumulado. La conversión es sin conexión y sin monedas. El historial guarda hasta 100 registros con perfil, opciones y campos financieros; seleccionar un resultado restaura el contexto. Limpiar también borra el archivo guardado.

## Configuración

- `s Configuración` → **Perfil** → Básico / Científica / Programación / Financiero / Conversión de unidades.
- ↑ / ↓ · Enter · Escape / ←.
- **Copiar resultado** · **Historial** · **Borrar historial**.
- **Básico**: 370 × 420; **Científica**: 440 × 640.
- **Programación**: 520 × 640; **Financiero**: 440 × 640; **Conversión de unidades**: 440 × 520.

## Básico / Científica

`+ - * / %` · `50% = 0.5` · `1.5 = 1,5` · `2+2` → Enter → `4` → `+2` → Enter → `6`.

`sin`, `cos`, `tan`, `asin`, `acos`, `atan`, `sqrt`, `log`, `ln`, `exp`, `pi`, `e`, `^`, `(`, `)`.

**Unidad angular**: Grados / Radianes. `-2^2 = -4`; `2^3^2 = 512`; `2*pi`.

IEEE 754 / f64 · 15 dígitos significativos · 4096 bytes · 64 niveles del analizador.

## Programación

Predeterminado: decimal, 64 bits con signo. La aritmética desborda según el ancho elegido; el desplazamiento derecho con signo conserva el signo. La división entera trunca hacia cero. División por cero y desplazamientos fuera de [0, bits) son errores. Sin prefijo se usa la base elegida. Cambiar la base convierte la expresión válida; ancho o signo reinterpretan los mismos bits.

- **Base**: 2 / 8 / 10 / 16; **Tamaño de palabra (bits)**: 8 / 16 / 32 / 64.
- **Con signo** / **Sin signo** · `0b1010`, `0o12`, `0xA`.
- `+ - * / %`, `AND OR XOR NOT`, `<< >>`, `& | ^ ~`, `(`, `)`.
- `9007199254740993+2 = 9007199254740995`; `0xFF AND 0b1010 = 10` (base 10).
- 8-bit: `255+1 = 0`; `NOT 0 = -1` (Con signo); `-7/2 = -3`.

## Financiero

Interés simple · Interés compuesto · Cuota fija · Valor presente · Valor futuro · Variación porcentual.

- `P`: Capital inicial; `FV`: Importe futuro; `A`: Pago por período.
- `r`: Tasa por período (%) / 100; `n`: Número de períodos.
- `G=(1+r)^n`; `H=(G-1)/r` (`r=0`: `H=n`).
- Interés simple: `P*(1+r*n)`; Interés compuesto: `P*G`.
- Cuota fija: `P*G/H`; Valor presente: `(FV+A*H)/G`; Valor futuro: `P*G+A*H`.
- Variación porcentual: `(Importe final - Importe original) / Importe original * 100`.

## Conversión de unidades

Longitud · Masa · Temperatura · Área · Volumen · Tiempo · Velocidad · Almacenamiento de datos.

**De** / **A**: Configuración · `1500 m → 1.5 km` · `32 °F → 0 °C` · `K ≥ 0`.

`1 B = 8 bit`; `1 kB = 1000 B`; `1 KiB = 1024 B`; `US gal = 3.785411784 L`.

## Historial

100 · Perfil + `context` · `Borrar historial`.

`$XDG_DATA_HOME/argvus/calculator/history.json` (→ `~/.local/share/argvus/calculator/history.json`).

`$XDG_CONFIG_HOME/argvus/calculator/settings.json` (→ `~/.config/argvus/calculator/settings.json`).

`{"scientific":true}` → `{"profile":"scientific"}`; `{"scientific":false}` → `{"profile":"basic"}`.

`.desktop`: `/usr/share/applications/argvus-calculator.desktop` · `Name=ARGVUS Calculator` · `Exec=argvus calculator`.

Los textos siguen Applications Font en Control Center → Appearance → Desktop Style → Fonts. Las aplicaciones GTK heredan gtk-font-name; se mantienen los tamaños de números y títulos. El lanzador lee apps_family generado y observa los cambios.

La cabecera combina una insignia ARGVUS del color de acento con el nombre de la aplicación. El pie pulsable s Settings ocupa una franja de ancho completo con un fondo distinto del tema, siguiendo el estilo del Control Center.
