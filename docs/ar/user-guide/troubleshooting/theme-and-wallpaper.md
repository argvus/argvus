---
title: استكشاف أخطاء السمات وخلفية الشاشة وإصلاحها
description: تشخيص انتشار السمة وملفات المظهر التي تم إنشاؤها.
---

تحقق من قسم `appearance` القانوني في `$XDG_CONFIG_HOME/argvus/config.json`، ثم افحص الملفات التي تم إنشاؤها ضمن `$XDG_CONFIG_HOME/argvus/data/generated`. ملفات النقاط القديمة هي مدخلات ترحيل فقط.

`argvus-config` هو الكاتب الوحيد تحت `data/generated/`. لاستعادة كسر
السمة أو ورق الحائط، قم بإعادة بناء الجيل بأكمله من المستند الأساسي و
قم بتطبيق النتيجة بدلاً من تحرير المستهلكين الذين تم إنشاؤهم مباشرة:

```sh
argvus config validate
argvus config apply
argvus sessionctl reload
```

`argvus theme-switch` (الذي يقوم بتشغيل `argvus config theme-apply`) و`argvus config accent`
هم منسقون: يقومون بالتحقق من الصحة، والقفل، وإظهار البداية، والالتزام
`argvus-config` ثم قم بتسوية المستهلكين الخارجيين الذين لا يستطيعون القراءة
`config.json`. لم يكتبوا أبدًا `data/generated/`، ومعاملة السمة تفعل ذلك
عدم كتابة ملفات GTK أو التخزين أو `mode.css` من خلال برامج نصية منفصلة.
استخدم `argvus config accent --apply` لإعادة تشغيل تسوية التمييز فقط.

تطبق تغييرات السمات الرسمية قيم المظهر المحفوظة الخاصة بها وتقوم بتحديث عملاء وقت التشغيل المتأثرين. يؤدي تطبيق سمة مخصصة إلى إعادة تشغيل شريط المهام وإعادة تحميل Hyprland وورق الحائط وخدمات لوحة التحكم. يؤدي التبديل بين الوضع الثابت أو المتباعد إلى إعادة تطبيق التباعد والحدود وهوامش شريط المهام لهذا الوضع، ثم إعادة تشغيل شريط المهام والقياس عن بعد وخدمات إعادة تحميل Hyprland. تعد ألوان إشعارات Dunst هي الاستثناء عندما يكون وضع "عدم الإزعاج" نشطًا: حيث يتم تأجيل إعادة التحميل حتى يتم إيقاف تشغيل DND. إذا احتفظ سطح آخر بالألوان القديمة، فتأكد من أن بنية السمة نشطة ومن أن السمة هي التي اخترتها، ثم قم بتشغيل عمليات إعادة التحميل مرة أخرى:

```sh
cat "${XDG_CONFIG_HOME:-$HOME/.config}/argvus/data/generated/theme/current/build.toml"
argvus config theme-runtime consumers
```

يأتي كل ملف ذو سمة من الإصدار: يتم رفض السمة غير المثبتة (لا يوجد `theme.toml` ضمن `/usr/share/argvus/themes/` أو `~/.local/share/argvus/themes/`) أو التي لا يتم عرضها، ويبقى السمة السابقة. يعرض `argvus theme check DIRECTORY` القالب والسطر الخاص بكل خطأ في العرض. يقوم شريط Waybar بإعادة تحميل ألوانه فقط باستخدام `reload_style_on_change` في JSONC الخاص به. إذا بدا المفتاح عالقًا، فتحقق مما إذا كان لا يزال قيد التشغيل؛ يسجل نفسه في `theme-switch.active` بجانب قفله:

```sh
cat "${XDG_CACHE_HOME:-$HOME/.cache}/argvus/theme-switch.active"
ps -ef | grep -E 'theme-switch|theme-apply'
```

إن وجود معرف الهوية (PID) الموجود في العلامة والذي لا يزال حيًا يعني أن التبديل لم ينته بعد. ينتظر المفتاح 60 ثانية على الأكثر لقفل السمة وللعرض، و5 ثوانٍ لأوامر إعادة التحميل؛ العلامة التي انتهت عمليتها قديمة ويتجاهلها المفتاح التالي. قم بتضمين مخرجات الأوامر أعلاه عند الإبلاغ عن المشكلة.

إذا كان مُرحب تسجيل الدخول المسبق باللون الأبيض فقط أو يستخدم السمة الاحتياطية، فقارن المصدر الخاص بإسقاطه العام:

```sh
uid=$(id -u)
argvus config get /appearance/theme --raw
sudo cat "/var/lib/argvus/greeter/themes/$uid"
sudo cat "/var/lib/argvus/greeter/themes/$uid.accent" 2>/dev/null || true
```

قم بإصلاح الإسقاط دون تغيير التكوين المرحب به:

```sh
sudo argvus greeter-setup --sync-themes
```

يتم نشر هذا الملف للمصادقة المسبقة بواسطة منسق المظهر و
يعيش خارج `data/generated/`، لذا فإن `argvus config apply` العادي لا يعيش
تحديثه.

إذا أبلغ المُرحب عن `Invalid color name: 'null'` في ذاكرة التخزين المؤقت لـ Kitty، فقم بتحديث
`argvus terminal` وأعد إنشاء ذاكرة التخزين المؤقت عن طريق فتح أداة الترحيب مرة أخرى.
اللكنة الأساسية الغائبة هي اللون الافتراضي، وليس لون كيتي؛ المحطة الحالية
يرفض المشغل JSON `null` والقيم القديمة غير الصالحة قبل الكتابة
`active_tab_foreground`.

إذا توقفت الوحدة الطرفية الافتراضية عن الفتح بعد عدة تغييرات في السمة، فافحص ذاكرة التخزين المؤقت المشتقة من Kitty بدلاً من تحريرها يدويًا:

```sh
argvus config get /appearance/theme --raw
ls -l "${XDG_CACHE_HOME:-$HOME/.cache}/argvus/argvus-terminal"
argvus terminal --apply
argvus terminal --print-config
```

يقرأ الأمر أعلاه التفضيلات الأساسية. `.active-theme` هو فقط
إسقاط التوافق يتم إعادة إنشاء ذاكرة التخزين المؤقت بواسطة `argvus terminal` و
متزامنة مع تطبيق الموضوع. مفقودة أو قديمة
يشير `/usr/lib/argvus/loading-theme/splash` إلى تحديث الحزمة غير الكامل؛
قم بإعادة بناء الحزم المنسقة وتثبيتها قبل الحكم على الجلسة المباشرة
نتيجة.

إذا احتفظت حدود النافذة بلون السمة السابق، فلا تقم بتحرير Hyprland's Lua
الملف يدويا. يتم عرض تغييرات السمة واللكنة من `config.json`، إذنيقوم `argvus sessionctl reload` بإعادة تحميل Hyprland وإعادة تطبيق العناصر النشطة وغير النشطة،
وتجميع ألوان الحدود خلال دورة حياة الجلسة:

```text
config.json → projection plan → Hyprland reload → hyprctl reapply of active/inactive/group borders
```

يتم تغيير *تفضيلات* الحدود من خلال `argvus config borders` (أو
`borders-switch.sh` مندوب) في مركز التحكم ولوحة التحكم؛ إعادة تحميل
يعيد تطبيق الألوان المخصصة خلال دورة حياة الجلسة ولا يفعل ذلك
إعادة استخلاصها.

فحص الخطة وقيم المكون الفعالة:

```sh
jq . "${XDG_STATE_HOME:-$HOME/.local/state}/argvus/config-projection.json"
hyprctl -j getoption general:col.active_border
hyprctl -j getoption general:col.inactive_border
argvus sessionctl reload
```

إذا لم يتمكن `hyprctl` من الوصول إلى مقبس Hyprland الحالي، فسيكون إسقاط الحدود
تم الإبلاغ عنها على أنها مكونات فاشلة ولا تعتبر مكونات تابعة ناجحة
إعادة تحميل. تحقق من دفتر يومية جلسة المستخدم قبل إعادة المحاولة.