---
title: فائل کے مقامات
description: اہم انسٹال کردہ ARGVUS فائلیں اور ڈائریکٹریز۔
---

- `/usr/bin/argvus*` - عوامی احکامات۔
- `/usr/share/argvus/hyprland/` - ہائپر لینڈ کنفیگریشن اور مددگار۔
- `/usr/share/argvus/control-panel/` — کنٹرول پینل QML اور اسکرپٹس۔
- `/usr/share/argvus/taskbar/` — ٹاسک بار کنفیگریشن اور اسکرپٹس۔
- `/usr/share/argvus/lock/` - ہائپر لاک کنفیگریشن۔
- `/usr/share/argvus/portal/` — پورٹل انٹیگریشن فائلیں۔
- `/usr/share/backgrounds/argvus/` - وال پیپر کے اثاثے۔
- `/etc/argvus/` — سسٹم فراہم کنندہ کنفیگریشن۔
- `/etc/argvus/greeter.toml` - سلام کنفیگریشن۔
- `/etc/argvus/removable-devices/config.json` — ہٹنے کے قابل اسٹوریج کے لیے سسٹم ڈیفالٹس۔
- `$XDG_CONFIG_HOME/argvus/config.json` - کینونیکل پورٹیبل ترجیحات۔
- `$XDG_CONFIG_HOME/argvus/data/` — نظم شدہ اجزاء کا ڈیٹا، بیک اپ، تالے اور تیار کردہ آؤٹ پٹ۔
- `$XDG_CONFIG_HOME/waybar/argvus-taskbar.{jsonc,css}` - اختیاری مکمل مقامی Waybar ٹاسک بار کو اوور رائیڈ کرتا ہے۔
- `$XDG_CONFIG_HOME/argvus/data/taskbar/argvus-taskbar.{jsonc,css}` — منظم ARGVUS ٹاسک بار کاپیاں۔
- `$XDG_CONFIG_HOME/argvus/data/generated/waybar/` — تیار کردہ Waybar پروفائلز، بشمول `argvus-taskbar.css` اور `argvus-widget-telemetry.{jsonc,css}`؛ یہ اوپر کی کاپیوں پر فوقیت رکھتے ہیں۔
- `$XDG_CONFIG_HOME/argvus/data/generated/waybar/argvus-widget-telemetry.{jsonc,css}` — `data/waybar/` کے تحت بھی آئینہ دار ہے، جو ظاہری شکل میں تبدیلی پر پیکڈ ڈیفالٹ سے بدل دیا جاتا ہے اور پھر اس کے `ARGVUS_TELEMETRY_*` اور فونٹ بلاکس کو دوبارہ لکھا جاتا ہے۔
- `$XDG_CONFIG_HOME/argvus/data/generated/yazi/` — پیش کردہ یازی کنفیگریشن ٹری، تھیمز کے لیے مختلف قسم سے آگاہ فال بیک کے ساتھ جو یازی ذائقہ نہیں بھیجتے ہیں۔
- `$XDG_CONFIG_HOME/argvus/data/generated/superfile/` - پیش کردہ سپر فائل تھیم سیٹ۔
- `$XDG_CONFIG_HOME/argvus/data/generated/terminal/` — متوقع ٹرمینل پروفائلز۔
- `$XDG_CONFIG_HOME/argvus/data/generated/qt6ct/` — پیش کردہ Qt6ct پیلیٹ۔
- `$XDG_CONFIG_HOME/argvus/data/generated/gtk/` - متوقع GTK تھیم فائلیں۔
- `$XDG_CONFIG_HOME/argvus/data/rofi/` — Rofi کنفیگریشن، تھیمز اور موڈ کی صارف کی کاپیاں۔ ARGVUS ہر ایک کاپی تخلیق کرتا ہے جب وہ پہلی بار فائل کو تبدیل کرتا ہے۔ غیر تبدیل شدہ فائلوں کو `/usr/share/argvus/launcher/config/` سے پڑھا جاتا ہے۔
- `$XDG_CONFIG_HOME/argvus/data/generated/removable-devices/theme.css` — متوقع ہٹنے کے قابل آلات کی اسٹائل شیٹ۔
- `$XDG_CONFIG_HOME/argvus/data/focus/settings.json`, `$XDG_DATA_HOME/argvus/focus/history.json`, `$XDG_STATE_HOME/argvus/focus/session.json` — ARGVUS فوکس کی ترتیبات، سیشن کی سرگزشت اور موجودہ سیشن چیک پوائنٹ۔
- `$XDG_RUNTIME_DIR/argvus/focus/status.json` — ٹاسک بار کے ذریعے استعمال ہونے والی عارضی ARGVUS فوکس کی حیثیت؛ فوکس ختم ہونے پر ہٹا دیا گیا۔
- `$XDG_CONFIG_HOME/argvus/data/clock/settings.json`, `$XDG_DATA_HOME/argvus/clock/alarms.json`, `$XDG_STATE_HOME/argvus/clock/timers.json` — ARGVUS گھڑی کی ترتیبات اور عالمی گھڑیاں، الارم، اور ٹائمر/اسٹاپ واچ چیک پوائنٹ۔
- `$XDG_CONFIG_HOME/argvus/data/generated/theme/current/time/theme.css` — ARGVUS فوکس اور ARGVUS کلاک کے رنگ (تھیم صارف `time`)۔
- `/usr/share/argvus/sounds/` — ARGVUS فوکس اور ARGVUS کلاک کی آوازیں (`argvus-chime.oga`, `argvus-beacon.oga`, `argvus-bell.oga`)۔
- `$XDG_CONFIG_HOME/argvus/data/generated/theme/` — تھیم بناتا ہے (`builds/<id>-<variant>-<hash>/`)، `argvus theme render` کے ذریعے لکھا گیا اور ہر تھیم سوئچ اور لہجے میں تبدیلی کے ذریعے، اور `current` فعال تھیم کی تعمیر کے لیے سملنک۔
- `$XDG_CONFIG_HOME/argvus/data/hypr/` — صارف Hyprland Lua کو اوور رائیڈ کرتا ہے اور مقامی Hyprland تخمینہ۔
- `$XDG_CONFIG_HOME/hypr/hyprland.lua` — اختیاری مکمل مقامی Hyprland کنفیگریشن `argvus start` کے ذریعے منتخب کی گئی ہے۔ یہ ایک اضافی اوورلے نہیں ہے۔
- `$XDG_STATE_HOME/argvus/config-projection.json` — پروجیکشن مینی فیسٹ: جنریشن، موثر کنفگ ہیش، تبدیل شدہ سیکشنز اور زیر التواء رن ٹائم کام۔ `argvus sessionctl reload` کے ذریعے پڑھیں؛ کبھی بھی سچائی کا ذریعہ نہیں۔
- `$XDG_STATE_HOME/argvus/widget-telemetry-blocks` — میراثی ٹیلی میٹری بلاک کی ترجیحات۔ صرف منتقلی ان پٹ؛ کیننیکل اقدار `config.json` میں رہتی ہیں اور یہ فائل اب نہیں لکھی جاتی ہے۔
- `$XDG_STATE_HOME/argvus/session.log` — گرافیکل سیشن لاگ (عام طور پر `~/.local/state/argvus/session.log`)۔
- `/usr/share/wayland-sessions/argvus.desktop` — گرافیکل سیشن کا اندراج۔
- `/usr/share/plymouth/themes/argvus/` - پیکڈ بوٹ سپلیش اثاثے۔

عین مطابق پے لوڈ پیکج کے لیے مخصوص ہیں۔ ڈویلپر جزو کا حوالہ اور متعلقہ پیکیج مینی فیسٹ دیکھیں۔