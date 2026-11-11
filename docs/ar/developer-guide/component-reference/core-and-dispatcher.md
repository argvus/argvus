---
title: النواة والمرسل
description: مساحة عمل argvus والمرسل الخاص بها وصناديق Rust المشتركة.
---

`argvus`                                                         ﻲﻓ Cargo ﻞﻤﻋ ﺔﺣﺎﺴﻣ ﻮﻫ`de/argvus`                                                             ﺮﻓﻮﺗ ﻲﺘﻟﺍ ﺔﻣﺰﺤﻟﺍﻭ`/usr/bin/argvus`                                                                     ﻕﻭﺪﻨﺼﻟﺍ .`argvus-cli`ﻲﻓ ﺓﺩﻮﺟﻮﻣ ﺔﻠﻣﺎﻜﻟﺍ ﺔﻴﻋﺮﻔﻟﺍ ﺮﻣﺍﻭﻷ﻿ﺍ ﺔﻤﺋﺎﻗ .ﺄﺒﻌﻣ ﻲﺠﻣﺮﺑ ﺺﻧ ﻞﻴﻐﺸﺗ ﻭﺃ ،ﺹﺎﺧ ﺞﻣﺎﻧﺮﺑ ﺬﻴﻔ[                                                               ﺮﻣﺍﻭﻷ﻿ﺍ ﺮﻄﺳ ﻊﺟﺮﻣ](/docs/reference/command-line/).

تحتوي مساحة العمل أيضًا على الصناديق التي تتشاركها الأوامر:

- `argvus-config`: مخزن التكوين الأساسي والكاتب الوحيد للشجرة التي تم إنشاؤها (`data/generated/`). وهي أيضًا مكتبة مستخدمة قيد التشغيل بواسطة `argvus config` و`theme-switch` و`effects-toggle`.
- `argvus-i18n` و`argvus-i18n-core`: الكتالوجات الموجودة في `resources/locales/`، والتحقق من صحتها، ووحدة `org.argvus.i18n` QML.
- `argvus-tui` و`argvus-theme`: واجهة المستخدم الطرفية المشتركة وتحميل السمات.
- `argvus-runtime`: الرسائل المترجمة ومساعدات العملية المشتركة بواسطة الأوامر.

يظل `argvus-session` مستودعًا منفصلاً. فهو يوفر نقاط دخول الجلسة (`session`، `start`، `tty`) ووحدات دورة الحياة، ويصل `argvus sessionctl` إلى وحدة التحكم الخاصة به.