---
title: استكشاف أخطاء بدء تشغيل الجلسة وإصلاحها
description: تشخيص حالات فشل جلسة المستخدم، Hyprland وsystemd.
---

تشغيل:

```sh
argvus sessionctl status
argvus sessionctl logs
systemctl --user status argvus-session.target
```

تحقق من جهاز الترحيب، `argvus start`، Hyprland ووحدة المستخدم ذات الصلة بشكل منفصل. لا يثبت إنشاء الحزمة الناجح أن الحمولة المثبتة أو تجاوز المستخدم النشط حديث.

استخدم الشيكات بهذا الترتيب:

1. تأكد من أن المُرحِّب يستخدم إعداد الترحيب المثبت ARGVUS.
2. اقرأ سجل الجلسة وابحث عن تكوين Hyprland Lua المفقود أو غير الصالح.
3. تحقق من الاستعداد وحالة الهدف باستخدام `systemctl --user status argvus-session.target`.
4. قم بفحص المكون الفاشل بدلاً من إعادة تشغيل كل خدمة مرة واحدة.

إذا كان `argvus-session.target` غير نشط بينما يكون تراكب التحميل نشطًا، فافحص
خطأ الإسقاط قبل إعادة تشغيل الخدمات:

```sh
argvus config project
journalctl --user -u argvus-session.target -u argvus-session-prepare.service -b --no-pager
```

`argvus-config` هو الكاتب الوحيد الموجود أسفل `data/generated/`، لذا فإن `project` هو الكاتب
طريقة موثوقة لإعادة إنتاج جيل أثناء التشخيص. بدء الجلسة
يديرها لأنه لا توجد خطة ملتزمة بها حتى الآن؛ الخطة التي يكتب لها
`${XDG_STATE_HOME:-$HOME/.local/state}/argvus/config-projection.json` هو ما
يتم تطبيق `argvus sessionctl reload` لاحقًا، دون الإسقاط مرة أخرى.

تقوم الجلسة بإيقاف تراكب التحميل عند الإعداد القاتل أو بدء تشغيل الهدف
فشل. يجب على فشل المستهلك البصري إما أن يستخدم احتياطيه الموثق أو أن يكون كذلك
ذكرت صراحة؛ ولا يجب أن تكون مخفية كجلسة ناجحة.

## تسجيل الدخول بطيء

إذا ظهر `greeter.handoff-takeover-failed` بدون `handoff.master-acquired`،
فحص `handoff.master-unavailable card=<name> errno=<n>` لعملية الاستحواذ
خطأ. يمكن للمراقب الذي تم فتحه بينما كان مؤلفًا آخر هو المعلم أن يعود إلى errno
13 (`EACCES`) حتى بعد أن يقوم المنضد بإصدار الملف الرئيسي. يقوم المساعد بإعادة فتح الملف
نفس GPU للحصول على الماجستير ضمنا؛ `handoff.master-reopened` يسجل هذا
يتم إعداد الاسترداد ومخزن إطارات مؤقت جديد على هذا الواصف. يقوم المُرحب بإلغاء المساعد الفاشل (`greeter.handoff-cancelled`)، لذلك
لا يمكن إضافة انتظار آخر قبل أن يبدأ مؤلف المستخدم. جلسة إرسال
`release` بينما عملية الاستحواذ معلقة تحصل على `handoff.release-before-master`:
يقوم المساعد بإغلاق واصفات DRM الخاصة به قبل الإقرار. مؤلف تحية
يعد الحادث في Hyprland / Aquamarine نتيجة منفصلة. إلغاء المساعد يفعل
لا تثبت أنه تم إصلاح مشكلة برنامج التشغيل أو المكون.

يقوم `greeter.vt-switch-returned active=<tty>` بتسجيل VT النشط عند تسجيل الدخول
تبديل إرجاع المكالمات. تسجل مهلة الرسم أيضًا `active-vt=<tty>` في ملفها
خطأ. هذه لقطات وليست دليلاً على اكتمال تسليم إدارة الحقوق الرقمية.

لإجراء مقارنة قبل/بعد، احفظ العمل واستخدم خمس عمليات تسجيل دخول لكل إصدار على
نفس الجهاز والموضوع وإعداد الشاشة. قبل كل تسجيل خروج تالٍ، انسخ
`/run/argvus-greeter/timing.log`، سجل الجلسة ومجلة التمهيد إلى ملف منفصل
نموذج الدليل ضمن `/tmp/argvus/`؛ يحل تشغيل الترحيب التالي محل توقيته
سجل. قم بتسجيل معرف التمهيد مع كل عينة وقارن الطوابع الزمنية خلال ذلك فقط
التمهيد. قياس المصادقة لتسليم الإطار الأول، وإدخال مجمّع الجلسة،
الإطلاق النهائي لـ Hyprland وتوقف التحميل الدوار. قارن بين المتوسط والأسوأ
وقت المصادقة إلى إيقاف الدوران والتحقق بصريًا من وجود إطارات سوداء،
ومضات أو الدوار عالقة. مدة أسطح سطح المكتب وحدها ليست هي المدة
وقت تسجيل الدخول من النهاية إلى النهاية. كرر اختبارات الفشل والاسترداد في جلسة يمكن التخلص منها،
دون إنهاء الجلسة التي تحتوي على عمل غير محفوظ.

تسجل كل خطوة بدء التشغيل مدتها في سجل الجلسة. لمعرفة خطوات آخر تسجيل دخول، الأبطأ أولاً:

```sh
grep timing ~/.local/state/argvus/session.log | tail -30 | sort -k4 -rn
systemd-analyze --user critical-chain argvus-session.target
```

لتحديد وقت التبديل من جهاز الترحيب إلى سطح المكتب، اقرأ علامتي الترحيب والتسليم مع دفتر اليومية. كلهم يستخدمون `CLOCK_MONOTONIC`، لذلك يصطفون:

```sh
cat /run/argvus-greeter/timing.log
journalctl -b -o short-monotonic --since -2min | grep -E 'greetd|Hyprland|coredump|argvus-greeter-handoff|session-loading'
grep monotonic_ns ~/.local/state/argvus/session.log | tail -5
```

يحتفظ `timing.log` بآخر تسجيل دخول فقط: المُرحب وسطح تسليم DRM (`handoff.master-acquired`، `handoff.output-found`، `handoff.first-frame`، `handoff.release-received`، `handoff.replaced-by-compositor`) و علامات `greeter.*`. تسجل خطوط `handoff.crtc` كل تغيير في CRTC أثناء انتظار عملية التسليم لـ DRM الرئيسي؛ السطر الذي يحتوي على `fb=0` أو `mode_valid=false`، أو `handoff.output-found ... framebuffer=0`، يعني أن مُنشئ الترحيب قام بإيقاف تشغيل الإخراج قبل أن تبدأ عملية التسليم، لذلك كان على الشاشة المزامنة مرة أخرى. `greeter.watchdog-stop-requested` يعني أن جلسة الترحيب انتهت دون انتظار منسقها، و`greeter.compositor-exit-deferred` أنه يُطلب من المنضد الخروج بمجرد انتهاء سطح التسليم؛ `greeter.compositor-exit-requested` وحده يعني أنه تمت مطالبته بالخروج مرة واحدة. إذا أظهرت أسطر الترحيب وتسجيل الدخول في المجلة فجوة تبلغ حوالي نصف ثانية بين إغلاق جلسة الترحيب وفتح جلسة المستخدم، فسيتم تشغيل عملية تفكيك منسق الترحيب أثناء تسجيل الدخول. هذا متوقع بينما تظهر العلامة `compositor-in-foreground=true`: يؤدي تعليق محطة الترحيب إلى إنهاء المنضد قبل الخروج المؤجل. يحدث ذلك عندما لا يتمكن المُرحب من مغادرة VT الخاص به قبل خروج المُركب الخاص به: ابحث عن `greeter.vt-switch-requested` متبوعًا بـ `greeter.handoff-drawing`؛وعندما يكون الثاني مفقودًا، يوضح `greeter.handoff-takeover-failed error=<reason>` السبب. `handoff.page-flip` يعني أن عملية التسليم قد حلت محل الإطار الأخير للمرحب بدون مجموعة أوضاع؛ `handoff.page-flip-failed errno=<n>` متبوعًا بـ `handoff.modeset` يعني أن النواة رفضت قلب الصفحة. `handoff.overlay plane=<id>` ثم `handoff.retired` يعني أن التراكب أخفى إطارات Hyprland الأولى حتى ظهر القرص الدوار للجلسة على الشاشة؛ `handoff.overlay-unavailable` يعني أن CRTC ليس لديه مستوى تراكب مجاني، و`handoff.retire-timeout` يعني أن القرص الدوار للجلسة لم يرسل `retire` مطلقًا. يقوم القرص الدوار للجلسة (`argvus-greeter-handoff`) بكتابة `first layer-shell frame callback` و`stop requested` للمجلة.

من المتوقع إدخال `project.theme-switch` أو إدخال `prepare.startup-theme` غير صفري بعد تغيير المظهر أو تغيير ملفات السمة أو تشكيل جديد أو ترحيل ختم بدء تشغيل قديم. تؤدي إعادة تثبيت الملفات المتطابقة إلى الحفاظ على ذاكرة التخزين المؤقت لبدء التشغيل المستندة إلى المحتوى. إذا ظهرت في كل تسجيل دخول، فإن ملفات السمات المادية مفقودة؛ قم بتشغيل `argvus theme-switch <theme>` مرة واحدة.يتم الاحتفاظ بسجل الجلسة تحت `$XDG_STATE_HOME/argvus/session.log` (عادة `~/.local/state/argvus/session.log`). بعد تغيير تجاوز مستخدم Hyprland، قم بتسجيل الخروج والدخول حتى تتحقق نقطة دخول الجلسة من صحتها وتحددها مرة أخرى. راجع [الجلسة الرسومية](/docs/argvus-session/graphical-session/) للتعرف على التدفق الطبيعي لبدء التشغيل.