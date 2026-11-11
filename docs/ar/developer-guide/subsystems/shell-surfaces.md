---
title: أسطح الغلاف
description: ملكية شريط المهام ولوحة التحكم والقياس عن بعد.
---

يوفر `argvus-waybar` `/usr/bin/waybar` المصحح. يمتلك `argvus-taskbar` إجراءات شريط المهام وتجربة المستخدم؛ شريط المهام JSONC وCSS الذي يشحنه عبارة عن إسقاطات لـ `config.json`، ويكتب `argvus-config` الطبقة النشطة تحت `data/generated/waybar/` بالإضافة إلى نسخة معكوسة تحت `data/waybar/`. تمتلك `argvus-control-panel` بطاقات Quickshell. تمتلك `argvus-widget-telemetry` شريط القياس عن بعد الاختياري ودلالات المجموعة الخاصة به؛ تتم كتابة JSONC وCSS الخاصة به إلى `data/waybar/` من الإعدادات الافتراضية المجمعة عند تغيير المظهر، وبعد ذلك يقوم `argvus-config` بإعادة كتابة `ARGVUS_TELEMETRY_*` وكتل الخطوط المحددة فقط. تمتلك `argvus-session` خدمات المستخدم الخاصة بها.

لا توجد حزمة `argvus-shell` الحالية أو `argvus-shell.service` في نموذج وقت التشغيل هذا.