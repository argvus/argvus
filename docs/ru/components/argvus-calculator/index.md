---
title: Калькулятор ARGVUS
description: Калькулятор ARGVUS — GTK4 / ARGVUS
---

# Калькулятор ARGVUS

`argvus calculator` · launcher: **Калькулятор ARGVUS**

- GTK4 / Relm4; ARGVUS theme (`time`), IBM Plex Mono.
- `+`, `-`, `*`, `/`, `%`, `±`; `50% = 0.5`.
- **Научный**: `sqrt`, `log`, `ln`, `exp`, `sin`, `cos`, `tan`, `asin`, `acos`, `atan`, `pi`, `e`, `^`, `(`, `)`.
- **Радианы**: Без отметки: градусы.
- `-2^2 = -4`; `2^3^2 = 512`; `2*pi`; `1.5 = 1,5`.
- Enter: `=`; Escape: **Очистить**; Backspace: **Удалить символ**; Tab / Shift+Tab.
- **Копировать результат**; **История**: 100; **Очистить историю**.
- IEEE 754 / f64; 15 digits; 4096 bytes; 64 levels.

## История

История сохраняется после закрытия (до 100 результатов). Очистка удаляет и сохранённые записи. Настройки: ↑/↓ для навигации, Enter или щелчок для активации, Escape для возврата.

`$XDG_DATA_HOME/argvus/calculator/history.json` (`~/.local/share/argvus/calculator/history.json`)

Всегда открывается в небольшом плавающем окне по центру (370 × 420); научный режим может увеличить его.

На главном экране только выражение, результат и клавиатура. Меню открывает научный режим, единицы углов, копирование и отдельную историю. Назад или Escape возвращает к расчёту; выбор результата использует его повторно. Базовый режим восстанавливает компактный размер, сохраняя выражение.

`SUPER + C` → `argvus calculator`

`.desktop`: `/usr/share/applications/argvus-calculator.desktop` · `Name=ARGVUS Calculator` · `Exec=argvus calculator`

ARGVUS: `focus_on_activate = true` (`org.argvus.*`, `sh.argvus.*`, `com.argvus.*`, `argvus-*`, `argvus.*`).

При увеличении окна кнопки растут. При каждом новом запуске используется стандартный размер. На дополнительных экранах сверху есть стрелка назад.

`s Settings` → `settings` · Escape / ←

Min: **370 × 420** / **440 × 640** · `0–9`: **16 px**.

`2+2` → Enter / `=` → `4` (input) · `4*3` → Enter → `12`.

Выбранный базовый или научный режим сразу сохраняется и восстанавливается при следующем запуске.

`$XDG_CONFIG_HOME/argvus/calculator/settings.json` (`~/.config/argvus/calculator/settings.json`).
