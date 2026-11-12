---
title: پیکیج میٹرکس
description: ARGVUS پیکیج کی ذمہ داریاں۔
---

| پیکیج | ذمہ داری |
| --- | --- |
| `argvus` | ماسٹر پیکج: ڈسپیچر، ان پروسیس کمانڈز (کنفیگریشن، ترجمے، چابیاں، پروجیکٹس، ٹرمینل، سسٹم مانیٹر، نیٹ ورک، بلوٹوتھ، ٹیلی میٹری، TUI ٹرمینل، سیٹ اپ)، مشترکہ ڈیٹا اور i18n کیٹلاگ۔ ہر جزو پیکج اس پر منحصر ہے. |
| `argvus` (مددگار) | اسی پیکج میں مددگار: کنٹرول سینٹر اور ڈسک کے استعمال کا تجزیہ کار (ٹرمینل ایپلی کیشنز)، نیز ٹاسک بار کیلنڈر کے لیے GTK 4 ایپلی کیشنز، ہٹانے کے قابل آلات، لوڈنگ اوورلے، ARGVUS فوکس اور ARGVUS کلاک۔ |
| `argvus ` (سلام کرنے والا) | سلام کے لیے لاگ ان گریٹر، اسی پیکج میں بھیج دیا گیا اور ` crates/argvus-greeter` سے بنایا گیا۔ |
| `argvus-desktop ` | ` argvus ` ماسٹر پیکیج کے لیے ڈیسک ٹاپ اور روزمرہ کی ایپلیکیشن پروفائل درکار ہے۔ یہ ` argvus` پر منحصر نہیں ہے، لہذا انحصار ایک طرفہ رہتا ہے۔ |
| `argvus-devtools `, ` argvus-pentest` | ترقیاتی ٹولز اور مجاز سیکیورٹی آڈیٹنگ کے لیے اختیاری آزاد میٹا پیکجز۔ ماسٹر پیکیج ان پروفائلز کو انسٹال نہیں کرتا ہے۔ |
| `argvus-session` | سیشن انٹری پوائنٹس اور صارف کا لائف سائیکل۔ |
| `argvus-hyprland` | ہائپر لینڈ کنفیگریشن۔ |
| `argvus-control-panel` | کوئیک شیل کنٹرول پینل۔ |
| `argvus-taskbar `, ` argvus-waybar` | وے بار کنفیگریشن اور پیچ شدہ بائنری۔ |
| `argvus-branding` | لوگو اور ورڈ مارک کے اثاثے۔ |
| `argvus-appearance `, ` argvus-wallpapers `, ` argvus-fonts `, ` argvus-icons` | ظاہری حالت اور بصری اثاثے۔ |
| `argvus-display `, ` argvus-power `, ` argvus-notifications` | ڈیسک ٹاپ فراہم کرنے والے۔ |
| `argvus-lock `, ` argvus-boot-splash` | لاک اسکرین اور بوٹ سپلیش۔ |
| `argvus-firewall` | فائر وال، اس کے اپنے مراعات کے ساتھ۔ |
| `argvus-launcher ` | لانچر کنفیگریشن اور انٹری پوائنٹس۔ || ` argvus-app-profiles` | بنڈل ٹرمینل ایپلی کیشنز کے لیے صرف ڈیٹا پروفائلز۔ |
| `argvus-portal` | XDG پورٹل اور Wayland/DBus ڈیفالٹس۔ |
| `argvus-theme-*` | ایک اختیاری پیکج فی آفیشل تھیم فیملی۔ |
| `argvus-game-snake `, ` argvus-games ` | اختیاری سانپ ایک آزاد منصوبہ ہے۔ ` argvus-games` اختیاری گیمز کو گروپ کرتا ہے۔ |

## بدلے گئے پیکیجز

`argvus ` نے ` replaces `، ` conflicts ` اور ` provides ` پیکجوں کے لیے اعلان کیا، اس لیے ایک اپ گریڈ انہیں ہٹا دیتا ہے اور ان کی فائلیں یتیم نہیں رہتیں:` argvus-config `, ` argvus-i18n ` (with ` argvus-language-en-us ` and ` argvus-language-pt-br `), ` argvus-keys `, ` argvus-projects `, ` argvus-terminal `, ` argvus-system-monitor `, ` argvus-network `, ` argvus-widget-telemetry ` and ` argvus-du `. ` argvus ` also replaces ` argvus-control-center `, ` argvus-taskbar-calendar `, ` argvus-loading-theme `, ` argvus-removable-devices `, and the formerتقسیم پیکیجز ` argvus-gtk ` اور ` argvus-greeter `، جو یہ اب بھی فراہم کرتا ہے تاکہ ان پر منحصر پیکیجز حل ہوتے رہیں۔ اکاؤنٹ مینیجر (` argvus accounts `) ` crates/argvus-accounts ` سے بنایا گیا ہے اور اسے `/usr/lib/argvus/commands/accounts ` کے طور پر نصب کیا گیا ہے، اس کے پولکیٹ اصول کے ساتھ۔ ڈسک کے استعمال کا تجزیہ کار ` crates/argvus-du ` سے بنایا گیا ہے اور اسے بطور انسٹال کیا گیا ہے۔`/usr/lib/argvus/commands/argvus-du `۔`/usr/bin/argvus-terminal ` ` argvus terminal` کے لیے مطابقت پذیری لانچر کے طور پر انسٹال رہتا ہے۔ ہائپرلینڈ کا ڈیفالٹ ٹرمینل، کنٹرول سینٹر کیٹلاگ اور محفوظ شدہ ڈیفالٹ ایپلی کیشنز اب بھی اس کمانڈ کا نام رکھتی ہیں۔

ہٹائے گئے پیکیجز `argvus-pwvucontrol ` اور ` argvus-snappy-switcher ` کسی بھی پیکیج کے ذریعے فراہم نہیں کیے گئے ہیں۔ARGVUS جزو پیکجوں میں کسی انسٹال آرڈر کی ضرورت نہیں ہے: ` argvus ` ان کا ماسٹر پیکج ہے، اور ہر جزو اس پر منحصر ہے۔ ` argvus ` انسٹال کرنے سے ` argvus-desktop ` پروفائل بھی انسٹال ہوتا ہے۔ ` argvus-devtools ` اور ` argvus-pentest ` آپٹ ان رہتے ہیں۔ اس ٹیبل میں پیکیج کی موجودگی کا مطلب یہ نہیں ہے کہ وہ پوری خصوصیت کا مالک ہے: مثال کے طور پر، ٹاسک بار ` argvus-waybar `، ` argvus-taskbar `، ` argvus-session` اور ڈومین فراہم کنندگان کو یکجا کرتا ہے۔
