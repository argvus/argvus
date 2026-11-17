---
title: ARGVUS Calculator
description: Basic · Scientific · Programming · Financial · Unit conversion
---

# ARGVUS Calculator

`argvus calculator` · `SUPER + C` · launcher: **ARGVUS Calculator**

Choose a profile in Settings → Profile using ↑/↓ and Enter, or click a row. The profile and its options are saved immediately and restored on the next launch. Each profile keeps its own expression while the application is open. Enter or = replaces the expression with its result without selecting it; the cursor stays at the end.

Financial rates are percentages per period; use the same period for rate, count and payment. Payments occur at period end. Zero rates are supported. Simple and compound interest return the accumulated total. Conversion is offline, without currencies. History persists up to 100 records, including profile/options and financial inputs; selecting a result restores that context. Clearing history also clears the saved file.

## Settings

- `s Settings` → **Profile** → Basic / Scientific / Programming / Financial / Unit conversion.
- ↑ / ↓ · Enter · Escape / ←.
- **Copy result** · **History** · **Clear history**.
- **Basic**: 370 × 420; **Scientific**: 440 × 640.
- **Programming**: 520 × 640; **Financial**: 440 × 640; **Unit conversion**: 440 × 520.

## Basic / Scientific

`+ - * / %` · `50% = 0.5` · `1.5 = 1,5` · `2+2` → Enter → `4` → `+2` → Enter → `6`.

`sin`, `cos`, `tan`, `asin`, `acos`, `atan`, `sqrt`, `log`, `ln`, `exp`, `pi`, `e`, `^`, `(`, `)`.

**Angle unit**: Degrees / Radians. `-2^2 = -4`; `2^3^2 = 512`; `2*pi`.

IEEE 754 / f64 · 15 significant digits · 4096 bytes · 64 parser levels.

## Programming

Defaults: decimal, signed 64-bit. Arithmetic wraps at the selected word width; signed right shifts extend the sign. Integer division truncates toward zero. Division by zero and shift counts outside [0, bits) are errors. Unprefixed numbers use the selected base. Changing base converts the current valid expression; changing width or sign reinterprets the same bit pattern.

- **Base**: 2 / 8 / 10 / 16; **Word size (bits)**: 8 / 16 / 32 / 64.
- **Signed** / **Unsigned** · `0b1010`, `0o12`, `0xA`.
- `+ - * / %`, `AND OR XOR NOT`, `<< >>`, `& | ^ ~`, `(`, `)`.
- `9007199254740993+2 = 9007199254740995`; `0xFF AND 0b1010 = 10` (base 10).
- 8-bit: `255+1 = 0`; `NOT 0 = -1` (Signed); `-7/2 = -3`.

## Financial

Simple interest · Compound interest · Fixed payment · Present value · Future value · Percentage change.

- `P`: Principal; `FV`: Future amount; `A`: Payment per period.
- `r`: Rate per period (%) / 100; `n`: Number of periods.
- `G=(1+r)^n`; `H=(G-1)/r` (`r=0`: `H=n`).
- Simple interest: `P*(1+r*n)`; Compound interest: `P*G`.
- Fixed payment: `P*G/H`; Present value: `(FV+A*H)/G`; Future value: `P*G+A*H`.
- Percentage change: `(Final amount - Original amount) / Original amount * 100`.

## Unit conversion

Length · Mass · Temperature · Area · Volume · Time · Speed · Data storage.

**From** / **To**: Settings · `1500 m → 1.5 km` · `32 °F → 0 °C` · `K ≥ 0`.

`1 B = 8 bit`; `1 kB = 1000 B`; `1 KiB = 1024 B`; `US gal = 3.785411784 L`.

## History

100 · Profile + `context` · `Clear history`.

`$XDG_DATA_HOME/argvus/calculator/history.json` (→ `~/.local/share/argvus/calculator/history.json`).

`$XDG_CONFIG_HOME/argvus/calculator/settings.json` (→ `~/.config/argvus/calculator/settings.json`).

`{"scientific":true}` → `{"profile":"scientific"}`; `{"scientific":false}` → `{"profile":"basic"}`.

`.desktop`: `/usr/share/applications/argvus-calculator.desktop` · `Name=ARGVUS Calculator` · `Exec=argvus calculator`.

Application text follows Applications Font in Control Center → Appearance → Desktop Style → Fonts. GTK applications inherit gtk-font-name; local sizes for numbers and headings remain. The launcher reads the generated apps_family and watches for changes.

The header combines an accent-colored ARGVUS badge with the application name. The clickable s Settings footer occupies a full-width band with a distinct theme surface, following the Control Center visual hierarchy.
