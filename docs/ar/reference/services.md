---
title: Services
description: ARGVUS نظام وخدمات المستخدم.
---

هدف المستخدم الرئيسي هو `argvus-session.target`. تشمل خدمات المستخدم الشائعة `argvus-taskbar.service`، `argvus-control-panel.service`، `argvus-widget-telemetry.service`، `argvus-hyprland.service`، `argvus-dunst.service`، `argvus-wallpaper.service`، `argvus-hypridle.service`، خدمات الحافظة و`argvus-session-loading.service`.

جدار الحماية عبارة عن خدمة نظام تسمى `argvus-firewall.service`. تقوم حزمة التقويم بتثبيت `argvus-taskbar-calendar.service` كوحدة مستخدم، ويقوم `argvus-clock.service` (معطل افتراضيًا) بتشغيل ARGVUS Clock المخفية بعد تسجيل الدخول عندما يكون **البدء بالجلسة** قيد التشغيل.

فحص الوحدات النشطة مع:

```sh
systemctl --user status argvus-session.target
systemctl --user list-units 'argvus-*.service'
```

## الوحدات المملوكة للجلسة

تقوم حزمة الجلسة بتثبيت وحدات المستخدم هذه ضمن `/usr/lib/systemd/user` (توجد الحمولة المصدر ضمن شجرة التكوين المشتركة لحزمة الجلسة):

| الوحدة | الدور |
| --- | --- |
| `argvus-session.target` | حدود مدى الحياة لجلسة ARGVUS الرسومية. |
| `argvus-session-prepare.service` | إعداد بدء التشغيل بلقطة واحدة والتكوين الذي تم إنشاؤه. |
| `argvus-control-panel.service` | لوحة تحكم كويك شيل. |
| `argvus-taskbar.service` | شريط المهام الرئيسي ARGVUS. |
| `argvus-widget-telemetry.service` | سطح اختياري للقياس عن بعد/معلومات النظام. |
| `argvus-hyprland.service` | وحدة طلقة واحدة عند الطلب تقوم بتشغيل `hyprctl reload`. |
| `argvus-config.service` | مشاريع التكوين الأساسي (`argvus config project`)؛ يعمل إعادة التحميل على `argvus sessionctl reload-config-transaction`. |
| `argvus-session-reload.service` | إعادة تحميل تسجيل الدخول (`argvus sessionctl reload-desktop --login`): يتم إعادة تحميل Hyprland فقط عندما يؤثر العرض عليه. |
| `argvus-ssh-agent.service` | وكيل SSH للجلسة. |
| `argvus-secret-service.service` | يبدأ الخدمة السرية من خلال `argvus sessionctl start-secret-service`. |
| `argvus-wallpaper.service` | خلفية خلفية نشطة. |
| `argvus-dunst.service` | شيطان الإخطار دونست. |
| `argvus-hypridle.service` | سياسة الخمول والقفل. |
| `argvus-clipboard-text.service` / `argvus-clipboard-image.service` | مراقبو التاريخ الحافظة. |
| `argvus-keyboard-layout.service` | إشعارات تخطيط لوحة المفاتيح. |
| `argvus-polkit.service` | تكامل وكيل PolicyKit. |
| `argvus-session-loading.service` | تراكب تحميل الجلسة من عملية تسليم المُركب حتى بدء مكونات سطح المكتب. |

يتم أيضًا شحن `argvus-blueman-applet.service` بواسطة حزمة الجلسة. بعض الوحدات اختيارية أو بدأت بشكل مشروط؛ فهم ليسوا جميعًا نشطين في كل جلسة.

لفحص التعريف الفعال للوحدة بدلاً من التحقق من مصدرها فقط:

```sh
systemctl --user cat argvus-session.target
systemctl --user status argvus-session.target
journalctl --user -u argvus-session.target -u 'argvus-*.service'
```

ينتمي `argvus-firewall.service` على مستوى النظام إلى `argvus-firewall`. يتم شحن حزمة التقويم بشكل منفصل `argvus-taskbar-calendar.service` كوحدة مستخدم. `argvus-clock.service` هي وحدة مستخدم لحزمة `argvus`، `WantedBy=graphical-session.target`، تم تمكينها من إعدادات ARGVUS Clock.

## أوامر إدارة الجلسة

يقوم الأمر `argvus sessionctl` بإدارة دورة حياة الجلسة ويمكنه إعادة تشغيل مكونات محددة دون تسجيل الخروج:

### دورة حياة الجلسة

```sh
argvus sessionctl start               # Start a new session
argvus sessionctl stop                # Stop the current session
argvus sessionctl restart             # Restart the entire session
argvus sessionctl reload              # Reload configuration and active services
argvus sessionctl status              # Show session status
argvus sessionctl logs                # Display session logs
```

### عمليات إعادة التشغيل الخاصة بالمكونات

أعد تشغيل الخدمات الفردية دون التأثير على الجلسة بأكملها:

```sh
# Desktop surfaces
argvus sessionctl restart waybar
argvus sessionctl restart wallpaper
argvus sessionctl restart shell

# Notifications and switcher
argvus sessionctl restart dunst

# Clipboard and input
argvus sessionctl restart clipboard keyboard-layout

# Bluetooth applet (optional)
argvus sessionctl restart blueman-applet
```

### مزامنة البيئة

استيراد متغيرات البيئة إلى الجلسة:

```sh
argvus sessionctl import-environment
```

يؤدي هذا إلى مزامنة متغيرات Wayland وHyprland وQt والمؤشر وDBus بين خدمات مستخدم Shell وsystemd.

## مرجع وحدات الخدمة

يمكن إدارة وحدات المستخدم الإضافية مباشرةً باستخدام `systemctl --user`:

```sh
systemctl --user status argvus-session.target
systemctl --user start argvus-session.target
systemctl --user stop argvus-session.target
systemctl --user restart <unit>

# View logs for a specific service
journalctl --user -u argvus-wallpaper.service -b -f
```

يتم تخزين سجلات جلسة التمهيد الأولية في:

```
${XDG_STATE_HOME:-$HOME/.local/state}/argvus/session.log
```
