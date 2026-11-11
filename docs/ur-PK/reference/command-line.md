---
title: کمانڈ لائن
description: argvus کمانڈ، اس کی ذیلی کمانڈ، کی بائنڈنگ کے لیے رکھے گئے جھنڈے، اور جہاں ہر ایک چلتا ہے۔
---

`/usr/bin/argvus` ARGVUS کا پبلک انٹری پوائنٹ ہے۔ یہ ایک زنگ پروگرام ہے جو `argvus-cli` کریٹ سے بنایا گیا ہے۔ ہر ذیلی کمانڈ یا تو اس پروگرام کے اندر چلتا ہے، کمانڈ ڈائرکٹری سے ایک ہیلپر چلاتا ہے، یا پیکیجڈ اسکرپٹ چلاتا ہے۔

اپنے ورژن کے ساتھ انسٹال کردہ فہرست کے لیے `argvus --help` چلائیں۔ ذیلی کمانڈز معروف `--` کے بغیر لکھے گئے ہیں۔ `--` فارم صرف اسی جگہ رکھے جاتے ہیں جہاں کی بائنڈنگ یا ڈیسک ٹاپ اندراج انہیں استعمال کرتا ہے۔

## جہاں ہر کمانڈ چلتا ہے۔- **In-process:** the command is a library of the workspace and runs inside `argvus`. No separate executable exists. These are `config`, `theme`, `i18n`, `keys`, `gpg`, `projects`, `terminal`, `system-monitor`, `network`, `bluetooth`, `focus status`, `widget-telemetry`,`tui-terminal`، `sensors` اور `setup`۔- 
- 
- **اسکرپٹ:** ایک پیک شدہ شیل اسکرپٹ  greeter-*  taskbar-actions  accounts  firewall  displayctl  notifications  app-profiles-yazi-keymap  git  ssh  snippets  launcher  sessionctl  tty  start  session  argvus  `/usr/lib/argvus/commands/`  clock  focus  removable-devices  taskbar-calendar  argvus du  argvus-du  control-center  `argvus`   `control-center`  `argvus-du`  `argvus du`  `taskbar-calendar`  `removable-devices`  `focus`  `clock` `/usr/lib/argvus/commands/`  `argvus`  `session`  `start`  `tty`  `sessionctl`  `launcher`  `snippets`  `ssh`  `git`  `app-profiles-yazi-keymap`  `notifications`  `displayctl`  `firewall`  `accounts`  `taskbar-actions`  `greeter-*` `/bin/sh` کے ساتھ چلتی ہے۔ یہ ذیل میں درج ہائپرلینڈ اسکرپٹ کمانڈز ہیں۔

`ARGVUS_COMMANDS_DIR` ٹیسٹ اور `DESTDIR` چیکس کے لیے کمانڈ ڈائرکٹری کو تبدیل کرتا ہے۔ ایک پرائیویٹ ایگزیکیوٹیبل جو وہاں نہیں ملا اسے `PATH` پر دیکھا جاتا ہے۔

## ذیلی کمانڈز

| کمانڈ | رنز | نوٹس |
| --- | --- | --- |
| `argvus config ...` | عمل میں | کنفیگریشن ماڈل اور ٹولز۔ `theme-switch` اور `effects-toggle` کے ذریعے بھی پہنچا۔ |
| `argvus theme ...` | عمل میں | `apply ID` تھیم سوئچ ہے (`theme-switch` کی طرح)۔ تھیم بناتا ہے: `list` انسٹال کردہ تھیمز، `check DIRECTORY` انسٹال شدہ ٹیمپلیٹس کے خلاف ایک تھیم، `render ID [--variant sticky\|spaced] [--accent #RRGGBB] [--output DIRECTORY]`۔ دیکھیں [ظاہر اور تھیمز](/docs/developer-guide/subsystems/appearance-and-themes/)۔ |
| `argvus i18n ...` | عمل میں | ترجمہ: `get DOMAIN KEY [name=value...]`, `locale`, `list`, `validate [LOCALE]`۔ || `argvus keys ...`                                            ۔ﺮﺠﯿﻨﻣ ﯽﮐ ﭻﯾﺍ ﺲﯾﺍ ﺲﯾﺍ | ﻞﻤﻋ ﺮﯾﺯ | `gpg`GPG ۔ﮯﮨ ﻡﺎﻧ ﯽﻓﺮﻋ ﮏﯾﺍ ﮯﯿﻟ ﮯﮐ ﺯﮉﻧﺎﻤﮐ ﯽﻠﯾﺫ |
| `argvus gpg ...` | In-process | GPG keys: `list`, `status`, `generate`, `export-public`, `export-private`, `import`, `import-private`, `delete`, `edit`, `keyserver`. |
| `argvus projects ...` | In-process | Project picker. |
| `argvus terminal ...` | عمل میں | ARGVUS ٹرمینل پروفائل۔ |
| `argvus system-monitor ...` | عمل میں | btop سسٹم مانیٹر؛ `--gpu` صرف GPU باکسز کو منتخب کرتا ہے، اور `--panel [--cpu|--mem|--gpu]` اسے ARGVUS TUI شیل میں سرایت کرتا ہے۔ |
| `argvus du` | مددگار (ٹرمینل ایپلی کیشن) | ایک وقف کٹی ونڈو میں ARGVUS ڈسک کے استعمال کے تجزیہ کار کو کھولتا ہے اور ncdu چلاتا ہے۔ || `argvus network ...` | عمل میں | نیٹ ورک مینیجر کی حیثیت اور کنٹرولز (`networkctl` کو بطور عرف رکھا جاتا ہے)۔ || `argvus bluetooth ...` | عمل میں | بلوٹوتھ کی حیثیت اور کنٹرولز (`bluetoothctl` کو بطور عرف رکھا جاتا ہے)۔ || `argvus widget-telemetry block ...` | عمل میں | وے بار ٹیلی میٹری بلاک۔ یہ بار کا گرم راستہ ہے۔ || `argvus widget-telemetry toggle ...` | عمل میں | ٹیلی میٹری بلاکس کو آن اور آف کرتا ہے، اور ان کا آرڈر۔ |
| `argvus game-snake` | پرائیویٹ ایگزیکیوٹیبل | ٹرمینل سانپ گیم، آزاد `argvus-game-snake` پروجیکٹ سے، ایک اختیاری پیکیج۔ ٹرمینل کے باہر یہ ایک کھولتا ہے۔ |
| `argvus control-center [page]` | مددگار (ٹرمینل ایپلی کیشن) | کنٹرول سینٹر کھولتا ہے، یا پہلے سے کھلی ہوئی ونڈو کو فوکس کرتا ہے۔ |
| `argvus welcome [--if-needed]` | ڈسپیچر → کنٹرول سینٹر | کنٹرول سینٹر میں انٹیگریٹڈ ویلکم پیج کھولتا ہے۔ `--if-needed` ایک بار کھلتا ہے جب تک کہ فی صارف پیش رفت ریکارڈ نہ کرے کہ ویلکم پہلے سے ظاہر، مکمل یا برخاست کر دیا گیا ہے۔ |
| `argvus taskbar-calendar [args]` | GTK 4 ایپلیکیشن | `toggle` پر ڈیفالٹس۔ |
| `argvus removable-devices [args]` | GTK 4 ایپلیکیشن | `menu` پر ڈیفالٹس۔ |
| `argvus focus [--toggle] [--start\|--pause] [--x X --y Y] [--bar-bottom PX --bar-right PX]` | GTK 4 ایپلیکیشن | ARGVUS فوکس پاپ اپ۔ فی سیشن ایک مثال۔ |
| `argvus focus status --waybar [--follow]` | عمل میں | فوکس ٹاسک بار کے اشارے کو Waybar JSON کے طور پر خارج کرتا ہے۔ `--follow` GTK لوڈ کیے بغیر تبدیلیاں کرتا ہے۔ |
| `argvus clock [--toggle] [--background] [--page PAGE] [--x X --y Y] [--bar-bottom PX --bar-right PX]` | GTK 4 ایپلیکیشن | ARGVUS کلاک پاپ اپ۔ فی سیشن ایک مثال؛ `--background` بغیر کسی پاپ اپ کے الارم کو متحرک رکھتا ہے۔ |
| `argvus loading-theme ...` | پیک شدہ `splash` `/usr/lib/argvus/loading-theme/splash` پر | براہ راست سیشن دوبارہ لوڈ ہونے کا اوورلے؛ ایک تھیم سوئچ اسے استعمال نہیں کرتا ہے۔ |
| `argvus session`, `start`, `tty`, `sessionctl` | پرائیویٹ ایگزیکیوٹیبلز | سیشن انٹری پوائنٹس اور سیشن کنٹرولر۔ |
| `argvus greeter`, `greeter-session`, `greeter-tui`, `greeter-setup` | پرائیویٹ ایگزیکیوٹیبلز | گریٹر انٹری پوائنٹس۔ |
| `argvus accounts`, `firewall` | پرائیویٹ ایگزیکیوٹیبلز | اکاؤنٹس اور فائر وال ٹولز۔ |
| `argvus launcher`, `snippets`, `ssh`, `git` | پرائیویٹ ایگزیکیوٹیبلز | لانچر اور چننے والے ٹولز۔ |
| `argvus tui-terminal` | عمل میں | TUI ایپلی کیشنز کے لیے ٹرمینل ریپر۔ |
| `argvus sensors` | عمل میں | `sensors -j` سے براہ راست درجہ حرارت کا جائزہ۔ |
| `app-profiles-yazi-keymap` | پرائیویٹ ایگزیکیوٹیبل | کلیدی نقشہ مددگار۔ |
| `argvus notifications`, `displayctl`, `taskbar-actions` | پرائیویٹ ایگزیکیوٹیبلز | اطلاعات، ڈسپلے اور ٹاسک بار کے اعمال۔ |
| `argvus setup [options]` | عمل میں | پیکیج شدہ ڈیفالٹس کو صارف کی ترتیب میں کاپی کرتا ہے۔ نیچے دیکھیں۔ |

آرگیومینٹس بغیر تبدیلی کے آگے بھیجے جاتے ہیں، بشمول خالی جگہیں اور خالی اقدار۔ معیاری ان پٹ، آؤٹ پٹ، ایرر، سگنلز اور ایگزٹ اسٹیٹس کا تعلق اس کمانڈ سے ہے جو چلتی ہے۔ ایک نامعلوم نام `argvus --help` چلانے کے اشارے کے ساتھ ناکام ہو جاتا ہے۔ ایک کمانڈ جو عمل میں چلتی ہے اس دلیل سے انکار کرتی ہے جو درست نہیں ہے UTF-8 (ایگزٹ اسٹیٹس 2) بدلی ہوئی قدر کو منتقل کرنے کے بجائے؛ کمانڈز جو ایک الگ پروگرام چلاتے ہیں بائٹس کو بغیر کسی تبدیلی کے وصول کرتے ہیں۔

## جھنڈے اور عرفی نام

یہ نام موجود ہیں تاکہ کی بائنڈنگز، ڈیسک ٹاپ اندراجات اور پرانے اسکرپٹ کام کرتے رہیں۔ نئے کوڈ میں سب کمانڈ فارم کو ترجیح دیں۔

| جھنڈا یا عرف | مساوی |
| --- | --- |
| `--control-center`, `--about`, `--default-apps` | `control-center` (`about` اور `apps` صفحات کے ساتھ) |
| `--displays`, `--monitors` صفحہ | `control-center displays` |
| `--network-interfaces` | `control-center network interfaces` |
| `--system-monitor`, `--terminal`, `--game-snake` | `system-monitor`, `terminal`, `game-snake` |
| `--calendar`, `--removable-devices`, `--storage` | `taskbar-calendar`, `removable-devices` |
| `--focus`, `--clock` | `focus`, `clock` |
| `--spf`, `--superfile` | فائل مینیجر (`spf`) |
| `--yazi`, `--yazy` | فائل براؤزر (`yazi`) |
| `--setup` | `setup` |
| `--start-desktop` | `tty` شروع ہوتا ہے (کوئی دلائل آگے نہیں بھیجے جاتے ہیں) |
| `--version`, `-V`, `version` | پرنٹس `ARGVUS <version>` |
| `help`, `--help`, `-h` | استعمال پرنٹ کرتا ہے || `networkctl`, `bluetoothctl` | `network`, `bluetooth` |
| `widget-telemetry-toggle`, `widget-telemetry-block` | `widget-telemetry toggle`, `widget-telemetry block` |
| `theme-switch` | `config theme-apply` |
| `effects-toggle` | `config effects` |

:ﮟﯿﻟﺎﺜﻣ ۔ﮯﮨ ﺎﺗﮌﻮﺟ ﮯﺳ ﮞﻮﻣﺎﻧ ﺞﯿﭘ ﮟﯿﮩﻧﺍ ﺭﻭﺍ ﮯﮨ ﺎﺗﺮﮐ ﻝﻮﺒﻗ ﻮﮐ ﺰﮕﯿﻠﻓ ﺞﯿﭘ ﯽﻧﺍﺮﭘ ﺮﭘ ﺭﻮ`--apps` becomes `apps`, `--keyboard-layout` becomes `locale`, and `--hostname` becomes `system`.

## ﺯﮉﻧﺎﻤﮐ ﭧﭘﺮﮑﺳﺍ

یہ پیکڈ اسکرپٹس چلاتے ہیں اور ہائپرلینڈ کنفیگریشن کے ذریعہ استعمال ہوتے ہیں: system  --hostname  locale  --keyboard-layout  apps  --apps  apps  --keyboard-layout  locale  --hostname  system `cheatsheets-hypr`, `cheatsheets-launcher`, `scratchpad`, `hypr-screenshot`, `spaces-switch`, `emoji-picker`, `toggle-sidebar`, `weather-location`, `layout-mode-menu`, `brightness-switch`, `toggle-mode`, `idle-timeout`, `keep-awake`, `hypr-power-menu`.

صارف کی ترتیب میں ایک اسکرپٹ پیک شدہ کو اوور رائیڈ کرتا ہے۔ تلاش کا آرڈر `_config_path` میں `hyprland.lua` جیسا ہی ہے: صارف کی کنفیگریشن کاپی، پھر ARGVUS ڈیٹا کاپی، پھر اسٹیٹ کاپی، پھر جنریٹڈ کاپی، اور صرف اس کے بعد `/usr/share/argvus` کے تحت پیک کردہ فائل۔## مثالیں۔

```sh
argvus control-center displays
argvus taskbar-calendar toggle
argvus widget-telemetry block header system
argvus widget-telemetry toggle status
argvus i18n get widget-telemetry label.device
argvus config get /appearance/theme
argvus network status
argvus bluetooth status
argvus gpg list
```

## سیٹ اپ`argvus setup` پیکڈ ڈیفالٹس کو `/usr/share/argvus/<app>` سے `~/.config/argvus/data/<app>` میں کاپی کرتا ہے، تاکہ آپ انہیں اپنی مرضی کے مطابق بنا سکیں۔ یہ کبھی بھی پیک شدہ فائلوں میں ترمیم نہیں کرتا ہے۔

```sh
argvus setup --copy <app>          # copy one packaged app
argvus setup --copy-all            # copy every packaged app directory
argvus setup --copy <app> --force  # back up the existing destination first
argvus setup --copy <app> --dry-run
```

ایک منزل جو پہلے سے موجود ہے اس کی اطلاع دی جاتی ہے اور اسے تنہا چھوڑ دیا جاتا ہے، جب تک کہ `--force` نہ دیا جائے۔ `--force` کے ساتھ، موجودہ ڈائریکٹری کا نام کاپی سے پہلے `<name>.bak-<timestamp>` رکھ دیا گیا ہے۔

`argvus` ایپ خاص ہے: یہ اپنی پیکڈ ڈائرکٹری کے مواد کو کنفیگریشن روٹ میں کاپی کرتی ہے۔ چونکہ سیٹ اپ کے چلنے پر کنفیگریشن روٹ پہلے سے موجود ہے، `--copy argvus` کو موجودہ کوڈ میں `--force` کی ضرورت ہے۔

سیٹ اپ کسی بھی چیز کو تبدیل کرنے سے پہلے ہر آپشن کی توثیق کرتا ہے۔ غلط ایپ کے نام (خالی، `.` سے شروع ہونے والے، یا `/` پر مشتمل) کو مسترد کر دیا گیا ہے۔

## سیشن اور دوبارہ لوڈ کریں۔

`SUPER + SHIFT + R` `argvus sessionctl reload` چلاتا ہے۔ سیشن کنٹرولر تیار کردہ کنفیگریشن کو لاگو کرتا ہے اور صرف ان اجزاء کو دوبارہ شروع کرتا ہے جو اس کی ملکیت ہیں۔ یہ `/usr/share/argvus` کو ہوم ڈائرکٹری میں کاپی نہیں کرتا ہے، اور یہ ایک خراب JSON فائل کو تبدیل نہیں کرتا ہے۔

## نائٹ لائٹ

**ڈیسک ٹاپ اور ان پٹ> ڈسپلے> نائٹ لائٹ** کھولیں یا `argvus control-center displays night-light` چلائیں۔ `argvus config display night-light` کے ذریعے درجہ حرارت، دستی ایکٹیویشن اور فکسڈ لوکل شیڈول کو ترتیب دیں۔ کمانڈز، حالت اور رن ٹائم کے تقاضوں کے لیے [نائٹ لائٹ](/docs/user-guide/night-light/) دیکھیں۔

`argvus tui-terminal --frame TITLE FOOTER COMMAND [ARG...]` مشترکہ ARGVUS ٹرمینل فریم کے اندر ایک انٹرایکٹو کمانڈ چلاتا ہے۔ Ctrl+E فریم کو بند کرتا ہے۔ اس کا کالر مقامی فوٹر فراہم کرتا ہے۔ اسے ٹرمینل کے اندر استعمال کریں، یا `argvus tui-terminal --term kitty -- ...` کو بھیجی گئی کمانڈ کے طور پر۔`argvus tui-terminal --term kitty --profile ai-agents` پوشیدہ ٹیبز اور صرف کلپ بورڈ شارٹ کٹس کے ساتھ ایک سرشار کٹی کنفیگریشن کا استعمال کرتا ہے۔ کال کرنے والا کمانڈ اور ٹائٹل فراہم کرتا ہے۔ ونڈو پلیسمنٹ کلاس `argvus-ai` کے لیے Hyprland اصول کے ذریعے سنبھالی جاتی ہے۔
