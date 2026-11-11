---
title: ARGVUS کنفیگریشن
description: منطقی JSON کنفیگریشن کنٹریکٹ جو ARGVUS استعمال کرتا ہے۔
---

ARGVUS ماڈیولر JSON فائلوں میں `$XDG_CONFIG_HOME/argvus/config/` (عام طور پر `~/.config/argvus/config/`) کے تحت منظم صارف کی ترجیحات رکھتا ہے۔ وہ ARGVUS کے زیر انتظام ترجیحات کے لیے کینونیکل منطقی ماڈل تشکیل دیتے ہیں، اور `argvus-config` وہ واحد جزو ہے جو `data/generated/` کے نیچے فائلیں لکھتا ہے۔ مقامی فائلیں اور تیار کردہ فائلیں مطابقت کے تخمینے یا ایپلیکیشن کے ساتھ مخصوص اوور رائیڈز رہیں۔

## صارف گائیڈ: فوری آغاز

زیادہ تر صارفین کے لیے، ترتیب ARGVUS کنٹرول سینٹر یا اس کمانڈ لائن ٹول کے ذریعے ہوتی ہے:

```sh
# View your configuration location and validate it
argvus config path
argvus config validate

# Get a specific value
argvus config get /appearance/theme
argvus config get /appearance/theme --raw    # Without JSON formatting

# Set a value (must be valid JSON)
argvus config set /appearance/theme '"gruvbox-dark"'

# Apply changes to reload the session
argvus config apply
```

منتظمین اور جدید ترتیب کے لیے، نیچے پڑھنا جاری رکھیں۔

صارف کی جڑ میں صرف کینونیکل دستاویز اور منظم ڈیٹا ہوتا ہے:

```text
~/.config/argvus/
├── config.json
└── data/
```

فائل جوہری طور پر لکھی گئی ہے، صارف کے لیے نجی رکھی گئی ہے، اور data/ درخت کے باہر `.config.lock` کے ذریعے محفوظ ہے جسے پروجیکشن کے دوران تبدیل کیا جا سکتا ہے۔ ایک پچھلا ورژن  `data/` `data/backups/config.json.bak` پر محفوظ ہے۔

## کمانڈ لائن

`argvus config` ذیلی کمانڈ `argvus` پیکیج کے ذریعے عمل میں فراہم کی جاتی ہے:

```sh
argvus config path
argvus config validate
argvus config ensure
argvus config migrate
argvus config status
argvus config explain /layout/gaps_out
argvus config get /appearance/theme --effective --raw
argvus config set /appearance/accent '"#61AFEF"'
argvus config patch appearance-patch.json
argvus config audio volume +5
argvus config audio mute toggle
argvus config apply-theme universe --variant sticky --accent '#EEEEEE' --gtk-mode dark
argvus config apply-theme universe-spaced --variant spaced --accent '#EEEEEE' --gtk-mode dark
argvus config theme-manifest universe-spaced
argvus config unset /appearance/accent
argvus config apply
argvus config rebuild
argvus config project
argvus config project --force
```

`ensure` اسکیما ڈیفالٹس کے ساتھ ایک نئے `config.json` کو تیار کرتا ہے، یا بھرتا ہے
واضح اقدار کو تبدیل کیے بغیر فیلڈز غائب ہیں۔ یہ خراب JSON پر ناکام ہوجاتا ہے۔
`migrate` اقدار کو تبدیل کیے بغیر موجودہ لیگیسی ظاہری حالت کو درآمد کرتا ہے
`config.json` میں پہلے سے موجود ہے۔ `argvus sessionctl` `recover`، `ensure` چلاتا ہے،
`migrate` اور `project` ترتیب میں تبدیلی کے بعد دوبارہ لوڈ کرنے سے پہلے:

```sh
argvus sessionctl apply-config
```

`apply` اور `rebuild` سے ہر پروجیکشن کو دوبارہ بنانے کے عرفی نام ہیں۔
`config.json`; `project` تاریخی ہجے ہے اور `--force` کو قبول کرتا ہے۔
`apply` `argvus sessionctl` کو پوسٹ کمٹ رن ٹائم ری لوڈ بھی سونپتا ہے۔
تیاری یا الگ تھلگ ٹیسٹ کے لیے `ARGVUS_NO_RUNTIME=1` سیٹ کریں۔
جنریشن اسٹیجنگ میں ہوتی ہے اور `data/` درخت کو تبدیل کیا جاتا ہے۔
پروجیکٹر کامیاب. لہذا `data/generated/` کو حذف کر کے چل رہا ہے۔
`argvus config apply` جزوی شائع کیے بغیر دوبارہ پیدا کرنے کے قابل دوبارہ تعمیر ہے
ہائپر لینڈ یا ٹاسک بار کے آؤٹ پٹس کی ترتیب۔ حاصل کردہ کمانڈ ریکارڈز
سیکشن ہیشز اور رن ٹائم کا فیصلہ
`$XDG_STATE_HOME/argvus/config-projection.json` (عام طور پر
`~/.local/state/argvus/config-projection.json`)۔ نہ ظاہر اور نہ ہی
پیدا شدہ فائلیں سچائی کے ذرائع ہیں۔ `apply` پوسٹ کمٹ دوبارہ لوڈ کرتا ہے۔
`argvus sessionctl` کے ذریعے؛ `project` مطابقت کے لیے صرف پروجیکشن رہتا ہے۔

ہر مستقل اتپریورتن ایک ہی قاعدہ کا استعمال کرتا ہے: اگر کینونیکل دستاویز تبدیل ہو جائے،
CLI کمٹ کے بعد ایک مرکزی `argvus sessionctl reload` طلب کرتا ہے۔ دی
دوبارہ لوڈ کا نتیجہ `data/internal/last-reload.json` میں ریکارڈ کیا گیا ہے۔

`argvus sessionctl reload` اس لیے ایک *لاگو* مرحلہ ہے، پروجیکشن مرحلہ نہیں۔
جب آنے والا مینی فیسٹ کام کو زیر التواء رپورٹ کرتا ہے تو یہ ہدف کو بحال کرتا ہے۔
پلان کے ذریعہ بیان کردہ صارفین اور `argvus config project` کو دوبارہ کال نہیں کرتے ہیں۔
دوبارہ پروجیکٹ کرنے سے ایک جیسے درخت کو دوبارہ بنایا جائے گا اور اتپریورتن کے منصوبے کو رد کر دیا جائے گا۔
صرف پیدا. یہ ہجرت اور پروجیکشن کی طرف واپس آتا ہے صرف اس صورت میں جب کوئی عزم نہ ہو۔
منصوبہ موجود ہے، مثال کے طور پر سیشن کے آغاز کے دوران۔

### اہداف کو دوبارہ لوڈ کریں۔

`argvus config session-plan` کے اہداف کے ساتھ دوبارہ لوڈ کا جواب دیتا ہے۔
سیکشنز جو بدل گئے ہیں، اور `argvus sessionctl` صرف وہی چلاتے ہیں۔ چھڑکاؤ
دوبارہ لوڈ کا احاطہ صرف اس وقت ہوتا ہے جب کوئی ہدف کسی سطح کو دوبارہ بناتا ہے: ایک وے بار بار یا
کنٹرول پینل اسکرین پر پیچھے ہٹنا ہر ونڈو کو شفٹ کرتا ہے۔ باقی سب کچھ
جگہ پر دوبارہ لوڈ کرتا ہے۔

| تبدیلی | اہداف | سپلیش |
| --- | --- | --- |
| تھیم، ہائی لائٹ رنگ یا جی ٹی کے موڈ | `hyprland`, `consumers`, `wallpaper`, `polkit` | نہیں |
| صرف وال پیپر | `wallpaper` (hyprpaper IPC؛ سروس صرف swaybg کے لیے دوبارہ شروع ہوتی ہے) | نہیں |
| `effects` (دھندلا، متحرک تصاویر، شفافیت، سایہ) | `hyprland`, `consumers` | نہیں || `layout` of windows (خالی، بارڈرز) | `hyprland` | نہیں |
| `layout.taskbar` (پوزیشن، مارجن) | `hyprland`, `taskbar`, `widget_telemetry` | ہاں |
| `fonts` | `consumers` | نہیں |
| `control_panel` (سوئچ، کارڈز) | `control_panel` | ہاں |
| `default_apps`, `keyboard_shortcuts`, `hyprland`, `displays` | `hyprland` | نہیں |
| `power` | `hypridle` | نہیں |

`consumers` `argvus config theme-runtime consumers` چلاتا ہے (بار اسٹائل شیٹس،
آئی پی سی پر کنٹرول پینل، ہائپر لینڈ کے رنگ، ڈنسٹ کی تھیم ری لوڈ، بی ٹاپ اور
کٹی)۔ ڈنسٹ اپنے تھیم کو دوبارہ لوڈ کرنے کو موخر کرتا ہے جب کہ ڈسٹرب نہ ہو فعال ہو۔
`wallpaper` `argvus config theme-runtime wallpaper` چلاتا ہے، جو ظاہر کرتا ہے
مؤثر `/appearance/wallpaper`۔ `layout.taskbar` ہیش رکھی گئی ہے۔
پروجیکشن مینی فیسٹ میں ونڈو لے آؤٹ کے علاوہ، تو ایک فرق تبدیلی کرتا ہے۔
سلاخوں کو دوبارہ شروع نہ کریں. فال بیک `full` پلان، اس وقت استعمال ہوتا ہے جب کوئی کمٹڈ پلان نہ ہو۔
موجود ہے، سپلیش کے نیچے ہر سطح کو دوبارہ شروع کرتا ہے۔

ہر موثر ترتیب میں ایک آپریشنل جنریشن ہوتی ہے۔ اس میں ذخیرہ کیا جاتا ہے۔
`data/internal/generation.json` اور پروجیکشن مینی فیسٹ کے تحت
`$XDG_STATE_HOME/argvus/config-projection.json`; یہ ڈاٹ فائلز کا حصہ نہیں ہے۔
ترتیب غیر تبدیل شدہ موثر ترتیب کو دوبارہ لاگو کرنے سے محفوظ رہتا ہے۔
نسل `status` رپورٹ کرتا ہے کہ آیا مینی فیسٹ ہیش، جنریشن میٹا ڈیٹا اور
تمام تیار کردہ فائلیں متفق ہیں؛ `apply` ایک باسی یا نامکمل پروجیکشن کی مرمت کرتا ہے۔

`explain /path` حل کرنے والے کو براہ راست پڑھتا ہے اور موثر قدر پرنٹ کرتا ہے،
ماخذ، تھیم اور مختلف قسم۔ یہ تیار کردہ فائلوں کو پارس نہیں کرتا ہے اور نہ ہی کوئی سیکنڈ بناتا ہے۔
ترجیح کا اصول

تھیم اور ایفیکٹ پروجیکشن خود `argvus-config` میں لاگو ہوتے ہیں۔
تھیم ٹرانزیکشن (`argvus theme-switch`، جو `theme-apply` چلاتا ہے)، `argvus config accent` اور `argvus effects-toggle` پروجیکٹ نہیں کرتے ہیں:
وہ اس CLI کے ذریعے تصدیق کرتے ہیں، لاک کرتے ہیں، کمٹ کرتے ہیں اور پھر
بیرونی صارفین کو جوڑیں جو `config.json` نہیں پڑھ سکتے ہیں۔ `argvus effects-toggle`
ایک پتلا مندوب ہے جس کی اپنی کوئی پیداوار نہیں ہے۔ ظاہری مددگار کبھی نہیں ہوتے ہیں۔
پروجیکٹر کے ذریعہ بلایا جاتا ہے، اور انہیں `data/generated/` کے نیچے کبھی نہیں لکھنا چاہئے۔

### پروجیکشن سطحیں۔

`project_effective` ہر ARGVUS کی ملکیت والے صارف کو ایک اسٹیجنگ ٹری میں پیش کرتا ہے۔
اشاعت سے پہلے:

| سطح | آؤٹ پٹ |
| --- | --- |
| ہائپرلینڈ | `generated/hypr/`، بشمول تھیم پروفائلز، `spaces-effective.conf`، `borders-effective.conf`، `hypridle.conf` اور `monitors.lua` |
| لے آؤٹ/اثرات | `generated/effects/`, `generated/spaces.conf`, `generated/borders.conf` |
| Waybar | `generated/waybar/`، بشمول `argvus-taskbar.css` اور `argvus-widget-telemetry.{jsonc,css}` |
| ویجیٹ ٹیلی میٹری تھیمز | `generated/widget-telemetry/` اور `generated/waybar/widget-telemetry-themes/` |
| QuickShell | `generated/quickshell/` |
| اطلاعات | `generated/notifications/` اور `dunstrc` |
| لاک اسکرین | `generated/hypr/hyprlock.conf` اور Hyptoolkit تھیم پروفائلز |
| جی ٹی کے | `generated/gtk/` |
| یازی | `generated/yazi/` |
| سپر فائل | `generated/superfile/` |
| ٹرمینل اور Qt پروفائلز | `generated/terminal/`, `generated/qt6ct/` |
| فونٹس | وے بار اسٹائل شیٹس میں `generated/fonts.conf` پلس مینیجڈ فونٹ بلاکس |
| ہٹنے کے قابل آلات | `generated/removable-devices/theme.css` |
| وال پیپر | `generated/hypr/hyprpaper.conf` اور `data/hypr/hyprpaper.conf` |
| کنٹرول پینل اور ڈیفالٹس | `data/control-panel/`, `data/control-center/defaults.json` |
| کیلنڈر، ان پٹ، کی بورڈ، پاور | کیلنڈر TOML، ان پٹ فریگمنٹ، کی بورڈ کی حالت، جاگتے رہیں مارکر |

تھیم کی پرت بھی پیدا شدہ جڑ کے باہر عکس بند کی گئی ہے:
`data/waybar/argvus-taskbar.{jsonc,css}`،
`data/waybar/argvus-widget-telemetry.{jsonc,css}`, `data/foot/`،
`data/qt6ct/`، `data/hypr/hyprtoolkit.conf` اور
`data/hypr/application-style.conf`۔ وہ کاپیاں پیکڈ سے بدل دی جاتی ہیں۔
ڈیفالٹس جب ظاہری حصے میں تبدیلی آتی ہے، اور پروجیکٹر پھر لکھتا ہے۔
صرف اس کے محدود کردہ انتظام شدہ بلاکس - وے بار اسٹائل شیٹس میں فونٹ بلاک اور
`ARGVUS_TELEMETRY_*` ٹیلی میٹری پروفائل میں بلاک کرتا ہے - لہذا باہر متن
مارکر تھیم سوئچ سے زندہ نہیں رہتے ہیں۔ `fonts.targets` اور
`control_panel.widget_telemetry_blocks` کیننیکل رہے: کنٹرول سینٹر۔ﺎﺘﮭﮑﻟ ﮟﯿﮩﻧ ﯽﮭﺒﮐ ﮟﯿﻠﺋﺎﻓ ﮧﯾ ﺩﻮﺧ ﺭﻭﺍ ﮯﮨ ﺎﺘﮭﮐﺭ ﺭﺍﺮﻗﺮﺑ ﮯﻌﯾﺭﺫ ﮯﮐ `argvus-config` 

                                        ﮯﮨ ﮧﺟﻭﺮﻣ ﺏﺎﺨﺘﻧﺍ ﺎﮐ ﮎﻼ﻿ﺑ ﯼﺮﭩﯿﻣ ﯽﻠﯿﭨ ﭧﯿﺟﻭ
`/control_panel/widget_telemetry_blocks`، ﮧﻠﺴﻠﺳ ﮏﯾﺍ ﺎﮐ ﮞﻮﻣﺎﻧ ﮎﻼ﻿ﺑ ﺕﺎﺳ
(`system`, `cpu_gpu`, `memory`, `storage`, `processes`, `network`, `keys`). The
                             ﮯﮨ ﮯﯿﻟ ﮯﮐ ﮯﻨﮬﮍﭘ ﻑﺮﺻ ﻞﺋﺎﻓ `$XDG_STATE_HOME/argvus/widget-telemetry-blocks` ﯽﺘﺛﺍﺭﻭ
مائیگریشن فال بیک اور کبھی نہیں لکھا جاتا۔ `argvus widget-telemetry-toggle blocks
set` and ` تمام` کو ایک ہی ایٹمک رائٹ میں کیننیکل صف کو مسدود کرتا ہے، اور
`apply-state` فرض کرتا ہے کہ اس کے کالر کے ذریعہ انتخاب کو پہلے ہی برقرار رکھا گیا تھا۔

Hyprland وقفہ کاری اور سرحدی ترجیحات ایک ہی حد کی پیروی کرتے ہیں۔ دیargvus config spaces اور argvus config بارڈرز کمانڈز (اور ان کی تاریخیمطابقت ریپرز) اپ ڈیٹ /لے آؤٹ؛ زنگ پیدا ہوتا ہے۔
data/generated/spaces-effective.conf اور data/generated/borders-effective.conf
اور توثیق اور لائیو hyprctl درخواست کا مالک ہے۔ خالی جگہیں اور سرحدیں ہیں۔
ہجرت صرف ریاست ہے اور کبھی بھی سچائی کا ذریعہ نہیں ہے۔

ایکسنٹ اور جی ٹی کے موڈ اسٹیٹ ایک ہی رسٹ باؤنڈری استعمال کرتے ہیں۔ `argvus-config استعمال کریں۔
لہجہ COLOR` or `argvus config accent --theme-default` لہجے کی ملکیت کے لیے،
اور GTK وضع کے لیے `argvus config mode toggle|set dark|light`۔ تاریخی
لہجہ اور موڈ اسکرپٹ مطابقت کے ریپرز ہیں۔ `.accent-color`،
`.accent-custom` اور `.gtk-mode` صرف منتقلی/ پروجیکشن فائلیں ہیں۔

منطقی ARGVUS راستے بھی `argvus config paths` کے ذریعے حل کیے جاتے ہیں۔ میراث
`paths.sh` فائل صرف مطابقت کے افعال کو ظاہر کرتی ہے۔ مقامی اوور رائیڈز، صارفڈیٹا، جنریٹڈ ڈیٹا اور سسٹم فال بیک کو Rust پاتھ ماڈل کے ذریعے حل کیا جاتا ہے۔`audio` پہلے سے طے شدہ آؤٹ پٹ سنک کو تبدیل کرتا ہے اور نتیجہ کو ایک قدم میں `/audio` میں ریکارڈ کرتا ہے: `argvus config audio volume <N|+N|-N> [--limit PERCENT]` اور `argvus config audio mute <on|off|toggle>`۔ یہ کنفیگریشن کو دوبارہ لوڈ کیے بغیر اسٹور لکھتا ہے، کیونکہ پائپ وائر کی قدر پہلے سے موجود ہے۔ Waybar والیوم ماڈیول، Hyprland میڈیا کیز اور کنٹرول پینل اسے استعمال کرتے ہیں، لہذا `argvus config project`، جو `/audio` کو سنک پر پروجیکٹ کرتا ہے، کبھی بھی پرانے والیوم کو بحال نہیں کرتا ہے۔ 100% سے زیادہ والیوم (ہائپرلینڈ کیز 150% تک اجازت دیتی ہیں) 100 کے طور پر محفوظ کی جاتی ہے۔

`patch` ایک JSON آبجیکٹ کا اطلاق کرتا ہے جس کی چابیاں JSON پوائنٹر ہیں ایک لاک میں
پڑھنے میں ترمیم کریں لکھنے کا عمل۔ `apply-theme` مساوی جوہری انجام دیتا ہے۔
تھیم ٹرانزیکشن اور سے منظم ظاہری سیکشنز کی جگہ لے لیتا ہے۔
تھیم کا اختیاری ظاہری پروفائل یا پیکڈ ڈیفالٹس۔
تبدیل کرنے والی کمانڈز ایک `NextUserConfig` بناتے ہیں، `EffectiveConfig` کو حل کرتے ہیں، اسٹیج
تمام پروجیکٹر، اور تب ہی کیننیکل JSON شائع کریں۔ ایک آپریشنل
`.config-transaction.json` مارکر `argvus config recover` کو دوبارہ پروجیکٹ کرنے کی اجازت دیتا ہے
ڈیٹا اور JSON اشاعت کے درمیان کریش ہونے کے بعد پچھلی ترتیب۔
تھیم ڈیفالٹس کا اعلان ہر تھیم کے مینی فیسٹ سے کیا جاتا ہے۔
(`themes/<theme>/theme.toml`): پہلے سے طے شدہ لہجہ (`palette.accent`)، GTK رنگ سکیم
اور وال پیپر. CSS، Lua، Rasi اور Qt فائلیں اس سے تھیم کی تعمیر میں پیش کی جاتی ہیں۔

لائٹ/ڈارک سمیت تھیم کو تبدیل کرنا، ہائی لائٹ کلر کو تھیم کے ڈیفالٹ پر سیٹ کرتا ہے،
یہاں تک کہ جب صارف اپنی مرضی کے رنگ کا انتخاب کرتا ہے: حسب ضرورت رنگ صرف اگلے تھیم تک رکھا جاتا ہے۔
تبدیلی تھیم میں تبدیلی کے بعد اسے دوبارہ سیٹ کرنے کے لیے `argvus config accent` استعمال کریں۔ لگانا a
`argvus config theme-apply` کے ساتھ تھیم `/appearance/accent` اور `/appearance/gtk_mode` لکھتا ہے
مینی فیسٹ سے، اور `argvus config accent` `accent_custom` کے ساتھ منتخب رنگ لکھتا ہے
ایک کمانڈ میں سیٹ کریں۔

`apply-theme` متغیر کو واضح طور پر `--variant sticky|spaced` اور
درخواست کردہ نام کو مینی فیسٹ کے ذریعے حل کرتا ہے، لہذا درخواست کبھی نہیں ہوسکتی ہے۔دوسرے قسم کے اثاثوں کو حل کریں۔ `theme-manifest THEME` وہی ہے۔
ریزولوشن صرف پڑھنے کے لیے بے نقاب: یہ مینی فیسٹ اندراج کو کے ساتھ پرنٹ کرتا ہے۔
`asset_id` اور `layout_variant` اس نے حل کیا، جس کی ضرورت کال کرنے والے کو
ایڈریس پیکڈ اثاثوں. چسپاں غیر لاحقہ آئی ڈی ہے اور صرف اسپیسڈ میں a ہوتا ہے۔`-spaced` لاحقہ، تو `dracula-sticky` اور `argvus-dark-dracula-sticky` ہیں
قبول کیا گیا اور `dracula` کو معمول پر لایا گیا۔

`--raw` کے ساتھ، سٹرنگ کی قدریں JSON کوٹس کے بغیر پرنٹ کی جاتی ہیں، جبکہ آبجیکٹ اور arrays JSON ویلیوز ہی رہتے ہیں۔ تھیم اور لے آؤٹ ڈاٹ فائلیں اب پروجیکشن مددگار کے ذریعہ نہیں لکھی جاتی ہیں۔ `config.json` ان ترجیحات کا واحد مستقل ذریعہ ہے۔ اگر پروجیکشن ناکام ہوجاتا ہے تو، ڈیسک ٹاپ سطحوں کو دوبارہ شروع کرنے سے پہلے سیشن کو دوبارہ لوڈ کرنا بند ہوجاتا ہے لہذا ایک پرانے جزوی تھیم کو موجودہ کے طور پر پیش نہیں کیا جاتا ہے۔

## برآمد اور درآمد

پروفائل کے تبادلے کے لیے وہی منطقی ماڈل استعمال کیا جاتا ہے:

```sh
argvus config export --scope appearance --output appearance.json
argvus config import --scope appearance appearance.json
```

مکمل منظم ڈیسک ٹاپ ماڈل کے لیے `--scope desktop` استعمال کریں۔ غائب فیلڈز ARGVUS ڈیفالٹس استعمال کرتے ہیں۔ پیدا شدہ آؤٹ پٹ کو براہ راست ترمیم نہیں کیا جانا چاہئے.Control Center theme export and import exchange a theme and its visual profile: the archive holds a schema 2 theme directory (`<id>/theme.toml`, optional `appearance.json` and `effects.conf`, `overrides/`, the wallpaper) that installs under `$XDG_DATA_HOME/argvus/themes/`. The profile holds appearance,ترتیب جیومیٹری، بصری اثرات، فونٹس اور پینل پریزنٹیشن۔ آڈیو، کی بورڈ، نیٹ ورک اور دیگر غیر بصری ترجیحات تھیمز سے باہر رہتی ہیں۔ ظاہری شکل کی مکمل ترتیب کو منتقل کرنے کے لیے اوپر `appearance` دائرہ کار استعمال کریں، اور مکمل ترتیب کے لیے `desktop` دائرہ کاربیک اپThe current schema groups preferences under `appearance`, `layout`, `effects`, `fonts`, `control_panel`, `default_apps`, `keyboard_shortcuts`, `hyprland`, `displays`, `power`, `session`, `calendar`, and `removable_devices`. `default_apps` is the only persistent source for preferred applications,اور `keyboard_shortcuts` مستحکم انگلش، سٹرنگ ویلیوز کے ساتھ لوکل سے آزاد کیز یا `null` کو غیر فعال بائنڈنگز کے لیے اسٹور کرتا ہے۔جب کنٹرول سینٹر ظاہری شکل کا صفحہ کھولتا ہے اور `argvus-config` کے ذریعے تبدیلیاں لکھتا ہے تو کیننیکل ظاہری اقدار کو پڑھتا ہے۔ موجودہ مقامی ریاست کی فائلیں ظاہری اسکرپٹس اور پرانی تنصیبات کے تخمینے بنی ہوئی ہیں۔کنٹرول پینل کارڈ آرڈر اور مرئیت کے لیے `control_panel.cards` اور پینل ماسٹر سوئچ کے لیے `control_panel.enabled` استعمال کرتا ہے۔ اس کی `control-panel/cards.json` اور `state/control-panel` فائلوں کو مطابقت کے تخمینوں کے طور پر رکھا گیا ہے۔ اس لیے کنٹرول پینل میں کی گئی تبدیلیاں `desktop` برآمدی دائرہ کار کا حصہ ہیں۔فونٹ کی ترجیحات `fonts.targets` فیملی، سٹائل، اور ٹاسک بار کے سائز، ٹیلی میٹری، کنٹرول پینل، سسٹم، ایپلی کیشنز، ٹرمینل، اور براؤزر کے اہداف کے لیے استعمال کرتی ہیں۔ `fonts.rendering` antialiasing، hinting، subpixel order، اور DPI ترجیحات کو اسٹور کرتا ہے۔ `fonts.conf`، فونٹ کنفگ رول، ڈیسک ٹاپ سیٹنگز اور Waybar اسٹائل شیٹس کے اندر مینیجڈ فونٹ بلاکس پروجیکشنز ہیں: کنٹرول سینٹر `/fonts/targets/*` کے ذریعے `argvus-config` کو برقرار رکھتا ہے، اور پروجیکٹر نے صرف اپنے بلاک کو دوبارہ لکھا ہے۔ہائپرلینڈ کی ترجیحات شارٹ کٹس کے لیے `keyboard_shortcuts`، ماؤس اور ٹچ پیڈ سیٹنگز کے لیے `hyprland.input`، اور XKB لے آؤٹ، ویرینٹ، اور اختیارات کے لیے `hyprland.keyboard` استعمال کرتی ہیں۔ ایک مینی فیسٹ بائنڈنگ ID ایک مستحکم `config_key` (مثال کے طور پر `window.close` → `close_window` اور `window.drag_mouse` → `drag_window__floating_window_only`) سے نقشہ بناتا ہے۔ ڈسپلے کی ترجیحات `displays.monitors`، `displays.workspaces`، اور `displays.primary_monitor` استعمال کرتی ہیں۔ مقامی TOML فائلیں اور `data/generated/` کے تحت تیار کردہ Lua فائلیں Hyprland کے ذریعہ استعمال ہونے والے تخمینے ہی رہیں گی۔ورچوئل مشینوں اور کم پاور سیشنز پر بھی ہر `effects` فعال (اینیمیشن، بلر، ویجیٹ ٹیلی میٹری اور ہر شفافیت کی سطح) پر ڈیفالٹ سوئچ کرتا ہے۔ ونڈو شیڈو (`shadow_enabled`) اور wobble (`wobble_enabled`) ڈیفالٹ آف ہے۔ اثر کی ترجیحات `effects` اینیمیشن ماسٹر سوئچ کے لیے استعمال کرتی ہیں (`animations`، `hyprland.lua` پر متوقع `hypr/state.lua` سے لاگو کیا جاتا ہے، جو کہ اثرات سے آزاد ہے، `blur_global_enabled` plus the `transparency_<surface>_enabled` of `taskbar`, `control-panel`, `widget-telemetry`, `terminal` and `launchers`, and reads as enabled when any of them is; `argvus effects-toggle animations|effects` switch them), the global blur switch (`blur_global_enabled`), the Hyprland blur parameters (`blur_size`, `blur_passes`, `blur_brightness`, `blur_noise`, `blur_contrast`, `blur_vibrancy`, `blur_vibrancy_darkness`), the window shadow(`shadow_enabled`، `shadow_sharp`, `shadow_color`, `shadow_range`, `shadow_render_power`, `shadow_offset_x`, `shadow_offset_y`, `shadow_scale`), the Animations values (`animation_<leaf>_speed`, `_curve`, `_style`, `_popin` and `_enabled` for each animation, plus `animation_bezier_x1` to `animation_bezier_y2` and `animation_spring_mass`, `animation_spring_stiffness` and `animation_spring_dampening` for the custom curves) and per-surface transparency values. Effects areتھیم کی ملکیت، متغیر کی ملکیت نہیں: چسپاں اور فاصلہ ایک ہی `<theme>.conf` اثرات کے پروجیکشن کا استعمال کرتے ہیں جبکہ ان کے لے آؤٹ پروجیکشن مختلف ہوتے ہیں۔ سطحوں میں فی الحال `taskbar`، `control-panel`، اور `widget-telemetry` شامل ہیں۔ شفافیت کی قدریں `0` سے `100` تک فیصد ہیں (کنٹرول سینٹر اور لانچر کی سطحیں `15` پر ڈیفالٹ ہیں، باقی `50` پر)، دھندلا پیرامیٹر کے اندر موجود نمبرز ہیں۔حدودمیں اعلان کیا `schema.json`, and enabled states are booleans. Animations values are validated by `argvus-config` too: speeds are numbers from `1` to `10` (decimals allowed), pop-in percentages are whole numbers from `1` to `100`, curves must beاینیمیشن پیج کے نام، اسٹائلز انیمیشن پیج کے نام ہونے چاہئیں اور اینیمیشن کے اپنے لیف کے ذریعہ قبول کیے گئے ہوں (مثال کے طور پر `popin` `animation_workspace_in_style` کے لیے رد کر دیا گیا ہے، `gnomed` صرف ونڈو کھولنے کے لیے قبول کیا جاتا ہے، اور ونڈو کھولنے کے لیے انکار کر دیا جاتا ہے۔کھڑکیاور پرت کے پتے)، Bézier Y پوائنٹس `-1` اور `2` کے درمیان رہتے ہیں، حد Hyprland 0.56.2 پارس، اور دیگر کریو پیرامیٹرز اپنی دستاویزی حدود میں رہتے ہیں۔ `layers` اور `switching` کے پتے اب موجود نہیں ہیں، لہذا کنفیگرز کو پہلے محفوظ کر لیا گیاان کے ہٹانے سے `animation_layers_enabled`، `animation_layers_speed`، `animation_layers_curve` اور `animation_switching_*` کیز لوڈ ہونے پر ضائع ہو جاتی ہیں۔ موجودہ سیٹ سے باہر کے اسٹائلز کو گرا دیا گیا ہے اور `-1` سے `2` کے باہر Bézier Y پوائنٹس کو بند کر دیا گیا ہے، لہذا پیکڈ ڈیفالٹس دوبارہ لاگو ہوتے ہیںجو بھی تھاہٹا دیاپاور ترجیحات `power.screen_off_minutes`، `power.lock_minutes`، `power.keep_awake`، اور ڈھکن/پاور بٹن کی پالیسیوں کا استعمال کرتی ہیں۔ متوقع شکلیں `data/generated/hypr/hypridle.conf` اور مارکر `data/power/keep-awake` ہیں؛ `.keep-awake` صرف ایک منتقلی ان پٹ ہے۔ `session.language` ریٹائرڈ ہے: موجودہ اقدار کو نظر انداز کر دیا جاتا ہے، نئی تحریریں مسترد کر دی جاتی ہیں، اور زبان کی مطابقت کی کوئی فائل پیش نہیں کی جاتی ہے۔ اس کے بجائے صارف locale.conf میں `LANG` سیٹ کریں۔ کیلنڈر کی ترجیحات  `locale.conf` `calendar` استعمال کرتی ہیں اور ٹاسک بار-کیلنڈر صارف TOML کو پیش کی جاتی ہیں۔ ہٹنے کے قابل ڈیوائس کی ترجیحات `removable_devices` استعمال کرتی ہیں۔ ہٹانے کے قابل ڈیوائسز ایپلیکیشن اس سیکشن کو پیکیجڈ ڈیفالٹس پر صارف کو اوور رائڈ کے طور پر مانتی ہے۔`migrate` imports the old root files and sections once, preserving an existing canonical value. Legacy `defaults.json`, `/defaults`, `/hyprland/keybindings`, `keybindings.toml`, dot-files, `generated/` and component directories are moved below `data/`; repeated reloads do not recreate the old root layout. Selecting the current theme again is idempotent: if canonical state and projections already agree, the projectionپلان میں کوئی تبدیل شدہ سیکشن کی اطلاع نہیں ہے اور سیشن سپلیش کو چھوڑ دیتا ہے اور سروس دوبارہ شروع ہو جاتی ہے۔

لاک اسکرین کی ظاہری شکل کا اندازہ زنگ کے ذریعہ کینونیکل تھیم، لہجہ،
اثرات، وال پیپر اور فونٹ کی حالت  0  argvus config project  argvus-hypridle.service  argvus sessionctl  data/hypr/hypridle.conf  argvus config project  config.json  `/power`  `config.json`  `argvus config project`  `data/hypr/hypridle.conf`  `argvus sessionctl`  `argvus-hypridle.service`  `argvus config project`  `0` `data/hypr/hyprlock.conf` میں۔ تاریخی
`hyprlock-theme.sh` انٹری پوائنٹ صرف مطابقت رکھتا ہے اور اس کے نمائندے
`argvus config lock apply`; `--invalidate` صرف اخذ کردہ دھندلا پن کو باطل کرتا ہے
وال پیپر کیش.

پاور ٹائم آؤٹ تبدیلیاں `/power/lock_minutes` اور استعمال کرتی ہیں۔
`/power/screen_off_minutes` ان کی سچائی کے ماخذ کے طور پر۔ `argvus config project`
موجودہ Hypridle renderer کو دوبارہ استعمال کرتا ہے، اور `idle-timeout.sh` انٹری پوائنٹ ہے
صرف ایک مطابقت اڈاپٹر؛ سروس دوبارہ شروع کرنے کے لئے تفویض رہتا ہے۔
`argvus sessionctl`۔

نوٹیفکیشن کی ظاہری شکل کیننیکل تھیم، لہجہ اور سے ماخوذ ہے۔
`data/notifications/dunstrc` میں اثرات۔ تاریخی اطلاعات
`theme.sh` فائل صرف مطابقت کے افعال کو ظاہر کرتی ہے جو کال کرتے ہیں۔
`argvus config notifications`; یہ ڈنسٹ کنفیگریشن کو پارس یا تبدیل نہیں کرتا ہے۔

ہارڈ ویئر پر مبنی اڈاپٹر جیسے ڈسپلے دریافت اور بلوٹوتھ کنٹرول ہیں۔اب زنگ ڈسپلے ڈومین کے ذریعہ درآمد کیا گیا ہے: `nwg-displays` ایک بیرونی درآمد کنندہ ہے،جبکہ `/displays` اور `data/generated/hypr/monitors.lua` کیننیکل اور
ماخوذ ریاست بالترتیب. ڈسپلے ٹولز لے آؤٹ کو تبدیل کرتے ہیں۔
`argvus config monitor-set`, `monitor-primary`, `monitor-workspaces` اور `monitor-remove`; ڈسپلے کمانڈز `argvus config display status|list-modes|set|apply|apply-nwg|migrate-nwg|nwg-import|nwg-apply` ہیں۔ Hyprland فائل کا تخمینہ `/displays` سے لگایا گیا ہے۔ کوئی شیل اسکرپٹ اسے مزید نہیں لکھتا ہے۔ وال پیپر کا انتخاب، ٹاسک بار یوٹیلیٹی گروپ موڈ،
اور ڈیفالٹ ایپ تلاش `argvus-config` APIs بھی استعمال کرتی ہے۔ ان کا تاریخی شیل
انٹری پوائنٹس UI یا مطابقت کے ریپرز ہیں۔ جاگتے رہنا ایک مستقل مزاج ہے۔
`/power/keep_awake` ترجیح `data/power/keep-awake` مارکر پر پیش کی گئی،
جب کہ lock-DPMS سیشن رن ٹائم اسٹیٹ کے تحت ہے۔
`$XDG_CACHE_HOME/argvus/lock-dpms`۔ کی بورڈ لے آؤٹ اب Rust کی ملکیت ہے۔
`argvus-keyboard-layout` مددگار: ترتیب شدہ آرڈر تیار کردہ سے آتا ہے
ہائپرلینڈ ان پٹ پروجیکشن یا لوکیل فال بیک، جبکہ فعال لے آؤٹ باقی ہے۔
ہائپرلینڈ سے سیشن رن ٹائم حالت دریافت ہوئی۔ سائیکل انٹری پوائنٹ اور
کی بورڈ لے آؤٹ ڈیمون صرف مطابقت کے ریپرز ہیں۔ صارف کی خدمت چلتی ہے۔
زنگ ڈیمون براہ راست اور `argvus sessionctl keyboard-layout cycle` ہے۔
کال کرنے والوں کے لیے مستحکم کمانڈ۔

بلوٹوتھ رن ٹائم حالت بیرونی اڈاپٹر کی حد کی پیروی کرتی ہے لیکن حصہ نہیں ہے۔
`argvus-config` کا: `argvus bluetooth` ٹائپ شدہ پارسنگ کا مالک ہے اور
`bluetoothctl` کے ارد گرد توثیق، باؤنڈڈ دوبارہ کوششیں، سسٹم سروس چیک اور
RFKill کی حیثیت۔ `argvus bluetoothctl` انٹری پوائنٹ ہے۔ اڈاپٹر پاور، جوڑی والے آلات اور منسلک آلات
کیننیکل JSON ریاست کے بجائے BlueZ رن ٹائم اسٹیٹ رہیں۔ ہارڈ ویئر لائیو
بلوٹوتھ کی توثیق ماحول پر منحصر ہے۔
## رن ٹائم شناخت اور مفاہمت

عین مطابق بائنری، تعمیر کا معائنہ کرنے کے لیے `argvus config status --verbose` استعمال کریں۔
شناخت، کنفیگریشن روٹ، جنریٹڈ روٹ، اثاثہ جڑ، مینی فیسٹ سورس اور
فعال نسل. یہ `synchronized`، `stale`، `incomplete`، `legacy` کی رپورٹ کرتا ہے
اور `pending-transaction` پروجیکشن اسٹیٹس۔`argvus config --version` پیکیج ورژن کی اطلاع دیتا ہے، جب سپلائی کیا جائے تو نظر ثانی کی تعمیر کریںتعمیراتی ماحول اور پروفائل کے ذریعے۔ ڈویلپمنٹ ٹیسٹ ان کے انجیکشن لازمی ہیں
بائنری اور اثاثے واضح طور پر؛ انہیں خاموشی سے `argvus config` استعمال نہیں کرنا چاہیے۔
یا `~/.config/argvus`۔ `apply` میراثی مشترکہ شناخت کنندگان کو محفوظ طریقے سے معمول بناتا ہے
جیسا کہ `argvus-dark-spaced` الگ تھیم اور مختلف اقدار میں پہلے
تخمینوں کی تعمیر نو پیکیجڈ اثاثہ کی شناخت اس ریلیز میں مینی فیسٹ میں درج نہیں ہے۔

## نائٹ لائٹ

**ڈیسک ٹاپ اور ان پٹ> ڈسپلے> نائٹ لائٹ** کھولیں یا `argvus control-center displays night-light` چلائیں۔ `argvus config display night-light` کے ذریعے درجہ حرارت، دستی ایکٹیویشن اور فکسڈ لوکل شیڈول ترتیب دیں۔ کمانڈز، حالت اور رن ٹائم کے تقاضوں کے لیے [نائٹ لائٹ](/docs/user-guide/night-light/) دیکھیں۔

### Wobble ترتیبات (Hyprland 0.57+)

`effects.wobble_enabled`، `wobble_stiffness`، `wobble_damping`، `wobble_mass`،
`wobble_intensity` اور `wobble_mesh` پروجیکٹ براہ راست `hypr/state.lua` میں۔
انہیں پڑھنے کے لیے `argvus effects-toggle wobble-settings get` استعمال کریں، یا
`argvus effects-toggle wobble-settings set enabled=enabled stiffness=100 damping=10 mass=1 intensity=1 mesh=8`
محفوظ کرنے اور دوبارہ لوڈ کی درخواست کرنے کے لیے۔ `enabled=disabled` اثر کو بند کر دیتا ہے۔

کنٹرول سینٹر اینیمیشن ڈرافٹ کا اشتراک کرتا ہے: `animations-settings get` بھی
`wobble_*` اندراجات واپس کرتا ہے، اور `animations-settings set` ان مکمل ناموں کو قبول کرتا ہے
اس کے موجودہ حرکت پذیری دلائل کے ساتھ۔ یہ ایک میں پورے مسودے کو برقرار رکھتا ہے۔
توثیق شدہ پیچ اور ایک دوبارہ لوڈ کی درخواست کرتا ہے۔ مثال کے طور پر:

```sh
argvus effects-toggle animations-settings set enabled=enabled wobble_enabled=true wobble_intensity=0 wobble_mesh=8
```

عالمی `enabled` متحرک تصاویر کو کنٹرول کرتا ہے۔ `wobble_enabled` اس کا خود مختار ہے۔
ترجیح عالمی اینیمیشنز کو غیر فعال کرنے سے Lua میں مٹائے بغیر لرزنے کو دبا دیا جاتا ہے۔
وہ ترجیح. غلط اقدار پورے پیچ کو مسترد کرتی ہیں۔ لاپتہ چابیاں موصول
ARGVUS ﮟﯿﮭﮑﯾﺩ ۔ﮯﮨ ﺎﺗﺮﮐ ﭧﻟﺎﻔﯾﮈ ﺮﯿﻐﺑ ﮯﮐ ﮯﻨﻟﺪﺑ ﻮﮐ ﺏﺎﺨﺘﻧﺍ ﮦﺩﻮﺟﻮﻣ [                                                                     ﻞﮑﺷ ﯼﺮﮨﺎﻇ](/docs/user-guide/appearance/#window-wobble)
حد اور ڈیفالٹس کے لیے۔


### صارف کے بنائے ہوئے کی بورڈ شارٹ کٹس

`/keyboard_shortcuts` آفیشل مینی فیسٹ ایکشنز کے لیے اپنی موجودہ سٹرنگ یا نال اقدار کو برقرار رکھتا ہے۔ اپنی مرضی کے عالمی کی بورڈ شارٹ کٹ ایک علیحدہ `/hyprland/custom_shortcuts` آبجیکٹ کا استعمال کرتے ہیں، جو `config/hyprland.json` میں محفوظ ہے۔ اس کی مستحکم `custom_` IDs تفصیل اور مقامی مقامات سے آزاد ہیں۔ ہر ریکارڈ میں `description`، `keys`، `enabled` اور ایک ٹائپ شدہ `action` ہے:

```json
{
  "custom_editor": {
    "description": "Open my editor",
    "keys": "SUPER + CTRL + E",
    "enabled": true,
    "action": { "type": "command", "command": "editor --new-window" }
  },
  "custom_float": {
    "description": "Toggle floating",
    "keys": "SUPER + CTRL + F",
    "enabled": false,
    "action": {
      "type": "dispatcher",
      "dispatcher": "window.float",
      "arguments": { "action": "toggle" }
    }
  }
}
```

مثال `/hyprland/custom_shortcuts` کی قدر ہے، مکمل کنفیگریشن دستاویز نہیں۔ ڈسپیچر کے نام معاون Lua ڈسپیچر فیکٹریوں سے آتے ہیں۔ دلائل ایک اختیاری JSON آبجیکٹ یا `null` ہیں، کبھی بھی کوڈ کا اندازہ نہیں کیا جاتا ہے۔ حسب ضرورت شارٹ کٹس میں کوئی ذیلی نقشہ، ماؤس بائنڈنگ یا جدید جھنڈے نہیں ہوتے ہیں۔ پروجیکشن `data/generated/hypr/custom-shortcuts.lua` کو ایک فرار شدہ ڈیٹا ٹیبل کے طور پر لکھتا ہے، بشمول ایک خالی ٹیبل جب آخری ریکارڈ ہٹا دیا جاتا ہے۔ مکمل پروجیکشن اور `project --only keyboard_shortcuts` دونوں اسے تیار کرتے ہیں۔ Hyprland `hl.bind` کے ساتھ فعال اندراجات کو رجسٹر کرتا ہے۔ اختیاری فائل اور انفرادی رجسٹریشن کی ناکامیاں موجود ہیں اور لاگ ان ہیں۔

کنٹرول سینٹر ایک `argvus config patch` ٹرانزیکشن کا استعمال کرتے ہوئے ریکارڈز تخلیق/ترمیم کرتا ہے اور تصدیق شدہ تنازعات کو غیر فعال کرتا ہے۔ یہ موجودہ کنفگ سروس کے ذریعے درخواست دینے سے پہلے بچاتا ہے۔ ایک ناکام درخواست کامیاب بچت کو کالعدم نہیں کرتی ہے۔ آفیشل ڈیفالٹس کو بحال کرنے سے حسب ضرورت ریکارڈ محفوظ رہتا ہے اور اگر یہ ایک فعال کسٹم شارٹ کٹ سے ٹکرا جاتا ہے تو انکار کر دیا جاتا ہے۔ صارف کی تفصیل تمام لوکلز میں لفظی متن اور موثر تیار کردہ چیٹ شیٹ رہتی ہے۔

## گلوبل ونڈو موڈ

**کنٹرول سینٹر → ڈیسک ٹاپ اور ان پٹ → ہائپرلینڈ → ونڈوز → موڈ** کھولیں اور **ٹائل ونڈوز (ڈیفالٹ)** یا **فلوٹنگ ونڈوز** کو منتخب کریں۔ وہی سلیکٹر ٹاسک بار یوٹیلیٹی گروپ میں ونڈو موڈ آئیکن سے دستیاب ہے۔

ترجیح ہر مانیٹر اور ورک اسپیس پر لاگو ہوتی ہے اور لاگ آؤٹ کے بعد یاد رکھی جاتی ہے۔ فلوٹنگ موجودہ ایپلیکیشن ونڈوز اور نئی کھلی ہوئی ونڈوز کو تبدیل کرتی ہے۔ ٹائل صرف ان ونڈوز کو بحال کرتا ہے جنہیں اس پالیسی نے تبدیل کیا تھا۔ ڈائیلاگ اور دیگر ونڈوز جو پہلے سے تیر رہے تھے تیرتے رہتے ہیں۔ فلوٹنگ کے فعال ہونے کے دوران واضح طور پر پن کی گئی ونڈو بھی اپنے پن کو محفوظ رکھنے کے لیے تیرتی رہتی ہے۔ ٹائل کی ترتیب کو دوبارہ شمار کیا جاتا ہے؛ اس کا بالکل سابقہ ​​تقسیم شدہ درخت بحال نہیں ہوا ہے۔`SUPER + SHIFT + Space` ٹائل میں اپنا معمول کے مطابق فی ونڈو رویہ رکھتا ہے۔ فلوٹنگ میں یہ کھڑکی کو واپس ٹائلنگ میں تبدیل نہیں کرتا ہے۔ پینلز، ٹاسک بار اور اطلاعات پرت شیل سطحیں ہیں اور متاثر نہیں ہوتی ہیں۔ مکمل اسکرین اور زیادہ سے زیادہ اضافہ Hyprland کے نارمل رویے کا استعمال جاری رکھے ہوئے ہے۔

کینونیکل ترجیح $XDG_CONFIG_HOME/argvus/config/hyprland.json میں `/hyprland/window_mode` ہے۔ اس کا تیار کردہ پروجیکشن  `$XDG_CONFIG_HOME/argvus/config/hyprland.json` `data/generated/hypr/window-mode.lua` ہے۔ اس میں ترمیم نہ کریں۔ فی ونڈو بحالی رجسٹری `$XDG_RUNTIME_DIR/argvus` کے تحت رہتی ہے، جس کا دائرہ کمپوزیٹر مثال تک ہوتا ہے، اور کنفیگریشن دوبارہ لوڈ ہونے سے بچ جاتا ہے۔ اسے نئے سیشن میں دوبارہ استعمال نہیں کیا جاتا ہے۔

اگر درخواست زیر التواء ہے یا ناکام ہوجاتی ہے، تو انٹرفیس اسے محفوظ کردہ ترجیح سے ممتاز کرتا ہے۔ دوبارہ کوشش کرنے کے لیے دوبارہ اختیار منتخب کریں۔ کمپوزیٹر کے بغیر، ترجیح اگلے لاگ ان کے لیے محفوظ ہو جاتی ہے۔

```sh
argvus config window-mode get
argvus config window-mode status --json
argvus config window-mode set floating
argvus config window-mode set tile
argvus config window-mode toggle
```

## ونڈو کے بٹن اور ایپلیکیشن بار

فلوٹنگ موڈ عام ایپلیکیشن ونڈوز میں **کم سے کم**، **زیادہ سے زیادہ/بحال کریں**، اور **بند** کے ساتھ ایک ARGVUS ٹائٹل بار کا اضافہ کرتا ہے۔ ٹائٹل بار پر ڈبل کلک کرنے سے ٹوگل زیادہ سے زیادہ ہو جاتا ہے۔ اضافی سجاوٹ ٹائل اور پوری اسکرین میں چھپی ہوئی ہے۔ ایپلی کیشنز جو اپنے کنٹرول کو اپنی طرف متوجہ کرتی ہیں وہ دونوں سلاخوں کو ظاہر کر سکتی ہیں؛ یکساں ARGVUS کنٹرول جان بوجھ کر ہیں۔

مرکزی ٹاسک بار کے مخالف کنارے پر، ہر مانیٹر پر دوسرا Waybar ظاہر ہوتا ہے۔ یہ صرف اس مانیٹر کے موجودہ ورک اسپیس سے تعلق رکھنے والی ونڈوز کی فہرست دیتا ہے، بشمول اس ورک اسپیس سے کم سے کم ونڈوز۔ ایک کھلی خصوصی کام کی جگہ کو فوقیت حاصل ہے۔ پن کی ہوئی کھڑکیاں نظر آتی رہتی ہیں۔ پوشیدہ گروپ ٹیبز اضافی اندراجات نہیں بناتے ہیں، اور ایک کم سے کم گروپ میں ایک ریکوری انٹری ہوتی ہے۔ ایپلیکیشن کے اندر موجود ٹیبز کو الگ سے شمار نہیں کیا جاتا ہے۔ کم سے کم اندراجات اپنی اصل مانیٹر/ ورک اسپیس ایسوسی ایشن کو برقرار رکھتی ہیں۔ اسے چالو کرنے کے لیے ایک غیر فعال ونڈو پر کلک کریں، اسے کم سے کم کرنے کے لیے ایک فعال ونڈو، یا اسے بحال کرنے کے لیے کم سے کم ونڈو پر کلک کریں۔ اس کا سیاق و سباق کا مینو بحال، کم سے کم، زیادہ سے زیادہ/بحال اور بند کرنے کی پیشکش کرتا ہے۔ نامعلوم ایپلیکیشن شبیہیں ونڈو کی علامت پر واپس آتی ہیں۔ کسی اندراج کو چالو کرنا یا بحال کرنا پوائنٹر کی پوزیشن کو محفوظ رکھتا ہے۔ ثانوی بار مؤثر پرائمری CSS (رنگ، ​​فونٹ، بارڈر اور راؤنڈنگ) کا استعمال کرتا ہے، اس کی اونچائی اور مخالف کنارے پر عکس والے مارجن کے ساتھ۔

`SUPER + Alt + Up` مین بار کو اوپر اور ونڈو بار کو نیچے رکھتا ہے۔ `SUPER + Alt + Down` الٹا کرتا ہے۔ کسی سمت کو دہرانے سے پوزیشنیں غیر تبدیل ہوتی ہیں۔ ٹائل میں صرف مرکزی بار ہے۔ ٹائل پر واپس آنا ونڈو بار کو ہٹانے سے پہلے تمام چھوٹی کھڑکیوں کو بحال کرتا ہے، اور پھر اصل ٹائل شدہ/تیرتی پالیسی کو محفوظ رکھتا ہے۔

مائنسائزیشن محفوظ `special:argvus-minimized` ورک اسپیس کا استعمال کرتی ہے۔ اس ورک اسپیس کو دستی طور پر استعمال نہ کریں۔ `argvus windows service` عمل `$XDG_RUNTIME_DIR/argvus` کے تحت کمپوزیٹر-انسٹانس-مخصوص رن ٹائم رجسٹری میں شناخت، اصل ورک اسپیس/مانیٹر، جیومیٹری، فل سکرین/زیادہ سے زیادہ اور پن کی حالت کو ریکارڈ کرتا ہے۔ ریکارڈز سروس کے دوبارہ شروع ہونے یا تشکیل دوبارہ لوڈ کرنے سے بچ جاتے ہیں، لیکن کمپوزیٹر سیشنز کو عبور نہیں کرتے ہیں۔ ونڈو کو بند کرنے سے اس کا ریکارڈ ختم ہوجاتا ہے۔ اگر ایک مانیٹر غائب ہو جاتا ہے، بحالی ایک دستیاب آؤٹ پٹ کا استعمال کرتا ہے. ایک گمشدہ یا ناکام ریکوری بار نئی مائنسائزیشن کو روکتا ہے۔ ناکام بحالی ریکارڈ اور ریکوری بار کو برقرار رکھتی ہے۔

سیشن `argvus-windows.service` اور `argvus-window-bar.service` کا مالک ہے۔ مؤخر الذکر اپنے Waybar چائلڈ کو ٹائل میں غیر فعال رکھتا ہے۔ hyprbars پلگ ان `argvus-hyprland` کے ذریعے بنڈل کیا گیا ہے، جو x86_64 پر Hyprland 0.57.0 ABI کے لیے بنایا گیا ہے۔ یہ صارف کی ہوم ڈائرکٹری میں مرتب نہیں ہے۔ کمپوزیٹر ABI اپ ڈیٹ کے ساتھ پلگ ان کو دوبارہ بنائیں۔

کمانڈز: `argvus windows list --json`، `watch --json`، `position top|bottom`، `restore-all`، اور `minimize|restore|activate|maximize|close ADDRESS`۔ ونڈو کے پتے فہرست سے آتے ہیں۔ `position top --persist` مین بار کو دوبارہ شروع کیے بغیر ترجیح کو اسٹور/پروجیکٹ کرتا ہے۔ کنفیگریشن اور رن ٹائم غلطی کی تفصیلات جرنل میں لکھی جاتی ہیں۔ مرئی کارروائی کی ناکامیاں بھی ترجمہ شدہ اطلاع دکھاتی ہیں۔**اسپیسڈ + فلوٹنگ** میں، کھڑکیاں بیرونی خلاء اور دونوں سلاخوں کے ذریعہ محفوظ کردہ علاقوں کا احترام کرتی ہیں۔ حدود میں بارڈر اور ٹائٹل شامل ہیں، جو ایک ہی ونڈو بارڈر کے اندر بیٹھتا ہے۔ ARGVUS کنٹرول کے ذریعے زیادہ سے زیادہ کرنا اس علاقے کو مارجن سے بھر دیتا ہے۔ بحال کرنا پچھلی جیومیٹری کو بحال کرتا ہے، بشمول سروس دوبارہ شروع کرنے کے بعد۔ پوری اسکرین اب بھی پوری آؤٹ پٹ استعمال کرتی ہے۔ تیرتی کھڑکیاں اوورلیپ ہو سکتی ہیں۔ ٹائلنگ اندرونی خلا کو الگ نہیں کرتے ہیں۔

سلاخوں کے ذریعہ محفوظ کردہ اندرونی مارجن کو کھڑکی کے بیرونی خلا میں دو بار شمار نہیں کیا جاتا ہے۔ متوازی وقفہ کاری کے ساتھ، نظر آنے والی کھڑکی سے بار کا فاصلہ اوپر اور نیچے برابر ہے، بشمول سلاخوں کو تبدیل کرنے کے بعد۔

`SUPER + Alt + Space` ٹاسک بار اور کنٹرول سینٹر جیسی ترجیحات کا استعمال کرتے ہوئے عالمی ٹائل/فلوٹنگ موڈ کو ٹوگل کرتا ہے۔ مساوی کمانڈ `argvus config window-mode toggle` ہے۔ ہم آہنگی کی درخواستیں سلسلہ وار ہیں؛ ٹوگلنگ محفوظ کردہ ترجیح کو استعمال کرتی ہے یہاں تک کہ جب درخواست زیر التواء ہو۔ کی بورڈ شارٹ کٹ میں شارٹ کٹ کو تبدیل یا غیر فعال کیا جا سکتا ہے۔ `SUPER + Shift + Space` اپنی انفرادی ونڈو کارروائی کو برقرار رکھتا ہے۔