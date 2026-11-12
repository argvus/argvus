---
title: سیشنز
description: ایک ARGVUS گرافیکل سیشن شروع کریں اور اس کا نظم کریں۔
---

`argvus-session ` سیشن انٹری پوائنٹس اور ` systemd --user ` لائف سائیکل کا مالک ہے۔ یہ ` argvus start` کے ذریعے Hyprland شروع کرتا ہے، سیشن کے ماحول کو درآمد کرتا ہے اور ARGVUS ہدف کا انتظام کرتا ہے۔

- [گرافیکل سیشن](/docs/argvus-session/graphical-session/)
- [TTY سیشن](/docs/argvus-session/tty-session/)
- [گریٹر](/docs/components/argvus-greeter/)

مفید تشخیص:

```sh
argvus sessionctl status
argvus sessionctl logs
argvus sessionctl restart waybar
```

سروس کی ملکیت کے لیے ڈیولپر [رن ٹائم لائف سائیکل](/docs/developer-guide/architecture/runtime-lifecycle/) دیکھیں۔
