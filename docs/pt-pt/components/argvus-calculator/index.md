---
title: Calculadora ARGVUS
description: Calculadora ARGVUS — GTK4 / ARGVUS
---

# Calculadora ARGVUS

`argvus calculator` · launcher: **Calculadora ARGVUS**

- GTK4 / Relm4; ARGVUS theme (`time`), IBM Plex Mono.
- `+`, `-`, `*`, `/`, `%`, `±`; `50% = 0.5`.
- **Científica**: `sqrt`, `log`, `ln`, `exp`, `sin`, `cos`, `tan`, `asin`, `acos`, `atan`, `pi`, `e`, `^`, `(`, `)`.
- **Radianos**: Desmarcado: graus.
- `-2^2 = -4`; `2^3^2 = 512`; `2*pi`; `1.5 = 1,5`.
- Enter: `=`; Escape: **Limpar**; Backspace: **Apagar caractere**; Tab / Shift+Tab.
- **Copiar resultado**; **Histórico**: 100; **Limpar histórico**.
- IEEE 754 / f64; 15 digits; 4096 bytes; 64 levels.

## Histórico

O histórico mantém-se entre aberturas (até 100 resultados). Limpar o histórico também apaga os registos guardados. Nas configurações, use ↑/↓ para navegar entre linhas, Enter ou clique para ativar e Escape para voltar.

`$XDG_DATA_HOME/argvus/calculator/history.json` (`~/.local/share/argvus/calculator/history.json`)

Abre sempre numa janela flutuante pequena e centrada (370 × 420); o modo científico pode expandi-la.

O ecrã principal contém apenas expressão, resultado e teclado. Use o menu para o modo científico, unidade angular, copiar o resultado e abrir o histórico num ecrã separado. Voltar ou Escape regressa ao cálculo; selecionar um resultado reutiliza-o. O modo básico restaura a janela compacta e preserva a expressão.

`SUPER + C` → `argvus calculator`

`.desktop`: `/usr/share/applications/argvus-calculator.desktop` · `Name=ARGVUS Calculator` · `Exec=argvus calculator`

ARGVUS: `focus_on_activate = true` (`org.argvus.*`, `sh.argvus.*`, `com.argvus.*`, `argvus-*`, `argvus.*`).

Redimensione a janela para ampliar os botões. Cada nova abertura usa o tamanho padrão. Os ecrãs secundários têm uma seta de voltar no topo.

`s Settings` → `settings` · Escape / ←

Min: **370 × 420** / **440 × 640** · `0–9`: **16 px**.

`2+2` → Enter / `=` → `4` (input) · `4*3` → Enter → `12`.

O modo básico ou científico escolhido é guardado imediatamente e restaurado na próxima abertura.

`$XDG_CONFIG_HOME/argvus/calculator/settings.json` (`~/.config/argvus/calculator/settings.json`).
