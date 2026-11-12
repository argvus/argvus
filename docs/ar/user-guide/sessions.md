---
title: الجلسات
description: بدء وإدارة جلسة ARGVUS الرسومية.
---

يمتلك `argvus-session` نقاط دخول الجلسة ودورة حياة `systemd --user`. يبدأ Hyprland من خلال `argvus start`، ويستورد بيئة الجلسة ويدير هدف ARGVUS.

- [الجلسة الرسومية](/docs/argvus-session/graphical-session/)
- [جلسة TTY](/docs/argvus-session/tty-session/)
- [مرحب](/docs/components/argvus-greeter/)

التشخيصات المفيدة:

```sh
argvus sessionctl status
argvus sessionctl logs
argvus sessionctl restart waybar
```

راجع المطور [دورة حياة وقت التشغيل](/docs/developer-guide/architecture/runtime-lifecycle/) لمعرفة ملكية الخدمة.
