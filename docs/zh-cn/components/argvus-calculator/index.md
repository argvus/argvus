---
title: ARGVUS 计算器
description: ARGVUS 计算器 — GTK4 / ARGVUS
---

# ARGVUS 计算器

`argvus calculator` · launcher: **ARGVUS 计算器**

- GTK4 / Relm4; ARGVUS theme (`time`), IBM Plex Mono.
- `+`, `-`, `*`, `/`, `%`, `±`; `50% = 0.5`.
- **科学**: `sqrt`, `log`, `ln`, `exp`, `sin`, `cos`, `tan`, `asin`, `acos`, `atan`, `pi`, `e`, `^`, `(`, `)`.
- **弧度**: 未选中：角度.
- `-2^2 = -4`; `2^3^2 = 512`; `2*pi`; `1.5 = 1,5`.
- Enter: `=`; Escape: **清除**; Backspace: **删除字符**; Tab / Shift+Tab.
- **复制结果**; **历史记录**: 100; **清除历史记录**.
- IEEE 754 / f64; 15 digits; 4096 bytes; 64 levels.

## 历史记录

关闭后仍保留历史记录（最多100条）。清除历史也会删除保存的记录。设置使用可选行：↑/↓ 导航，Enter 或点击激活，Escape 返回。

`$XDG_DATA_HOME/argvus/calculator/history.json` (`~/.local/share/argvus/calculator/history.json`)

始终以居中的小型浮动窗口打开（370 × 420）；科学模式可扩大窗口。

主界面只显示表达式、结果和键盘。菜单提供科学模式、角度单位、复制结果和独立的历史页面。返回或 Escape 回到计算；选择历史结果可重复使用。切回基本模式会恢复小窗口并保留表达式。

`SUPER + C` → `argvus calculator`

`.desktop`: `/usr/share/applications/argvus-calculator.desktop` · `Name=ARGVUS Calculator` · `Exec=argvus calculator`

ARGVUS: `focus_on_activate = true` (`org.argvus.*`, `sh.argvus.*`, `com.argvus.*`, `argvus-*`, `argvus.*`).

调整窗口大小时按键随之扩大。每次重新启动均使用默认尺寸。子页面顶部有返回箭头。

`s Settings` → `settings` · Escape / ←

Min: **370 × 420** / **440 × 640** · `0–9`: **16 px**.

`2+2` → Enter / `=` → `4` (input) · `4*3` → Enter → `12`.

所选基本或科学模式会立即保存，并在下次启动时恢复。

`$XDG_CONFIG_HOME/argvus/calculator/settings.json` (`~/.config/argvus/calculator/settings.json`).
