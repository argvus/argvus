---
title: الجلسة الأولى
description: قم بتشغيل ARGVUS بعد التثبيت.
---

لتسجيل الدخول الرسومي، قم بتكوين التكامل المرحب به مع `argvus greeter-setup --help` واستخدم تدفق مدير العرض الموضح في [الجلسات](/docs/user-guide/sessions/).

بالنسبة لجلسة TTY، قم بتسجيل الدخول على محطة افتراضية وقم بتشغيل:

```sh
argvus --start-desktop
```

بعد تسجيل الدخول، استخدم `argvus sessionctl status` لفحص هدف مستخدم ARGVUS. إذا لم يبدأ المُركب أو الصدفة، فاتبع [استكشاف أخطاء الجلسة وإصلاحها](/docs/user-guide/troubleshooting/session-startup/).