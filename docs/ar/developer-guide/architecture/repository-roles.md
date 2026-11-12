---
title: أدوار المستودع
description: مسؤوليات مستودعات مصدر ARGVUS بعد دمج مساحة العمل.
---

يتم تجميع المستودعات حسب المسؤولية وليس حسب صفحة التنقل:

- مساحة العمل: `argvus` (المرسل، الأوامر قيد التشغيل، مساعدو GTK 4، الصناديق المشتركة، المُرحِّب في `crates/argvus-greeter` وبيانات الأوامر)؛
- الجلسة والتكامل: `argvus-session`، `argvus-hyprland`، `argvus-portal`، `argvus-lock`، `argvus-boot-splash`، والمرحب (`argvus-greeter`، مبني من `argvus/crates/argvus-greeter`);
- واجهة المستخدم خارج مساحة العمل: لوحة التحكم، وشريط المهام (`argvus-taskbar`)، والمشغل وشريط الطريق؛
- مقدمي الخدمة: العرض، والطاقة، وجدار الحماية، والإخطارات؛
- الأصول والتغليف: المظهر والخلفيات والخطوط والأيقونات والملفات الشخصية.

## تم الدمج في مساحة العمل

تم دمج هذه المستودعات في `de/argvus`. توجد وثائقهم الآن ضمن `docs/` لمساحة العمل، ويتم الاحتفاظ بمستودعاتهم للتاريخ فقط:

| المستودع السابق | الآن | التوثيق |
| --- | --- | --- |
| `argvus-config` | `argvus config` | [سطر الأوامر](/docs/reference/command-line/)، [ملفات التكوين](/docs/reference/configuration-files/) |
| `argvus-i18n` | `argvus i18n` | [عنصر التدويل](/docs/components/argvus-i18n/) |
| `argvus-keys`, `argvus-projects` | `argvus keys`, `argvus projects` | سطر الأوامر |
| `argvus-terminal` | `argvus terminal` | [المكون الطرفي](/docs/components/argvus-terminal/) |
| `argvus-system-monitor` | `argvus system-monitor` | [مكون مراقبة النظام](/docs/components/argvus-system-monitor/) |
| `argvus-network` | `argvus network`, `argvus bluetooth` | [مكون الشبكة](/docs/components/argvus-network/) |
| `argvus-widget-telemetry` | `argvus widget-telemetry` | سطر الأوامر |
| `argvus-control-center` | `argvus control-center` (تطبيق المحطة) | [مكون مركز التحكم](/docs/components/argvus-control-center/) |
| `argvus-taskbar-calendar` | `argvus taskbar-calendar` (مساعد GTK 4) | [مكون تقويم شريط المهام](/docs/components/argvus-taskbar-calendar/) |
| `argvus-removable-devices` | `argvus removable-devices` (مساعد GTK 4) | [مكون الأجهزة القابلة للإزالة](/docs/components/argvus-removable-devices/) |
| `argvus-loading-theme` | `argvus loading-theme` (مساعد `splash`) | سطر الأوامر |
| `argvus-tui-terminal` | `argvus tui-terminal` (صندوق في مساحة العمل) | سطر الأوامر |
| `argvus-tui`, `argvus-theme` | الصناديق المشتركة `crates/argvus-tui` و`crates/argvus-theme` | ملاحظات المطور |
| `argvus-greeter` | المصدر في `crates/argvus-greeter`، مشحون في حزمة `argvus` | [مكون الترحيب](/docs/components/argvus-greeter/) |
| `argvus-themes` | متقاعد. كل سمة رسمية هي حزمة `argvus-theme-*` الخاصة بها. | [تثبيت المواضيع](/docs/user-guide/appearance/) |

ملاحظات المطور الخاصة بالمكونات المدمجة موجودة في `docs/internal/<repository>/`.

اللغة ليست حدود الملكية: يحتفظ كل مكون بمساعديه في وقت التشغيل مع حزمته. القاعدة الصعبة الوحيدة التي تشمل الحزم: فقط `argvus-config` يكتب أسفل `data/generated/`. يقوم المكون الذي يحتاج إلى الوصول إلى تطبيق لا يمكنه قراءة `config.json` بإضافة محول في مساعده الخاص ويسمح للمنسق بالاتصال به بعد الالتزام القانوني؛ لا يكتب ملفًا تم إنشاؤه أبدًا. تنتمي CLIs العامة إلى `/usr/bin`؛ تنتمي المساعدات الداخلية إلى `/usr/lib/argvus/<component>` أو إلى دليل الأوامر عندما تكون هناك حاجة إليها في وقت التشغيل؛ تظل البيانات المستقلة عن البنية ضمن `/usr/share/argvus/<component>`.

راجع [package Matrix](/docs/reference/package-matrix/) للتعرف على حدود الحزمة المثبتة.
