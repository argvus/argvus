---
title: متغيرات البيئة
description: عقود البيئة المستخدمة بواسطة مكونات جلسة ARGVUS.
---

تنشئ الجلسة تكامل بيئة XDG وWayland وقد تقوم بتصدير متغيرات توافق المحاكاة الافتراضية عند اكتشافها. يتم استخدام `ARGVUS_CONFIG_HOME` و`ARGVUS_SYSTEM_CONFIG` و`ARGVUS_BOOTSTRAP` بواسطة مساعدي الجلسة والمرسل لحل جذور التكوين.

تعامل مع متغيرات البيئة كعقود وقت تشغيل: تحقق من `--help` أو مصدر المساعد المثبت قبل إضافة تجاوز مستمر.

تعيش الحالة المشتقة تحت جذر الحالة بدلاً من جذر التكوين.
`XDG_STATE_HOME` (عادة `~/.local/state`) يتلقى `argvus/config-projection.json`،
بيان الإسقاط `argvus-config` يعيد البناء و`argvus sessionctl reload`
ينطبق، بالإضافة إلى `argvus/session.log`. ولا مصدر للحقيقة.

## بيئة اللغة

تفضيل اللغة العام الوحيد هو `LANG` في `$XDG_CONFIG_HOME/locale.conf` (`~/.config/locale.conf` الافتراضي). بدون قيمة صالحة، يحدد ARGVUS اللغة الإنجليزية (`en-US`)، حتى عندما يقوم `/etc/locale.conf` أو PAM أو مدير تسجيل الدخول بتوفير اللغة البرتغالية. تتم تسوية القيم مثل `pt_BR.UTF-8` إلى `pt-BR` للكتالوجات.

تطبق الجلسة هذا التفضيل قبل بدء تشغيل مكونات سطح المكتب واستيراد مستخدم systemd وبيئات تنشيط D-Bus. فهو يستبدل `LANG` و`LC_MESSAGES` و`LANGUAGE` الموروث ويمسح `LC_ALL`؛ يتم الاحتفاظ بإعدادات `LC_*` الإقليمية في نفس الملف. تستخدم اللغة الإنجليزية `en_US.UTF-8` عند إنشائها، وإلا `C.UTF-8`. لا يتم إنشاء ملف التفضيل تلقائيًا. قم بتسجيل الخروج ثم الدخول مرة أخرى بعد تغيير اللغة.

يتم تجاهل البيانات القديمة `session.language` و`argvus/data/language` و`argvus/data/internal/language`. لم يتم استيرادهم أو تجديدهم؛ يتم الاحتفاظ بالملفات الموجودة. يرفض `argvus config set` و`patch` عمليات الكتابة الجديدة إلى `session.language` ويوجهان المتصلين إلى `locale.conf`.

للتشخيص، قارن `argvus i18n locale` و`cat "${XDG_CONFIG_HOME:-$HOME/.config}/locale.conf"` و`systemctl --user show-environment`. يصف `localectl status` النظام، وليس لغة ARGVUS الخاصة بالمستخدم. `ARGVUS_I18N_DIR` يقوم بتحديد دليل الكتالوج للتطوير، وليس اللغة.