---
title: ARGVUS 计算器
description: 基本 · 科学 · 编程 · 金融 · 单位换算
---

# ARGVUS 计算器

`argvus calculator` · `SUPER + C` · launcher: **ARGVUS Calculator**

在设置 → 模式中使用 ↑/↓ 和 Enter，或点击一行选择模式。模式及选项会立即保存并在下次启动时恢复。每种模式在当前会话中保留独立的表达式。Enter 或 = 将表达式替换为结果，不选中文本，光标位于末尾。

金融利率为每期百分比；利率、期数和付款必须使用同一周期。付款发生在期末，支持零利率。单利和复利返回累计总额。单位换算完全离线，不含货币。历史保留最多100条记录，包括模式、选项和金融输入；选择结果会恢复上下文。清除历史也会清除保存的文件。

## 设置

- `s 设置` → **模式** → 基本 / 科学 / 编程 / 金融 / 单位换算.
- ↑ / ↓ · Enter · Escape / ←.
- **复制结果** · **历史记录** · **清除历史记录**.
- **基本**: 370 × 420; **科学**: 440 × 640.
- **编程**: 520 × 640; **金融**: 440 × 640; **单位换算**: 440 × 520.

## 基本 / 科学

`+ - * / %` · `50% = 0.5` · `1.5 = 1,5` · `2+2` → Enter → `4` → `+2` → Enter → `6`.

`sin`, `cos`, `tan`, `asin`, `acos`, `atan`, `sqrt`, `log`, `ln`, `exp`, `pi`, `e`, `^`, `(`, `)`.

**角度单位**: 角度 / 弧度. `-2^2 = -4`; `2^3^2 = 512`; `2*pi`.

IEEE 754 / f64 · 15位有效数字 · 4096字节 · 64层解析.

## 编程

默认：十进制、有符号64位。算术溢出按字长回绕，有符号右移保留符号。整数除法向零截断。除以零或移位数超出 [0, bits) 会报错。无前缀数字使用所选进制。切换进制转换有效表达式；切换字长或符号重新解释相同位模式。

- **进制**: 2 / 8 / 10 / 16; **字长（位）**: 8 / 16 / 32 / 64.
- **有符号** / **无符号** · `0b1010`, `0o12`, `0xA`.
- `+ - * / %`, `AND OR XOR NOT`, `<< >>`, `& | ^ ~`, `(`, `)`.
- `9007199254740993+2 = 9007199254740995`; `0xFF AND 0b1010 = 10` (base 10).
- 8-bit: `255+1 = 0`; `NOT 0 = -1` (有符号); `-7/2 = -3`.

## 金融

单利 · 复利 · 固定付款 · 现值 · 终值 · 百分比变化.

- `P`: 初始本金; `FV`: 未来金额; `A`: 每期付款.
- `r`: 每期利率（%） / 100; `n`: 期数.
- `G=(1+r)^n`; `H=(G-1)/r` (`r=0`: `H=n`).
- 单利: `P*(1+r*n)`; 复利: `P*G`.
- 固定付款: `P*G/H`; 现值: `(FV+A*H)/G`; 终值: `P*G+A*H`.
- 百分比变化: `(最终金额 - 原始金额) / 原始金额 * 100`.

## 单位换算

长度 · 质量 · 温度 · 面积 · 体积 · 时间 · 速度 · 数据存储.

**从** / **到**: 设置 · `1500 m → 1.5 km` · `32 °F → 0 °C` · `K ≥ 0`.

`1 B = 8 bit`; `1 kB = 1000 B`; `1 KiB = 1024 B`; `US gal = 3.785411784 L`.

## 历史记录

100 · 模式 + `context` · `清除历史记录`.

`$XDG_DATA_HOME/argvus/calculator/history.json` (→ `~/.local/share/argvus/calculator/history.json`).

`$XDG_CONFIG_HOME/argvus/calculator/settings.json` (→ `~/.config/argvus/calculator/settings.json`).

`{"scientific":true}` → `{"profile":"scientific"}`; `{"scientific":false}` → `{"profile":"basic"}`.

`.desktop`: `/usr/share/applications/argvus-calculator.desktop` · `Name=ARGVUS Calculator` · `Exec=argvus calculator`.

应用文本使用 Control Center → Appearance → Desktop Style → Fonts 中的 Applications Font。GTK 应用继承 gtk-font-name，保留数字和标题的独立字号。启动器读取生成的 apps_family 并监听更改。

顶部将强调色的 ARGVUS 标识与应用名称并列。可点击的 s Settings 页脚采用通栏背景，与主题主背景区分，延续 Control Center 的视觉层次。
