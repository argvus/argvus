---
title: سروسز
description: ARGVUS سسٹمڈ اور صارف کی خدمات۔
---

فائر وال ایک سسٹم سروس ہے جس کا نام  `argvus-session.target`  `argvus-taskbar.service`  `argvus-control-panel.service`  `argvus-widget-telemetry.service`  `argvus-hyprland.service`  `argvus-dunst.service`  `argvus-wallpaper.service`  `argvus-hypridle.service`  `argvus-session-loading.service` `argvus-firewall.service` ہے۔ کیلنڈر پیکیج `argvus-taskbar-calendar.service` کو صارف یونٹ کے طور پر انسٹال کرتا ہے، اور `argvus-clock.service` (بطور ڈیفالٹ غیر فعال) لاگ ان ہونے کے بعد چھپی ہوئی ARGVUS گھڑی شروع کرتا ہے جب **سیشن کے ساتھ شروع کریں** آن ہوتا ہے۔

اس کے ساتھ فعال یونٹس کا معائنہ کریں:

```sh
systemctl --user status argvus-session.target
systemctl --user list-units 'argvus-*.service'
```

## سیشن کی ملکیت والے یونٹسیشن پیکیج ان صارف یونٹوں کو `/usr/lib/systemd/user` کے تحت انسٹال کرتا ہے (ذریعہ پے لوڈ سیشن پیکج کے مشترکہ کنفیگریشن ٹری کے نیچے ہے):

| یونٹ | کردار |
| --- | --- |
| `argvus-session.target` | ARGVUS گرافیکل سیشن کے لیے لائف ٹائم باؤنڈری۔ |
| `argvus-session-prepare.service` | ایک شاٹ سٹارٹ اپ کی تیاری اور جنریٹڈ کنفیگریشن۔ |
| `argvus-control-panel.service` | کوئیک شیل کنٹرول پینل۔ |
| `argvus-taskbar.service` | مین ARGVUS ٹاسک بار۔ |
| `argvus-widget-telemetry.service` | اختیاری ٹیلی میٹری/sysinfo سطح۔ |
| `argvus-hyprland.service` | آن ڈیمانڈ ون شاٹ یونٹ جو `hyprctl reload` چلاتا ہے۔ |
| `argvus-config.service` | کینونیکل کنفیگریشن کو پروجیکٹ کرتا ہے (`argvus config project`)؛ دوبارہ لوڈ `argvus sessionctl reload-config-transaction` چلتا ہے۔ || `argvus-session-reload.service` | لاگ ان ری لوڈ (`argvus sessionctl reload-desktop --login`): Hyprland کو صرف اس وقت دوبارہ لوڈ کرتا ہے جب پروجیکشن نے اسے متاثر کیا۔ |
| `argvus-ssh-agent.service` | سیشن کے لیے SSH ایجنٹ۔ |
| `argvus-secret-service.service` | `argvus sessionctl start-secret-service` کے ذریعے خفیہ سروس شروع کرتا ہے۔ || `argvus-wallpaper.service` | ایکٹو وال پیپر بیک اینڈ۔ |
| `argvus-dunst.service` | ڈنسٹ نوٹیفکیشن ڈیمون۔ |
| `argvus-hypridle.service` | آئیڈیل اور لاک پالیسی۔ |
| `argvus-clipboard-text.service` / `argvus-clipboard-image.service` | کلپ بورڈ ہسٹری دیکھنے والے۔ |
| `argvus-keyboard-layout.service` | کی بورڈ لے آؤٹ کی اطلاعات۔ |
| `argvus-polkit.service` | پالیسی کٹ ایجنٹ کا انضمام۔ |
| `argvus-session-loading.service` | ڈیسک ٹاپ کے اجزاء شروع ہونے تک کمپوزیٹر ہینڈ آف سے سیشن لوڈنگ اوورلے۔ |

`argvus-blueman-applet.service` سیشن پیکج کے ذریعے بھیجا جاتا ہے۔ کچھ یونٹ اختیاری ہیں یا مشروط طور پر شروع کیے گئے ہیں۔ وہ ہر سیشن میں فعال نہیں ہوتے ہیں۔

صرف اس کے ماخذ چیک آؤٹ کے بجائے یونٹ کی موثر تعریف کا معائنہ کرنا:

```sh
systemctl --user cat argvus-session.target
systemctl --user status argvus-session.target
journalctl --user -u argvus-session.target -u 'argvus-*.service'
```

سسٹم لیول `argvus-firewall.service` `argvus-firewall` سے تعلق رکھتا ہے۔ کیلنڈر پیکیج علیحدہ طور پر `argvus-taskbar-calendar.service` کو بطور صارف یونٹ بھیجتا ہے۔ `argvus-clock.service` `argvus` پیکیج، `WantedBy=graphical-session.target` کا صارف یونٹ ہے، جسے ARGVUS گھڑی کی ترتیبات سے فعال کیا گیا ہے۔

## سیشن مینجمنٹ کمانڈز

`argvus sessionctl` کمانڈ سیشن لائف سائیکل کا انتظام کرتی ہے اور لاگ آؤٹ کیے بغیر مخصوص اجزاء کو دوبارہ شروع کر سکتی ہے:

### سیشن لائف سائیکل

```sh
argvus sessionctl start               # Start a new session
argvus sessionctl stop                # Stop the current session
argvus sessionctl restart             # Restart the entire session
argvus sessionctl reload              # Reload configuration and active services
argvus sessionctl status              # Show session status
argvus sessionctl logs                # Display session logs
```

### جزو کے ساتھ مخصوص دوبارہ شروع ہوتا ہے۔

پورے سیشن کو متاثر کیے بغیر انفرادی خدمات کو دوبارہ شروع کریں:

```sh
# Desktop surfaces
argvus sessionctl restart waybar
argvus sessionctl restart wallpaper
argvus sessionctl restart shell

# Notifications and switcher
argvus sessionctl restart dunst

# Clipboard and input
argvus sessionctl restart clipboard keyboard-layout

# Bluetooth applet (optional)
argvus sessionctl restart blueman-applet
```

### ماحولیات کی مطابقت پذیری۔

سیشن میں ماحولیاتی متغیرات درآمد کریں:

```sh
argvus sessionctl import-environment
```

یہ شیل اور سسٹمڈ صارف خدمات کے درمیان Wayland، Hyprland، Qt، کرسر، اور DBus متغیرات کو ہم آہنگ کرتا ہے۔

## سروس یونٹس کا حوالہ

اضافی صارف یونٹوں کا براہ راست `systemctl --user` کے ساتھ انتظام کیا جا سکتا ہے:

```sh
systemctl --user status argvus-session.target
systemctl --user start argvus-session.target
systemctl --user stop argvus-session.target
systemctl --user restart <unit>

# View logs for a specific service
journalctl --user -u argvus-wallpaper.service -b -f
```

ابتدائی سیشن بوٹسٹریپ کے لاگز یہاں پر محفوظ کیے جاتے ہیں:

```
${XDG_STATE_HOME:-$HOME/.local/state}/argvus/session.log
```
