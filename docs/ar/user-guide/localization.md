---
title: التوطين
description: استخدم ARGVUS باللغة الإنجليزية أو البرتغالية.
---

`argvus-i18n` يقدم الفهرس المشترك `argvus i18n validate` القيادة QML الوحدات `org.argvus.i18n`. ARGVUS 18 لغة بينية: الإسبانية والإنكليزية والبرتغالية والروسية والروسية والروسية والروسية والأوردية والفييتنامية والصينية المبسطة، والألمانية والإنكليزية والفرنسية والهندية والإنكليزية والإيطالية واليابانية والكورية والبرازيلية.

Native ARGVUS وتستهلك عناصر التطبيقات والقذائف الفهرس المشترك. ولذلك يمكن استخدام الخيط المترجم بأكثر من مستودع واحد. استخدام اللغة العربية والأوردو في مخطط الحق في الانتقال في الدعم GTK.. Quickshell و واجهات طرفية

Use **Control Center  the Preferences  the Language  the Language** to select the ARGVUS لغة الوصل هذا مكتوب `ARGVUS_LANGUAGE` إلى `$XDG_CONFIG_HOME/locale.conf` (عادة) `~/.config/locale.conf`) على سبيل المثال، `ARGVUS_LANGUAGE=pt-PT` يختار البرتغالية الأوروبية. البرتغالية (البرازيل)`pt-BR`خيار منفصل تم تركيب كتالوج اللغة ARGVUSلذا لا يعتمد على `locale-gen`.

The Control Panel restarts automatically after the interface language changes. بدء دورة جديدة لتطبيقها على عناصر الدورة. The separate **Regional Locale** setting writes `LANG` وتحكم الأشكال الإقليمية وتطبيقات الأطراف الثالثة. وهو يتطلب نظاماً محلياً مولداً؛ ويمكّن نظاماً محلياً؛ `/etc/locale.gen` ! `locale-gen` بإذن من المدير تصميم اللوحات الرئيسية، والخيارات وخط الأساس هي أماكن منفصلة.

## عندما تتغلب متغيرات البيئة على التغيير

ARGVUS يستعاض عن عبارة: `ARGVUS_LANGUAGE` من المستعمل `locale.conf`وإذا لم يكن هناك، فإنها تحل المنطقة `LANG` قيمة كتالوج مركب المتغيرات المحلية المدمجة لا تبطل `ARGVUS_LANGUAGE` الأفضل ولا تزال تطبيقات الأطراف الثالثة تستخدم النظام المحلي الإقليمي.

فحص القيم الفعالة بما يلي:

```sh
systemctl --user show-environment | rg '^(LANG|LC_|ARGVUS)='
locale
printf '%s\n' "$LANG"
localectl status
```

تحقق من ذلك `~/.config/locale.conf` يحتوي على صحة `ARGVUS_LANGUAGE=<catalog>` قيمة To change regional formatting or third-party application language, use **Regional Locale** and select a generated locale. ويعاد تشكيل فريق الرقابة بعد إدخال أي من التغييرات على الأفضليات؛ ويُرجى الخروج من القائمة، وإذا كانت عناصر الدورة الأخرى لا تزال تظهر لغة الوصل القديمة. انظر [متغيرات البيئة](/docs/reference/environment-variables/) للعقود التقنية والأسبقية.
