---
title: مقدمو الخدمات والخدمات
description: حزم الموفر وواجهات النظام الخاصة بهم.
---

تعرض مشاريع الموفر واجهات مركزة لعملاء واجهة المستخدم:

- `argvus network` و`argvus bluetooth` يدمجان NetworkManager ومساعدي Bluetooth؛
- `argvus-display` يدمج حالة الشاشة مع Hyprland؛
- `argvus-power` يدمج إجراءات الخمول والقفل والطاقة؛
- `argvus-firewall` يدير `argvus-firewall.service`؛
- يستهلك المساعد `argvus removable-devices` في `argvus` UDisks2؛
- يستهلك المساعد `argvus control-center` في `argvus` واجهات برمجة تطبيقات الموفر وواجهات D-Bus الخاصة بالنظام.

مركز التحكم هو عميل لهؤلاء المزودين، وليس بديلاً لهم.