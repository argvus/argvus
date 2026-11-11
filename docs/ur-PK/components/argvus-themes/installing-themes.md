---
title: تھیمز انسٹال کرنا
description: Pacman کے ساتھ آفیشل ARGVUS تھیم پیکجز انسٹال کریں۔
---

سرکاری تھیمز آرک لینکس پیکجز کے طور پر شائع کیے گئے ہیں۔ ہر تھیم کا اپنا ایک پیکیج ہے جس کا نام `argvus-theme-<THEME_NAME>` ہے۔

## تھیمز کہاں سے حاصل کریں۔

ہر سرکاری تھیم کا ماخذ ARGVUS پروجیکٹ تنظیم میں رہتا ہے:

- ہر تھیم `argvus-theme-<THEME_NAME>` (مثال کے طور پر `argvus-theme-dracula`) نامی ایک ذخیرہ ہے، اسی نام کے ساتھ ایک پیکیج کے طور پر بنایا گیا ہے۔

تھیم پیکجز کا انحصار `argvus-appearance` پر ہے۔ پہلے ARGVUS انسٹال کریں، جیسا کہ [Installation](/docs/getting-started/installation/) میں بیان کیا گیا ہے، تاکہ ظاہری جزو موجود ہو۔

## ایک ہی تھیم انسٹال کریں۔

ہر آفیشل تھیم کو ایک ساتھ انسٹال کرنے کے لیے، `argvus-themes` میٹا پیکج انسٹال کریں:

```bash
sudo pacman -S argvus-themes
```

یہ تمام شائع شدہ `argvus-theme-*` پیکیجز پر منحصر ہے۔ ARGVUS ڈارک اور ARGVUS
روشنی پہلے ہی `argvus-appearance` کے ذریعے بھیجی گئی ہے۔

ایک تھیم کو اس کے پیکیج کے نام کے ساتھ انسٹال کریں:

```bash
sudo pacman -S argvus-theme-<THEME_NAME>
```

مثال کے طور پر:

```bash
sudo pacman -S argvus-theme-dracula
sudo pacman -S argvus-theme-rose-pine
```

پیکیج تھیم مینی فیسٹ، `/usr/share/argvus/themes/<theme-id>/theme.toml` (اس کا نام، زمرہ، وال پیپر اور پیلیٹ، اختیاری `overrides/` کے ساتھ)، اور `/usr/share/backgrounds/argvus/` کے تحت تھیم کا وال پیپر انسٹال کرتا ہے۔ جب تھیم کا اطلاق ہوتا ہے تو ہر جزو کی فائلیں پیلیٹ سے پیش کی جاتی ہیں۔ یہ آپ کی ہوم ڈائرکٹری میں نہیں لکھتا، اور یہ آپ کی موجودہ تھیم کو تبدیل نہیں کرتا ہے۔ اسے بعد میں کنٹرول سینٹر سے منتخب کریں، دیکھیں [آفیشل تھیمز](/docs/components/argvus-themes/official-themes/)۔

## متعدد تھیمز انسٹال کریں۔

کوئی میٹا پیکج نہیں ہے جو ہر تھیم کو انسٹال کرتا ہے۔ اپنے مطلوبہ تھیمز کو ایک کمانڈ میں انسٹال کریں:

```bash
sudo pacman -S argvus-theme-dracula argvus-theme-sunset
```
## دستیاب پیکجز

ذیل کے پیکجز آج شائع ہونے والے سرکاری تھیم پیکجز ہیں۔

** تاریک تھیمز**

| پیکیج | تھیم |
| --- | --- |
| `argvus-theme-one-dark` | ایک سیاہ |
| `argvus-theme-dracula` | ڈریکولا |
| `argvus-theme-silver-dark` | سلور ڈارک |
| `argvus-theme-slate-dark` | سلیٹ ڈارک |
| `argvus-theme-universe` | کائنات |
| `argvus-theme-gruvbox-high-dark` | Gruvbox ہائی ڈارک |
| `argvus-theme-gruvbox-dark` | Gruvbox ڈارک |
| `argvus-theme-rose-pine` | روزے پائن |
| `argvus-theme-tokyo-night` | ٹوکیو نائٹ |
| `argvus-theme-solitude` | تنہائی |
| `argvus-theme-sunset` | غروب آفتاب |
| `argvus-theme-matrix` | میٹرکس |
| `argvus-theme-monokai-dark` | مونوکائی ڈارک |

**ہلکے تھیمز**

| پیکیج | تھیم |
| --- | --- |
| `argvus-theme-github-light` | گٹ ہب لائٹ |
| `argvus-theme-solarized-light` | سولرائزڈ لائٹ |
| `argvus-theme-nord-light` | نورڈ لائٹ |
| `argvus-theme-one-light` | ایک روشنی |
| `argvus-theme-everforest-light` | ایورفورسٹ لائٹ |
| `argvus-theme-catppuccin-latte` | Catppuccin Latte |
| `argvus-theme-gruvbox-light` | Gruvbox لائٹ |
| `argvus-theme-watercolor` | پانی کا رنگ |

بلٹ ان **ARGVUS Dark** اور **ARGVUS Light** تھیمز کو الگ پیکج کی ضرورت نہیں ہے۔ وہ `argvus-appearance` کے ساتھ آتے ہیں۔

## چیک کریں کہ کیا انسٹال ہے۔

انسٹال کردہ تھیم پیکجز کو درج ذیل کے ساتھ درج کریں:

```bash
pacman -Qs argvus-theme
```

کنٹرول سینٹر کھولیں اور سلیکٹر میں فی الحال دستیاب تھیمز دیکھنے کے لیے **ظاہر → تھیمز → آفیشل تھیمز** پر جائیں۔ ایک تھیم جو انسٹال ہے لیکن اس فہرست سے غائب ہے وہ پیکیجنگ کا مسئلہ ہے۔ [تھیمز گائیڈ](/docs/components/argvus-themes/) کا ٹربل شوٹنگ سیکشن دیکھیں۔

## اپنا ایک تھیم شامل کریں۔

ایک تھیم ایک منشور ہے، لہذا آپ کی اپنی تھیم کو کسی پیکیج کی ضرورت نہیں ہے۔ بنائیں
`~/.local/share/argvus/themes/<id>/theme.toml`، جہاں ڈائریکٹری کا نام ہے۔
تھیم ID (چھوٹے حروف، ہندسے اور سنگل ہائفن):

```toml
schema = 2
id = "mist"
name = "Mist"
category = "dark"
wallpaper = "/usr/share/backgrounds/argvus/argvus-dark.jxl"

[palette]
bg = "#101418"
fg = "#D8DEE9"
accent = "#88C0D0"
```

صرف `bg`, `fg` اور `accent` درکار ہیں: ہر دوسرے رنگ کا ڈیفالٹ ہوتا ہے۔
ان سے اخذ کیا گیا ہے، اور مینی فیسٹ صرف اسے تبدیل کرنے کے لیے ایک کی وضاحت کرتا ہے۔ چیک کریں،
پھر اسے کسی دوسرے تھیم کی طرح لاگو کریں:

```bash
argvus theme check ~/.local/share/argvus/themes/mist
argvus theme apply mist
```

`argvus theme list` آپ کے تھیمز کو آفیشل تھیمز کے ساتھ دکھاتا ہے (ایک آفیشل
اسی ID کے ساتھ تھیم جیت جاتی ہے)۔ آپ کے تھیمز تھیم مینو میں ظاہر ہوتے ہیں۔
(`SUPER + Shift + T`); پری لاگ ان گریٹر آپ کی ہوم ڈائرکٹری نہیں پڑھ سکتا، اس لیے
یہ آخری آفیشل تھیم رکھتا ہے جسے آپ نے اپلائی کیا ہے جب کہ آپ کی تھیم میں سے ایک فعال ہے۔

فی تھیم کاپیاں جو پرانے ARGVUS ورژن کے تحت لکھی ہیں۔`~/.config/argvus/data/<app>/themes/` اب نہیں پڑھے جاتے ہیں۔ پہلا تھیم سوئچ
اپ ڈیٹ کے بعد انہیں `~/.config/argvus/data/backups/theme-copies-<time>/` پر منتقل کر دیا جائے گا۔
آپ نے جو فائل ترمیم کی ہے اسے رکھنے کے لیے اسے `overrides/` کے تحت اپنی تھیم میں رکھیں اور
مینی فیسٹ کے `[overrides]` ٹیبل میں اس کا نام دیں۔

## ایک تھیم کو ہٹا دیں۔

ایک ہی تھیم پیکیج کو اس کے ساتھ ہٹائیں:

```bash
sudo pacman -R argvus-theme-<THEME_NAME>
```

تھیم پیکج کو ہٹانے سے آپ کے درآمد شدہ پروفائلز حذف نہیں ہوتے ہیں۔ اگر ہٹائی گئی تھیم فی الحال لاگو کی گئی ہے، تو اسے ہٹانے سے پہلے کنٹرول سینٹر سے کوئی اور تھیم منتخب کریں۔