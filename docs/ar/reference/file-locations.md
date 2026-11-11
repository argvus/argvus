---
title: مواقع الملفات
description: ملفات وأدلة ARGVUS الهامة المثبتة.
---

- `/usr/bin/argvus*` — الأوامر العامة.
- `/usr/share/argvus/hyprland/` — تكوين Hyprland والمساعدين.
- `/usr/share/argvus/control-panel/` — لوحة التحكم QML والبرامج النصية.
- `/usr/share/argvus/taskbar/` — تكوين شريط المهام والبرامج النصية.
- `/usr/share/argvus/lock/` — تكوين Hyprlock.
- `/usr/share/argvus/portal/` — ملفات تكامل البوابة الإلكترونية.
- `/usr/share/backgrounds/argvus/` — أصول ورق الحائط.
- `/etc/argvus/` — تكوين موفر النظام.
- `/etc/argvus/greeter.toml` — تكوين الترحيب.
- `/etc/argvus/removable-devices/config.json` — إعدادات النظام الافتراضية للتخزين القابل للإزالة.
- `$XDG_CONFIG_HOME/argvus/config.json` — التفضيلات المحمولة الأساسية.
- `$XDG_CONFIG_HOME/argvus/data/` - بيانات المكونات المُدارة والنسخ الاحتياطية والأقفال والمخرجات التي تم إنشاؤها.
- `$XDG_CONFIG_HOME/waybar/argvus-taskbar.{jsonc,css}` — تجاوزات شريط المهام الأصلية الكاملة الاختيارية لشريط الطريق.
- `$XDG_CONFIG_HOME/argvus/data/taskbar/argvus-taskbar.{jsonc,css}` - نسخ شريط المهام ARGVUS المُدارة.
- `$XDG_CONFIG_HOME/argvus/data/generated/waybar/` - ملفات تعريف شريط الطريق التي تم إنشاؤها، بما في ذلك `argvus-taskbar.css` و`argvus-widget-telemetry.{jsonc,css}`؛ هذه لها الأسبقية على النسخ أعلاه.
- `$XDG_CONFIG_HOME/argvus/data/generated/waybar/argvus-widget-telemetry.{jsonc,css}` — تنعكس أيضًا تحت `data/waybar/`، والذي تم استبداله من الإعداد الافتراضي المعبأ عند تغيير المظهر ثم تمت إعادة كتابة `ARGVUS_TELEMETRY_*` وكتل الخطوط الخاصة به.
- `$XDG_CONFIG_HOME/argvus/data/generated/yazi/` — شجرة تكوينات Yazi المتوقعة، مع خيار احتياطي مدرك للمتغيرات للموضوعات التي لا تحمل أي نكهة Yazi.
- `$XDG_CONFIG_HOME/argvus/data/generated/superfile/` - مجموعة سمات Superfile المتوقعة.
- `$XDG_CONFIG_HOME/argvus/data/generated/terminal/` — التشكيلات الطرفية المتوقعة.
- `$XDG_CONFIG_HOME/argvus/data/generated/qt6ct/` - لوحة Qt6ct المتوقعة.
- `$XDG_CONFIG_HOME/argvus/data/generated/gtk/` - ملفات سمات GTK المتوقعة.
- `$XDG_CONFIG_HOME/argvus/data/rofi/` - نسخ المستخدم من تكوين Rofi وموضوعاته ووضعه. يقوم ARGVUS بإنشاء كل نسخة عند تغيير الملف لأول مرة؛ تتم قراءة الملفات التي لم تتغير من `/usr/share/argvus/launcher/config/`.
- `$XDG_CONFIG_HOME/argvus/data/generated/removable-devices/theme.css` - ورقة أنماط الأجهزة القابلة للإزالة المتوقعة.
- `$XDG_CONFIG_HOME/argvus/data/focus/settings.json`، `$XDG_DATA_HOME/argvus/focus/history.json`، `$XDG_STATE_HOME/argvus/focus/session.json` - إعدادات التركيز ARGVUS وسجل الجلسة ونقطة تفتيش الجلسة الحالية.
- `$XDG_RUNTIME_DIR/argvus/focus/status.json` - حالة التركيز ARGVUS سريعة الزوال التي يستهلكها شريط المهام؛ تتم إزالته عند خروج التركيز.
- `$XDG_CONFIG_HOME/argvus/data/clock/settings.json`, `$XDG_DATA_HOME/argvus/clock/alarms.json`, `$XDG_STATE_HOME/argvus/clock/timers.json` — إعدادات ساعة ARGVUS والساعات العالمية، والمنبهات، ونقطة تفتيش المؤقت/ساعة الإيقاف.
- `$XDG_CONFIG_HOME/argvus/data/generated/theme/current/time/theme.css` - ألوان تركيز ARGVUS وساعة ARGVUS (مستهلك السمة `time`).
- `/usr/share/argvus/sounds/` - أصوات تركيز ARGVUS وساعة ARGVUS (`argvus-chime.oga`، `argvus-beacon.oga`، `argvus-bell.oga`).
- `$XDG_CONFIG_HOME/argvus/data/generated/theme/` - تصميمات السمة (`builds/<id>-<variant>-<hash>/`)، مكتوبة بواسطة `argvus theme render` ومن خلال كل تبديل سمة وتغيير لهجة، والارتباط الرمزي `current` لبناء السمة النشطة.
- `$XDG_CONFIG_HOME/argvus/data/hypr/` — تجاوز المستخدم Hyprland Lua وإسقاطات Hyprland الأصلية.
- `$XDG_CONFIG_HOME/hypr/hyprland.lua` - تكوين Hyprland الأصلي الكامل الاختياري المحدد بواسطة `argvus start`؛ إنه ليس تراكبًا متزايدًا.
- `$XDG_STATE_HOME/argvus/config-projection.json` - بيان الإسقاط: الإنشاء، وتجزئة التكوين الفعال، والأقسام المتغيرة، والعمل في وقت التشغيل المعلق. تمت قراءته بواسطة `argvus sessionctl reload`؛ أبدا مصدر الحقيقة.
- `$XDG_STATE_HOME/argvus/widget-telemetry-blocks` — تفضيلات كتلة القياس عن بعد القديمة. مدخلات الهجرة فقط؛ القيم الأساسية موجودة في `config.json` ولم يعد هذا الملف مكتوبًا.
- `$XDG_STATE_HOME/argvus/session.log` - سجل الجلسة الرسومية (عادة `~/.local/state/argvus/session.log`).
- `/usr/share/wayland-sessions/argvus.desktop` — دخول الجلسة الرسومية.
- `/usr/share/plymouth/themes/argvus/` - أصول البداية المعبأة.

الحمولات الدقيقة خاصة بالحزمة؛ راجع مرجع مكون المطور وبيان الحزمة ذات الصلة.