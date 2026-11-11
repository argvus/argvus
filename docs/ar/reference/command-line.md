---
title: سطر الأوامر
description: أمر argvus وأوامره الفرعية والأعلام المحفوظة لربطات المفاتيح ومكان تشغيل كل منها.
---

`/usr/bin/argvus` هي نقطة الدخول العامة لـ ARGVUS. إنه أحد برامج Rust التي تم إنشاؤها من صندوق `argvus-cli`. يتم تشغيل كل أمر فرعي داخل هذا البرنامج، أو تشغيل مساعد قابل للتنفيذ من دليل الأوامر، أو تشغيل برنامج نصي معبأ.

قم بتشغيل `argvus --help` للقائمة المثبتة مع الإصدار الخاص بك. تتم كتابة الأوامر الفرعية بدون `--` البادئة. يتم الاحتفاظ بنماذج `--` فقط عندما يستخدمها رابط المفاتيح أو إدخال سطح المكتب.

## مكان تشغيل كل أمر

- **قيد التشغيل:** الأمر عبارة عن مكتبة لمساحة العمل ويتم تشغيله داخل `argvus`. لا يوجد ملف قابل للتنفيذ منفصل. هذه هي `config`، `theme`، `i18n`، `keys`، `gpg`، `projects`، `terminal`، `system-monitor`، `network`، `bluetooth`، `focus status`، `widget-telemetry`، `tui-terminal`، `sensors` و `setup`.
- **المساعد:** ملف قابل للتنفيذ منفصل في `/usr/lib/argvus/commands/`، تم تثبيته بواسطة الحزمة `argvus`. هذه هي `control-center` (تطبيق طرفي)، `argvus-du` (تم إطلاقه مع `argvus du`) وتطبيقات GTK 4 `taskbar-calendar`، `removable-devices`، `focus` و `clock`.
- **خاص قابل للتنفيذ:** ملف قابل للتنفيذ تقوم بتثبيته حزمة أخرى في `/usr/lib/argvus/commands/`. `argvus` يبدأ تشغيله فقط. وتشمل هذه `session`، `start`، `tty`، `sessionctl`، `launcher`، `snippets`، `ssh`، `git`، `app-profiles-yazi-keymap`، `notifications`، `displayctl`، `firewall`، `accounts`، `taskbar-actions` و نقاط الدخول `greeter-*`.
- **البرنامج النصي:** برنامج نصي مُعبأ يعمل باستخدام `/bin/sh`. هذه هي أوامر البرنامج النصي Hyprland المدرجة أدناه.

يقوم `ARGVUS_COMMANDS_DIR` بتغيير دليل الأوامر للاختبارات وعمليات التحقق من `DESTDIR`. يتم البحث عن الملف القابل للتنفيذ الخاص الذي لم يتم العثور عليه هناك على `PATH`.

## الأوامر الفرعية

| الأمر | يدير | ملاحظات |
| --- | --- | --- |
| `argvus config ...` | قيد المعالجة | نموذج التكوين وأدواته. تم الوصول إليه أيضًا بواسطة `theme-switch` و`effects-toggle`. |
| `argvus theme ...` | قيد المعالجة | `apply ID` هو مفتاح السمة (مثل `theme-switch`). إنشاءات السمات: `list` السمات المثبتة، `check DIRECTORY` سمة مقابل القوالب المثبتة، `render ID [--variant sticky\|spaced] [--accent #RRGGBB] [--output DIRECTORY]`. راجع [المظهر والسمات](/docs/developer-guide/subsystems/appearance-and-themes/). |
| `argvus i18n ...` | قيد المعالجة | الترجمات: `get DOMAIN KEY [name=value...]`، `locale`، `list`، `validate [LOCALE]`. |
| `argvus keys ...` | قيد المعالجة | مدير مفاتيح SSH. `gpg` هو اسم مستعار لأوامر GPG الفرعية. |
| `argvus gpg ...` | قيد المعالجة | مفاتيح GPG: `list`، `status`، `generate`، `export-public`، `export-private`، `import`، `import-private`، `delete`، `edit`، `keyserver`. |
| `argvus projects ...` | قيد المعالجة | منتقي المشروع. |
| `argvus terminal ...` | قيد المعالجة | ملف تعريف محطة ARGVUS. |
| `argvus system-monitor ...` | قيد المعالجة | مراقب نظام btop؛ يحدد `--gpu` صناديق GPU فقط، ويقوم `--panel [--cpu|--mem|--gpu]` بتضمينها في غلاف ARGVUS TUI. |
| `argvus du` | مساعد (تطبيق المحطة) | يفتح ARGVUS Disk Use Analyzer في نافذة Kitty مخصصة ويقوم بتشغيل ncdu. |
| `argvus network ...` | قيد المعالجة | حالة NetworkManager وعناصر التحكم (يتم الاحتفاظ بـ `networkctl` كاسم مستعار). |
| `argvus bluetooth ...` | قيد المعالجة | حالة Bluetooth وعناصر التحكم (يتم الاحتفاظ بـ `bluetoothctl` كاسم مستعار). |
| `argvus widget-telemetry block ...` | قيد المعالجة | كتلة القياس عن بعد Waybar. هذا هو المسار الساخن للشريط. || `argvus widget-telemetry toggle ...` | قيد المعالجة | يقوم بتشغيل وإيقاف كتل القياس عن بعد، وترتيبها. |
| `argvus game-snake` | خاص قابل للتنفيذ | لعبة Snake الطرفية، من مشروع `argvus-game-snake` المستقل، حزمة اختيارية. خارج المحطة يتم فتح واحدة. |
| `argvus control-center [page]` | مساعد (تطبيق المحطة) | يفتح مركز التحكم، أو يركز على النافذة المفتوحة بالفعل. |
| `argvus welcome [--if-needed]` | المرسل → مركز التحكم | يفتح صفحة الترحيب المدمجة في مركز التحكم. يتم فتح `--if-needed` مرة واحدة ما لم يتم تسجيل التقدم لكل مستخدم والذي تم بالفعل عرض الترحيب أو إكماله أو رفضه. |
| `argvus taskbar-calendar [args]` | تطبيق GTK 4 | الإعدادات الافتراضية هي `toggle`. |
| `argvus removable-devices [args]` | تطبيق GTK 4 | الإعدادات الافتراضية هي `menu`. |
| `argvus focus [--toggle] [--start\|--pause] [--x X --y Y] [--bar-bottom PX --bar-right PX]` | تطبيق GTK 4 | ARGVUS التركيز المنبثق. مثيل واحد لكل جلسة. |
| `argvus focus status --waybar [--follow]` | قيد المعالجة | يُصدر مؤشر شريط المهام Focus كـ Waybar JSON؛ يتغير تدفق `--follow` بدون تحميل GTK. |
| `argvus clock [--toggle] [--background] [--page PAGE] [--x X --y Y] [--bar-bottom PX --bar-right PX]` | تطبيق GTK 4 | ARGVUS الساعة المنبثقة. مثيل واحد لكل جلسة؛ `--background` يبقي التنبيهات نشطة بدون نافذة منبثقة. |
| `argvus loading-theme ...` | معبأة `splash` في `/usr/lib/argvus/loading-theme/splash` | تراكب عمليات إعادة التحميل المباشرة للجلسة؛ لا يستخدمه مفتاح السمة. |
| `argvus session`، `start`، `tty`، `sessionctl` | الملفات التنفيذية الخاصة | نقاط دخول الجلسة ووحدة التحكم بالجلسة. |
| `argvus greeter`، `greeter-session`، `greeter-tui`، `greeter-setup` | الملفات التنفيذية الخاصة | نقاط الدخول تحية. |
| `argvus accounts`, `firewall` | الملفات التنفيذية الخاصة | أدوات الحسابات وجدار الحماية. |
| `argvus launcher`، `snippets`، `ssh`، `git` | الملفات التنفيذية الخاصة | أدوات الإطلاق والمنتقي. |
| `argvus tui-terminal` | قيد المعالجة | المجمع الطرفي لتطبيقات TUI. |
| `argvus sensors` | قيد المعالجة | نظرة عامة على درجة الحرارة الحية من `sensors -j`. |
| `app-profiles-yazi-keymap` | خاص قابل للتنفيذ | مساعد خريطة المفاتيح. |
| `argvus notifications`، `displayctl`، `taskbar-actions` | الملفات التنفيذية الخاصة | الإخطارات والعرض وإجراءات شريط المهام. |
| `argvus setup [options]` | قيد المعالجة | نسخ الإعدادات الافتراضية المعبأة في تكوين المستخدم. انظر أدناه. |

تتم إعادة توجيه الوسائط دون تغيير، بما في ذلك المسافات والقيم الفارغة. ينتمي الإدخال القياسي والإخراج والخطأ والإشارات وحالة الخروج إلى الأمر الذي يتم تشغيله. فشل اسم غير معروف مع تلميح لتشغيل `argvus --help`. الأمر الذي يتم تشغيله أثناء العملية يرفض وسيطة غير صالحة UTF-8 (حالة الخروج 2) بدلاً من تمرير قيمة تم تغييرها؛ الأوامر التي تقوم بتشغيل برنامج منفصل تتلقى البايتات دون تغيير.

## الأعلام والأسماء المستعارة

توجد هذه الأسماء بحيث تستمر روابط المفاتيح وإدخالات سطح المكتب والبرامج النصية القديمة في العمل. تفضل نموذج الأمر الفرعي في الكود الجديد.

| العلم أو الاسم المستعار | يعادل |
| --- | --- |
| `--control-center`، `--about`، `--default-apps` | `control-center` (مع صفحات `about` و`apps`) |
| صفحة `--displays`، `--monitors` | `control-center displays` |
| `--network-interfaces` | `control-center network interfaces` |
| `--system-monitor`، `--terminal`، `--game-snake` | `system-monitor`، `terminal`، `game-snake` |
| `--calendar`، `--removable-devices`، `--storage` | `taskbar-calendar`, `removable-devices` |
| `--focus`, `--clock` | `focus`, `clock` |
| `--spf`, `--superfile` | مدير الملفات (`spf`) |
| `--yazi`, `--yazy` | متصفح الملفات (`yazi`) |
| `--setup` | `setup` |
| `--start-desktop` | يبدأ `tty` (لا تتم إعادة توجيه أي وسائط) |
| `--version`، `-V`، `version` | يطبع `ARGVUS <version>` |
| `help`، `--help`، `-h` | يطبع الاستخدام || `networkctl`, `bluetoothctl` | `network`, `bluetooth` |
| `widget-telemetry-toggle`, `widget-telemetry-block` | `widget-telemetry toggle`، `widget-telemetry block` |
| `theme-switch` | `config theme-apply` |
| `effects-toggle` | `config effects` |

يقبل مركز التحكم علامات الصفحات القديمة كوسيطة أولى ويقوم بتعيينها لأسماء الصفحات. أمثلة: `--apps` يصبح `apps`، `--keyboard-layout` يصبح `locale`، و`--hostname` يصبح `system`.

## أوامر البرنامج النصي Hyprland

تعمل هذه البرامج النصية المجمعة ويتم استخدامها بواسطة تكوين Hyprland:

`cheatsheets-hypr`، `cheatsheets-launcher`، `scratchpad`، `hypr-screenshot`، `spaces-switch`، `emoji-picker`، `toggle-sidebar`، `weather-location`، `layout-mode-menu`، `brightness-switch`، `toggle-mode`، `idle-timeout`، `keep-awake`، `hypr-power-menu`.

يتجاوز البرنامج النصي الموجود في تكوين المستخدم البرنامج النصي المعبأ. ترتيب البحث هو نفسه `_config_path` في `hyprland.lua`: نسخة تكوين المستخدم، ثم نسخة بيانات ARGVUS، ثم نسخة الحالة، ثم النسخة التي تم إنشاؤها، وبعد ذلك فقط الملف المعبأ تحت `/usr/share/argvus`.

## أمثلة

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

## الإعداد

يقوم `argvus setup` بنسخ الإعدادات الافتراضية المجمعة من `/usr/share/argvus/<app>` إلى `~/.config/argvus/data/<app>`، بحيث يمكنك تخصيصها. لا يقوم أبدًا بتحرير الملفات المجمعة.

```sh
argvus setup --copy <app>          # copy one packaged app
argvus setup --copy-all            # copy every packaged app directory
argvus setup --copy <app> --force  # back up the existing destination first
argvus setup --copy <app> --dry-run
```

يتم الإبلاغ عن الوجهة الموجودة بالفعل وتركها بمفردها، ما لم يتم تحديد `--force`. باستخدام `--force`، تتم إعادة تسمية الدليل الحالي إلى `<name>.bak-<timestamp>` قبل النسخة.

يعد تطبيق `argvus` خاصًا: فهو ينسخ محتويات الدليل المعبأ الخاص به إلى جذر التكوين. نظرًا لوجود جذر التكوين بالفعل عند تشغيل الإعداد، يحتاج `--copy argvus` إلى `--force` في التعليمات البرمجية الحالية.

يقوم الإعداد بالتحقق من صحة كل خيار قبل أن يغير أي شيء. تم رفض أسماء التطبيقات غير الصالحة (فارغة، تبدأ بـ `.`، أو تحتوي على `/`).

## الجلسة وإعادة التحميل

يقوم `SUPER + SHIFT + R` بتشغيل `argvus sessionctl reload`. تطبق وحدة التحكم بالجلسة التكوين الذي تم إنشاؤه وتعيد تشغيل المكونات التي تمتلكها فقط. ولا ينسخ `/usr/share/argvus` إلى الدليل الرئيسي، ولا يستبدل ملف JSON المشوه.

## ضوء الليل

             ﻞﻴﻐﺸﺘﺑ ﻢﻗ ﻭﺃ **ﻲﻠﻴﻠﻟﺍ ءﻮﻀﻟﺍ < ﺕﺎﺷﺎﺸﻟﺍ < ﻝﺎﺧﺩﻹ﻿ﺍﻭ ﺐﺘﻜﻤﻟﺍ ﺢﻄﺳ** ﺢﺘﻓﺍ`argvus control-center displays night-light`      ﻝﻼ﻿ﺧ ﻦﻣ ﺔﺘﺑﺎﺜﻟﺍ ﺔﻴﻠﺤﻤﻟﺍ ﻝﻭﺍﺪﺠﻟﺍﻭ ﻱﻭﺪﻴﻟﺍ ﻂﻴﺸﻨﺘﻟﺍ ،ﺓﺭﺍﺮﺤﻟﺍ ﺔﺟﺭﺩ ﻦﻳﻮﻜﺘﺑ ﻢﻗ .`argvus config display night-light`                                                                        ﺮﻈﻧﺍ .[                                                                     ﻞﻴﻠﻟﺍ ءﻮﺿ](/docs/user-guide/night-light/)                                        .ﻞﻴﻐﺸﺘﻟﺍ ﺖﻗﻭ ﺕﺎﺒﻠﻄﺘﻣﻭ ،ﺔﻟﺎﺤﻟﺍﻭ ،ﺮﻣﺍﻭﻸ﻿ﻟ

يقوم `argvus tui-terminal --frame TITLE FOOTER COMMAND [ARG...]` بتشغيل أمر تفاعلي داخل إطار محطة ARGVUS المشترك. Ctrl+E يغلق الإطار؛ المتصل به يوفر التذييل المترجم. استخدم هذا داخل المحطة، أو عند تمرير الأمر إلى `argvus tui-terminal --term kitty -- ...`.

يستخدم `argvus tui-terminal --term kitty --profile ai-agents` تكوين Kitty مخصصًا مع علامات تبويب مخفية واختصارات الحافظة فقط. يقوم المتصل بتوفير الأمر والعنوان؛ تتم معالجة وضع النافذة بواسطة قاعدة Hyprland للفئة `argvus-ai`.