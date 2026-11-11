---
title: Introduction
description: ما هو ARGVUS وكيف يعمل سطح المكتب الخاص به وأين تتناسب مكوناته.
---

ARGVUS هي بيئة سطح مكتب معيارية Hyprland وWayland لنظام Arch Linux. إنها مجموعة متكاملة من الحزم بدلاً من تطبيق واحد متجانس: تعمل الجلسة والمؤلف وأسطح الصدفة وتطبيقات الإعدادات والموفرون والأصول المرئية معًا من خلال الحالة المشتركة وعقود وقت التشغيل الصريحة.

هذه الصفحة هي خريطة الوثائق. فهو يشرح سطح المكتب بأكمله بمستوى عالٍ؛ تحتوي الصفحات المرتبطة على تفاصيل التشغيل والتنفيذ.

## كيف يعمل ARGVUS

تتبع الجلسة العادية هذا المسار:

```text
graphical login or TTY
        ↓
argvus-session → argvus start
        ↓
Hyprland + argvus-hyprland
        ↓
argvus-session.target (systemd --user)
        ↓
taskbar · control panel · notifications · wallpaper · idle · clipboard
```

يمتلك `argvus-session` دورة حياة الجلسة وإعداد البيئة. إنه لا ينفذ كل ميزات سطح المكتب: يحتفظ كل مكون بملكية الأوامر والتكوين وحمولة الخدمة الخاصة به.


## الطبقات الرئيسية

### نقاط الدخول والبنية التحتية

توفر الحزمة `argvus` مرسل `/usr/bin/argvus` ونقطة إدخال الحزمة الرئيسية. يوفر `argvus-session` نقاط دخول رسومية وTTY وهدف المستخدم و`argvus sessionctl`. تأتي وظيفة TUI والترجمة المشتركة من مساحة العمل `argvus` (صناديق `argvus-tui` و`argvus-i18n`) والأمر `argvus i18n`.

### المُركب وسطح المكتب

يوفر `argvus-hyprland` تكوين Hyprland وربطات المفاتيح. يوفر `argvus-waybar` ثنائي Waybar المصحح، بينما يمتلك `argvus-taskbar` تكوين شريط المهام وإجراءاته. يوفر `argvus-control-panel` لوحة Quickshell، ويوفر `argvus widget-telemetry` عناصر واجهة مستخدم النظام الاختيارية، قيد التشغيل في `argvus`.


### الإعدادات والتطبيقات

`argvus control-center` هو تطبيق إعدادات لوحة المفاتيح أولاً، والذي يتم شحنه كمساعد في `argvus`. أدوات التقويم والأجهزة القابلة للإزالة هي مساعدات GTK 4 في `argvus`؛ تعتبر شاشة النظام والمحطة أوامر فرعية لـ `argvus`؛ المشغل هو `argvus-launcher`. تم دمجها جميعًا مع الصدفة والمرسل.


### مقدمي الخدمات وعمليات التكامل

يتم توفير العرض والشبكة والبلوتوث والطاقة والإشعارات وجدار الحماية والتخزين القابل للإزالة من خلال المشاريع المركزة. يستهلك مركز التحكم ولوحة التحكم هؤلاء الموفرين؛ لا يحل محلهم.

تعتبر مرئيات تسجيل الدخول والقفل وبدء التشغيل منفصلة أيضًا: `argvus-greeter` يتعامل مع الترحيب، `argvus-lock` يتعامل مع Hyprlock، مساعد التسليم الذي يتم شحنه في `argvus` يرسم قرص تسجيل الدخول، ويتعامل تراكب التحميل مع انتقالات السمة، ويتعامل `argvus-boot-splash` `$XDG_CONFIG_HOME/argvus`  `~/.config/argvus`  `argvus-config`  `data/generated/`  `argvus launcher`  `data/rofi/`  مع بداية التمهيد.


## حالة المظهر المشترك


يتحكم النظام البصري أيضًا في التخطيط. الوضع الثابت مضغوط ومربع. يستخدم الوضع المتباعد فجوات نافذة أكبر وزوايا مستديرة وهوامش أوسع حول شريط المهام والهيكل. يتم تعيين شريط المهام افتراضيًا على الحافة العلوية ولكنه يدعم الموضع العلوي/السفلي من خلال حالة التخطيط المشترك. تتحكم التأثيرات في الشفافية والتمويه والظلال والرسوم المتحركة عبر أسطح المُركب وسطح المكتب.

راجع [جدول السمات] (/docs/user-guide/appearance/) للعائلات والألوان الحالية، و[التأثيرات] (/docs/argvus-appearance/effects/) لعقد التأثيرات المشتركة، و[شريط المهام] (/docs/argvus-taskbar/taskbar/) لوضع الشريط والنافذة.

## ما هو ARGVUS وما هو ليس كذلك

ARGVUS هي طبقة بيئة سطح المكتب الموجودة أعلى نظام Arch Linux. يوفر Arch Linux نظام التشغيل والحزم؛ يوفر Hyprland تكوين Wayland. يقوم ARGVUS بتنسيق الجلسة الرسومية وأسطح الغلاف والإعدادات والمظهر وخدمات سطح المكتب والتطبيقات المتكاملة.

وبالتالي فهو أكثر من مجرد تكوين Hyprland أو مجموعة من ملفات النقاط الشخصية. تحدد ARGVUS نقطة دخول جلسة مدعومة، وتمتلك دورة حياة خدمات سطح المكتب الخاصة بها، وتوفر إعدادات مشتركة ونموذج مظهر، وتربط المكونات المركزة مثل الشبكات وشاشات العرض والطاقة والإشعارات والحسابات والتخزين القابل للإزالة.

لا يحتاج ARGVUS إلى استبدال كل تطبيق. فهو يدمج تطبيقات مختارة حيث يكون هذا هو الخيار الأكثر موثوقية، مع الحفاظ على تماسك السلوك والتكوين على مستوى سطح المكتب.



## تابع بالهدف

-  for the installed and user paths.(/docs/developer-guide/architecture/overview/).

                                                    ﺮﻈﻧﺍ ،ﻞﻴﻐﺸﺘﻟﺍ ءﺪﺑﻭ ﺖﻴﺒﺜﺘﻠﻟ[                                                                       ﺖﻴﺒﺜﺘﻟﺍ](/docs/getting-started/installation/)                                                                             ﻭ[                                                                         ﺕﺎﺴﻠﺟ](/docs/user-guide/sessions/)                                                  ﺭﻮﻄﻤﻟﺍ ﻊﺟﺍﺭ ،ﺔﻣﺪﺨﻟﺍ ﺔﻴﻜﻠﻤﻟ .[                                                         ﻞﻴﻐﺸﺘﻟﺍ ﺖﻗﻭ ﺓﺎﻴﺣ ﺓﺭﻭﺩ](/docs/developer-guide/architecture/runtime-lifecycle/).
                                                                          ﺮﻈﻧﺍ[                                                                    ﺐﺘﻜﻤﻟﺍ ﺢﻄﺳ](/docs/user-guide/desktop/)                                                                    ﻭ ﻡﺍﺪﺨﺘﺳﻼ﻿ﻟ[                                                                   ﺓﺮﺸﻘﻟﺍ ﺢﻄﺳﺃ](/docs/developer-guide/subsystems/shell-surfaces/)                                                               .ﺬﻴﻔﻨﺘﻟﺍ ﺔﻴﻜﻠﻤﻟ
                                                                          ﺮﻈﻧﺍ[                                                                     ﺕﺎﻘﻴﺒﻄﺘﻟﺍ](/docs/user-guide/applications/)                                                                             ﻭ[                                                                   ﺮﻣﺍﻭﻷ﻿ﺍ ﻊﺟﺮﻣ](/docs/reference/command-line/).
                                                                          ﺮﻈﻧﺍ[                                                                        ﺓﺰﻬﺟﻷ﻿ﺍ](/docs/user-guide/hardware/), [                                                               ﻥﺎﻣﻷ﻿ﺍﻭ ﺔﻴﺻﻮﺼﺨﻟﺍ](/docs/user-guide/privacy-and-security/)                                                                       ﺭﻮﻄﻤﻟﺍﻭ[                                                                   ﻥﻮﻜﻤﻟﺍ ﻊﺟﺮﻣ](/docs/developer-guide/).
                      ﺮﻈﻧﺍ .ﺕﺎﻣﻮﻠﻌﻤﻠﻟ ﻲﻘﻴﻘﺤﻟﺍ ﺭﺪﺼﻤﻟﺍ ﺲﻴﻟ ﻩﺅﺎﺸﻧﺇ ﻢﺗ ﻱﺬﻟﺍ ﻞﻴﻟﺪﻟﺍ[                                                                        ﺮﻬﻈﻤﻟﺍ](/docs/user-guide/appearance/)                                                                             ﻭ[                                                               ﺔﻟﺎﺤﻟﺍﻭ ﻦﻳﻮﻜﺘﻟﺍ](/docs/developer-guide/architecture/configuration-and-state/).
                                                                      ﻰﻟﺇ ﺮﻈﻧﺍ[                                                                  ﻉﻮﺿﻮﻤﻟﺍ ﻝﻭﺪﺟ](/docs/user-guide/appearance/)                                                      ،ﺔﻴﻟﺎﺤﻟﺍ ﻥﺍﻮﻟﻷ﻿ﺍﻭ ﺕﻼ﻿ﺋﺎﻌﻠﻟ[                                                                       ﺕﺍﺮﻴﺛﺄﺗ](/docs/argvus-appearance/effects/)                                                    ﻭ ،ﺔﻛﺮﺘﺸﻤﻟﺍ ﺕﺍﺮﻴﺛﺄﺘﻟﺍ ﺪﻘﻌﻟ[                                                                   ﻡﺎﻬﻤﻟﺍ ﻂﻳﺮﺷ](/docs/argvus-taskbar/taskbar/)                                                          .ﺬﻓﺍﻮﻨﻟﺍﻭ ﺭﺎﺒﻟﺍ ﻊﺿﻮﻟ
 ﺮﻈﻧﺍ ،ﻖﻄﻨﻟﺍ ﺕﻮﺻﻭ ﻞﻣﺎﻜﻟﺍ ﺡﺮﺸﻟﺍ ﻰﻠﻋ ﻝﻮﺼﺤﻠﻟ .ﻡﺪﺨﺘﺴﻤﻟﺍ ﻰﻠﻋ ﺰﻛﺮﻳ ﻱﺬﻟﺍ ﺐﺘﻜﻤﻟﺍ ﺢﻄﺳ ﻑ[                                                                     ARGVUS ﻦﻋ](/about/).
                                      ﺭﻮﻄﻤﻟﺍ ﻊﺟﺍﺭ ،ﺔﻣﺰﺤﻟﺍ ﺔﻴﻜﻠﻣﻭ ﺬﻴﻔﻨﺘﻟﺍ ﺩﻭﺪﺤﻟ[                                               ﺔﻳﺭﺎﻤﻌﻤﻟﺍ ﺔﺳﺪﻨﻬﻟﺍ ﻰﻠﻋ ﺔﻣﺎﻋ ﺓﺮﻈﻧ](/docs/developer-guide/architecture/overview/)                                                                             ﻭ[                                                                ﻉﺩﻮﺘﺴﻤﻟﺍ ﺭﺍﻭﺩﺃ](/docs/developer-guide/architecture/repository-roles/).
- [                                                           ﺔﻌﺋﺎﺸﻟﺍ ﺔﻠﺌﺳﻷ﻿ﺍ ﺃﺮﻗﺍ](/docs/user-guide/faq/).
- [                                                                    ARGVUS ﺖﺒﺛ](/docs/getting-started/installation/).
- [                                                   TTY ﺔﺴﻠﺟ ﻭﺃ ﺔﻴﻣﻮﺳﺭ ﺔﺴﻠﺟ ءﺪﺑ](/docs/user-guide/sessions/).
- [                                                  ﺔﺷﺎﺸﻟﺍ ﺕﺎﻴﻔﻠﺧﻭ ﺕﺎﻤﻴﺜﻟﺍ ﻦﻳﻮﻜﺗ](/docs/user-guide/appearance/).
- [                                               ﻢﻜﺤﺘﻟﺍ ﺔﺣﻮﻟﻭ ﻡﺎﻬﻤﻟﺍ ﻂﻳﺮﺷ ﻡﺪﺨﺘﺳﺍ](/docs/user-guide/desktop/).
- [                                                 ﺕﺎﻣﺪﺨﻟﺍﻭ ﺕﺍﺭﺎﺴﻤﻟﺍﻭ ﺮﻣﺍﻭﻷ﻿ﺍ ﻢﻬﻓ](/docs/reference/).
- [                                                                  ﺓﺭﺎﻤﻌﻟﺍ ﺱﺭﺩﺍ](/docs/developer-guide/architecture/overview/).
