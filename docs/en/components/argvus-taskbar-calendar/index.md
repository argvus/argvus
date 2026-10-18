---
title: Calendar
description: Use the ARGVUS taskbar calendar.
---

`argvus-taskbar-calendar` is the GTK4 calendar popup opened from the Waybar date module or with:

```sh
argvus --calendar
argvus taskbar-calendar toggle
argvus taskbar-calendar status
```

It supports local events, reminders and ICS import/export. Its user service is `argvus-taskbar-calendar.service`. Use `argvus taskbar-calendar --help` for options supported by the installed version.

The calendar follows the active ARGVUS theme, including Sticky and Float variants. Theme changes are applied through the generated user cache; use `argvus taskbar-calendar reload` if a running popup needs an explicit refresh.

When opened from the taskbar date module, the popup is anchored to the bar edge: it opens below a top taskbar and above a bottom taskbar, aligned to the date module's right edge. Direct launches without taskbar geometry use the pointer or monitor fallback.
