---
title: ARGVUS کیلکولیٹر
description: ARGVUS کیلکولیٹر — GTK4 / ARGVUS
---

# ARGVUS کیلکولیٹر

`argvus calculator` · launcher: **ARGVUS کیلکولیٹر**

- GTK4 / Relm4; ARGVUS theme (`time`), IBM Plex Mono.
- `+`, `-`, `*`, `/`, `%`, `±`; `50% = 0.5`.
- **سائنسی**: `sqrt`, `log`, `ln`, `exp`, `sin`, `cos`, `tan`, `asin`, `acos`, `atan`, `pi`, `e`, `^`, `(`, `)`.
- **ریڈین**: غیر منتخب: درجے.
- `-2^2 = -4`; `2^3^2 = 512`; `2*pi`; `1.5 = 1,5`.
- Enter: `=`; Escape: **صاف کریں**; Backspace: **حرف حذف کریں**; Tab / Shift+Tab.
- **نتیجہ نقل کریں**; **تاریخ**: 100; **تاریخ صاف کریں**.
- IEEE 754 / f64; 15 digits; 4096 bytes; 64 levels.

## تاریخ

بند کرنے کے بعد بھی زیادہ سے زیادہ 100 نتائج محفوظ رہتے ہیں۔ تاریخ صاف کرنے سے محفوظ ریکارڈ بھی حذف ہوتے ہیں۔ ترتیبات میں ↑/↓ سے حرکت، Enter یا کلک سے فعال اور Escape سے واپس جائیں۔

`$XDG_DATA_HOME/argvus/calculator/history.json` (`~/.local/share/argvus/calculator/history.json`)

ہمیشہ درمیان میں چھوٹی تیرتی ونڈو (370 × 420) میں کھلتا ہے؛ سائنسی موڈ اسے بڑا کر سکتا ہے۔

مرکزی اسکرین پر صرف عبارت، نتیجہ اور کی پیڈ ہیں۔ مینو سے سائنسی موڈ، زاویے کی اکائیاں، نقل اور الگ تاریخ کی اسکرین کھولیں۔ واپس یا Escape حساب پر لوٹاتا ہے؛ تاریخ کا نتیجہ منتخب کرنے سے دوبارہ استعمال ہوتا ہے۔ بنیادی موڈ عبارت برقرار رکھتے ہوئے چھوٹی ونڈو بحال کرتا ہے۔

`SUPER + C` → `argvus calculator`

`.desktop`: `/usr/share/applications/argvus-calculator.desktop` · `Name=ARGVUS Calculator` · `Exec=argvus calculator`

ARGVUS: `focus_on_activate = true` (`org.argvus.*`, `sh.argvus.*`, `com.argvus.*`, `argvus-*`, `argvus.*`).

ونڈو بڑی کرنے پر بٹن بڑے ہوتے ہیں۔ ہر نئی شروعات میں طے شدہ سائز استعمال ہوتا ہے۔ دوسری اسکرینوں کے اوپر واپس جانے کا تیر ہے۔

`s Settings` → `settings` · Escape / ←

Min: **370 × 420** / **440 × 640** · `0–9`: **16 px**.

`2+2` → Enter / `=` → `4` (input) · `4*3` → Enter → `12`.

منتخب بنیادی یا سائنسی موڈ فوراً محفوظ ہوتا ہے اور اگلی بار کھولنے پر بحال ہوتا ہے۔

`$XDG_CONFIG_HOME/argvus/calculator/settings.json` (`~/.config/argvus/calculator/settings.json`).
