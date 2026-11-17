---
title: حاسبة ARGVUS
description: حاسبة ARGVUS — GTK4 / ARGVUS
---

# حاسبة ARGVUS

`argvus calculator` · launcher: **حاسبة ARGVUS**

- GTK4 / Relm4; ARGVUS theme (`time`), IBM Plex Mono.
- `+`, `-`, `*`, `/`, `%`, `±`; `50% = 0.5`.
- **علمية**: `sqrt`, `log`, `ln`, `exp`, `sin`, `cos`, `tan`, `asin`, `acos`, `atan`, `pi`, `e`, `^`, `(`, `)`.
- **راديان**: غير محدد: درجات.
- `-2^2 = -4`; `2^3^2 = 512`; `2*pi`; `1.5 = 1,5`.
- Enter: `=`; Escape: **مسح**; Backspace: **حذف حرف**; Tab / Shift+Tab.
- **نسخ النتيجة**; **السجل**: 100; **مسح السجل**.
- IEEE 754 / f64; 15 digits; 4096 bytes; 64 levels.

## السجل

يُحفظ السجل بعد الإغلاق حتى 100 نتيجة. مسح السجل يحذف البيانات المحفوظة أيضًا. الإعدادات: ↑/↓ للتنقل، Enter أو النقر للتفعيل، Escape للرجوع.

`$XDG_DATA_HOME/argvus/calculator/history.json` (`~/.local/share/argvus/calculator/history.json`)

تفتح دائمًا في نافذة عائمة صغيرة في الوسط (370 × 420)؛ يمكن للوضع العلمي تكبيرها.

تحتوي الشاشة الرئيسية على التعبير والنتيجة ولوحة المفاتيح فقط. تتيح القائمة الوضع العلمي ووحدات الزوايا والنسخ وشاشة سجل منفصلة. يعود الرجوع أو Escape إلى الحساب؛ واختيار نتيجة يعيد استخدامها. يعيد الوضع الأساسي النافذة الصغيرة مع الحفاظ على التعبير.

`SUPER + C` → `argvus calculator`

`.desktop`: `/usr/share/applications/argvus-calculator.desktop` · `Name=ARGVUS Calculator` · `Exec=argvus calculator`

ARGVUS: `focus_on_activate = true` (`org.argvus.*`, `sh.argvus.*`, `com.argvus.*`, `argvus-*`, `argvus.*`).

تكبر الأزرار عند تكبير النافذة. يستخدم كل تشغيل جديد الحجم الافتراضي. تحتوي الشاشات الفرعية على سهم رجوع في الأعلى.

`s Settings` → `settings` · Escape / ←

Min: **370 × 420** / **440 × 640** · `0–9`: **16 px**.

`2+2` → Enter / `=` → `4` (input) · `4*3` → Enter → `12`.

يُحفظ الوضع الأساسي أو العلمي المختار فورًا ويُستعاد عند التشغيل التالي.

`$XDG_CONFIG_HOME/argvus/calculator/settings.json` (`~/.config/argvus/calculator/settings.json`).
