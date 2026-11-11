---
title: ریپوزٹری کے کردار
description: ورک اسپیس کنسولیڈیشن کے بعد ARGVUS سورس ریپوزٹریز کی ذمہ داریاں۔
---

ذخیروں کو نیویگیشن صفحہ کے بجائے ذمہ داری کے لحاظ سے گروپ کیا گیا ہے:

- ورک اسپیس: `argvus` (ڈسپیچر، ان پروسیس کمانڈز، GTK 4 مددگار، مشترکہ کریٹس، `crates/argvus-greeter` میں گریٹر اور کمانڈز کا ڈیٹا)؛-                                                               :ﻡﺎﻤﻀﻧﺍ ﺭﻭﺍ ﻦﺸﯿﺳ`argvus-session`, `argvus-hyprland`, `argvus-portal`, `argvus-lock`, `argvus-boot-splash`                                                    ) ﻻ﻿ﺍﻭ ﮯﻨﯾﺩ ﮧﯿﻟﺎﺒﻘﺘﺳﺍ ﺭﻭﺍ ،`argvus-greeter`                                                                ﺎﯿﮔ ﺎﯾﺎﻨﺑ ﮯﺳ ،`argvus/crates/argvus-greeter`);
- ؛ﺭﺎﺑ ﮮﻭ ﺭﻭﺍ ﺮﭽﻧﻻ﻿ ،(`argvus-taskbar`) ﺭﺎﺑ ﮏﺳﺎﭨ ،ﻞﻨﯿﭘ ﻝﻭﺮﭩﻨﮐ :UI ﺮﮨﺎﺑ ﮯﮐ ﮯﻗﻼ﻿ﻋ ﮯ
- ؛ﺕﺎﻋﻼ﻿ﻃﺍ ،ﻝﺍﻭ ﺮﺋﺎﻓ ،ﯽﻠﺠﺑ ،ﮯﻠﭙﺳﮈ :ﺯﺭﮉﯿﺋﺍﻭﺮﭘ
- اثاثے اور پیکیجنگ: ظاہری شکل، وال پیپر، فونٹس، شبیہیں، پروفائلز۔## ورک اسپیس میں اکٹھا کیا گیا۔

ان ذخیروں کو `de/argvus` میں ضم کر دیا گیا تھا۔ ان کی دستاویزات اب ورک اسپیس کے `docs/` کے تحت رہتی ہیں، اور ان کے ذخیرے صرف تاریخ کے لیے رکھے جاتے ہیں:

| سابقہ ​​ذخیرہ | اب | دستاویزی |
| --- | --- | --- |
| `argvus-config` | `argvus config` | [کمانڈ لائن](/docs/reference/command-line/)، [کنفیگریشن فائلز](/docs/reference/configuration-files/) |
| `argvus-i18n` | `argvus i18n` | [انٹرنیشنلائزیشن جزو](/docs/components/argvus-i18n/) |
| `argvus-keys`, `argvus-projects` | `argvus keys`, `argvus projects` | کمانڈ لائن |
| `argvus-terminal` | `argvus terminal` | [ٹرمینل جزو](/docs/components/argvus-terminal/) |
| `argvus-system-monitor` | `argvus system-monitor` | [سسٹم مانیٹر کا جزو](/docs/components/argvus-system-monitor/) || `argvus-network` | `argvus network`, `argvus bluetooth` | [نیٹ ورک کا جزو](/docs/components/argvus-network/) || `argvus-widget-telemetry` | `argvus widget-telemetry` | کمانڈ لائن |
| `argvus-control-center` | `argvus control-center` (ٹرمینل ایپلی کیشن) | [کنٹرول سینٹر کا جزو](/docs/components/argvus-control-center/) |
| `argvus-taskbar-calendar` | `argvus taskbar-calendar` (GTK 4 مددگار) | [ٹاسک بار کیلنڈر کا جزو](/docs/components/argvus-taskbar-calendar/) |
| `argvus-removable-devices` | `argvus removable-devices` (GTK 4 مددگار) | [ہٹنے کے قابل آلات کا جزو](/docs/components/argvus-removable-devices/) |
| `argvus-loading-theme` | `argvus loading-theme` (`splash` مددگار) | کمانڈ لائن |
| `argvus-tui-terminal` | `argvus tui-terminal` (ورک اسپیس میں کریٹ) | کمانڈ لائن |
| `argvus-tui`, `argvus-theme` | مشترکہ کریٹس `crates/argvus-tui` اور `crates/argvus-theme` | ڈویلپر نوٹس |
| `argvus-greeter` | `crates/argvus-greeter` میں ماخذ، `argvus` پیکیج میں بھیج دیا گیا | [سلام کرنے والا جزو](/docs/components/argvus-greeter/) |
| `argvus-themes` | ریٹائرڈ۔ ہر آفیشل تھیم اس کا اپنا `argvus-theme-*` پیکیج ہے۔ | [تھیمز انسٹال کرنا](/docs/user-guide/appearance/) |

جمع شدہ اجزاء کے ڈویلپر نوٹس `docs/internal/<repository>/` میں ہیں۔

زبان ملکیت کی حد نہیں ہے: ہر جزو اپنے رن ٹائم مددگار اپنے پیکج کے ساتھ رکھتا ہے۔ ایک سخت قاعدہ جو پیکجوں میں کاٹتا ہے: صرف `argvus-config` `data/generated/` کے نیچے لکھتا ہے۔ ایک جزو جس کو کسی ایسی ایپلیکیشن تک پہنچنے کی ضرورت ہے جو `config.json` کو نہیں پڑھ سکتا ہے اپنے ہیلپر میں ایک اڈاپٹر شامل کرتا ہے اور آرکیسٹریٹر کو کینونیکل کمٹ کے بعد اسے کال کرنے دیتا ہے۔ یہ کبھی بھی تیار کردہ فائل نہیں لکھتا ہے۔ عوامی CLIs کا تعلق `/usr/bin` سے ہے۔ اندرونی مددگار `/usr/lib/argvus/<component>` کے تحت یا کمانڈ ڈائرکٹری میں ہیں جب رن ٹائم پر ان کی ضرورت ہوتی ہے۔ فن تعمیر سے آزاد ڈیٹا `/usr/share/argvus/<component>` کے تحت رہتا ہے۔انسٹال کردہ پیکیج کی حد کے لیے [پیکیج میٹرکس](/docs/reference/package-matrix/) دیکھیں۔