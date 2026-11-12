---
title: مصفوفة الحزمة
description: مسؤوليات حزمة ARGVUS.
---

| الحزمة | المسؤولية |
| --- | --- |
| `argvus` | الحزمة الرئيسية: المرسل، والأوامر قيد التشغيل (التكوين، والترجمات، والمفاتيح، والمشاريع، والمحطة، ومراقبة النظام، والشبكة، والبلوتوث، والقياس عن بعد، ومحطة TUI، والإعداد)، والبيانات المشتركة وكتالوجات i18n. كل حزمة مكون تعتمد عليه. |
| `argvus` (المساعدين) | المساعدون في نفس الحزمة: مركز التحكم ومحلل استخدام القرص (التطبيقات الطرفية)، بالإضافة إلى تطبيقات GTK 4 لتقويم شريط المهام، والأجهزة القابلة للإزالة، وتراكب التحميل، وتركيز ARGVUS وساعة ARGVUS. |
| `argvus` (تحية) | قم بتسجيل الدخول للترحيب، ويتم شحنه في نفس الحزمة وتم إنشاؤه من `crates/argvus-greeter`. |
| `argvus-desktop` | مطلوب ملف تعريف سطح المكتب والتطبيقات اليومية بواسطة الحزمة الرئيسية `argvus`. ولا يعتمد على `argvus`، لذا تظل التبعية في اتجاه واحد. |
| `argvus-devtools`, `argvus-pentest` | حزم تعريف مستقلة اختيارية لأدوات التطوير والتدقيق الأمني ​​المعتمد. لا تقوم الحزمة الرئيسية بتثبيت ملفات التعريف هذه. |
| `argvus-session` | نقاط دخول الجلسة ودورة حياة المستخدم. |
| `argvus-hyprland` | تكوين هايبرلاند |
| `argvus-control-panel` | لوحة تحكم كويك شيل. |
| `argvus-taskbar`, `argvus-waybar` | تكوين شريط الطريق والثنائي المصحح. |
| `argvus-branding` | أصول الشعار والعلامة النصية. |
| `argvus-appearance`، `argvus-wallpapers`، `argvus-fonts`، `argvus-icons` | حالة المظهر والأصول البصرية. |
| `argvus-display`, `argvus-power`, `argvus-notifications` | مقدمي سطح المكتب. |
| `argvus-lock`, `argvus-boot-splash` | قفل الشاشة وبداية التمهيد. |
| `argvus-firewall` | جدار الحماية، مع الامتيازات الخاصة به. |
| `argvus-launcher` | تكوين قاذفة ونقاط الدخول. || `argvus-app-profiles` | ملفات تعريف البيانات فقط للتطبيقات الطرفية المجمعة. |
| `argvus-portal` | بوابة XDG والإعدادات الافتراضية لـ Wayland/DBus. |
| `argvus-theme-*` | حزمة اختيارية واحدة لكل مجموعة سمات رسمية. || `argvus-game-snake`, `argvus-games` | خياري. الأفعى هو مشروع مستقل. يقوم `argvus-games` بتجميع الألعاب الاختيارية. |

## الحزم المستبدلة`argvus` declares `replaces`, `conflicts` and `provides`     :ﺔﻤﻴﺘﻳ ﺕﺎﻔﻠﻤﻛ ﺎﻬﺗﺎﻔﻠﻣ ﻰﻘﺒﺗ ﻻ﻿ﻭ ﺎﻬﻠﻳﺰﺗ ﺔﻴﻗﺮﺘﻟﺍ ﻥﺈﻓ ﺍﺬﻟ ،ﺎﻬﺒﻋﻮﺘﺳﺍ ﻲﺘﻟﺍ ﻡﺰﺤﻠﻟ

`argvus-config`، `argvus-i18n` (مع `argvus-language-en-us` و`argvus-language-pt-br`)، `argvus-keys`، `argvus-projects`، `argvus-terminal`، `argvus-system-monitor` و`argvus-network` و`argvus-widget-telemetry` و`argvus-du`. يحل `argvus` أيضًا محل `argvus-control-center`، و`argvus-taskbar-calendar`، و`argvus-loading-theme`، و`argvus-removable-devices`، والحزم المقسمة السابقة `argvus-gtk` و `argvus-greeter`، والذي لا يزال يوفره بحيث تستمر الحزم المعتمدة عليها في الحل. تم إنشاء مدير الحساب (`argvus accounts`) من `crates/argvus-accounts` وتم تثبيته كـ `/usr/lib/argvus/commands/accounts`، مع قاعدة polkit الخاصة به. تم إنشاء محلل استخدام القرص من `crates/argvus-du` وتم تثبيته كـ `/usr/lib/argvus/commands/argvus-du`. يظل `/usr/bin/argvus-terminal` مثبتًا كمشغل توافق لـ `argvus terminal`. لا تزال محطة Hyprland الافتراضية وكتالوج مركز التحكم والتطبيقات الافتراضية المحفوظة تسمي هذا الأمر.

لا يتم توفير الحزم التي تمت إزالتها `argvus-pwvucontrol` و`argvus-snappy-switcher` بواسطة أي حزمة. لا يلزم وجود أمر تثبيت بين حزم مكونات ARGVUS: `argvus` هي الحزمة الرئيسية الخاصة بها، ويعتمد كل مكون عليها. يؤدي تثبيت `argvus` أيضًا إلى تثبيت ملف التعريف `argvus-desktop`؛ يظل `argvus-devtools` و`argvus-pentest` قيد الاشتراك. لا يعني وجود الحزمة في هذا الجدول أنها تمتلك الميزة بأكملها: على سبيل المثال، يجمع شريط المهام بين `argvus-waybar` و`argvus-taskbar` و`argvus-session` وموفري المجال.
