---
title: أسطح الغلاف
description: ملكية شريط المهام ولوحة التحكم والقياس عن بعد.
---

يوفر `argvus-shell` `Quickshell Taskbar` المصحح. يمتلك `argvus-shell` إجراءات شريط المهام وتجربة المستخدم؛ شريط المهام JSONC وCSS الذي يشحنه عبارة عن إسقاطات لـ `config.json`، ويكتب `argvus-config` الطبقة النشطة تحت `data/generated/taskbar/` بالإضافة إلى نسخة معكوسة تحت `data/taskbar/`. تمتلك `argvus-shell` بطاقات Quickshell. تمتلك `argvus-shell` شريط القياس عن بعد الاختياري ودلالات المجموعة الخاصة به؛ تتم كتابة JSONC وCSS الخاصة به إلى `data/taskbar/` من الإعدادات الافتراضية المجمعة عند تغيير المظهر، وبعد ذلك يقوم `argvus-config` بإعادة كتابة `ARGVUS_TELEMETRY_*` وكتل الخطوط المحددة فقط. تمتلك `argvus-session` خدمات المستخدم الخاصة بها.

لا توجد حزمة `argvus-shell` الحالية أو `argvus-shell.service` في نموذج وقت التشغيل هذا.