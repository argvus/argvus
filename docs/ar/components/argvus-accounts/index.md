---
title: Accounts
description: ملفات تعريف المستخدمين المحلية، وأسماء العرض، والصور الرمزية، وعضوية المجموعة، من خلال حسابات argvus.
---

يقوم `argvus accounts` بسرد وتحرير حسابات المستخدمين المحليين لسطح مكتب ARGVUS. إنها خاصة
القابل للتنفيذ لحزمة `argvus`، المثبتة في `/usr/lib/argvus/commands/accounts`. ال
يبدأ الأمر `argvus`، ولا يتم إلا العمليات التي تغير الحسابات الأخرى أو بيانات النظام
تشغيل كجذر.

##الأوامر

```text
argvus accounts list [--all]                   list users (--all includes system accounts)
argvus accounts show USER                      show account details
argvus accounts name USER "DISPLAY NAME"       change another user's display name
argvus accounts avatar USER IMAGE              set another user's avatar
argvus accounts avatar USER --remove           remove another user's avatar
argvus accounts groups USER                    list groups
argvus accounts groups USER --add G --remove G change groups (authorization required)
argvus accounts passwd USER                    change a password (reads three lines from stdin)
argvus accounts self                           show your own account
argvus accounts self name "DISPLAY NAME"       change your own display name
argvus accounts self avatar IMAGE              set your avatar
argvus accounts self avatar --remove           remove your avatar
argvus accounts self groups                    list your groups
argvus accounts --verbose ...                  print diagnostic information to stderr
```

## عرض الأسماء والصور الرمزية

يتم تخزين اسم العرض في حقل GECOS القياسي. لا يتم تغيير اسم مستخدم Unix أبدًا.

الصورة الرمزية هي صورة بتنسيق PNG أو JPEG أو WebP، يصل حجمها إلى 20 ميجابايت. تم تغيير حجم الصورة إلى 256×256
بكسل وحفظها بتنسيق PNG باستخدام الوضع `0644` في `~/.face`، الموقع المجاني لسطح المكتب الذي تمت قراءته بواسطة
مرحبًا ومن قبل معظم مديري العرض. تتم كتابة الصورة الرمزية الخاصة بك داخل الدليل الرئيسي الخاص بك و
لا يتطلب إذن.

##التفويض

فقط `list`، و`show`، و`self`، و`self groups`، والصورة الرمزية الخاصة بك هي التي تعمل بدون ترخيص. كل
تغيير آخر يطلب من وكيل polkit لسطح المكتب الحصول على إذن. قاعدة ARGVUS
(`/usr/share/polkit-1/rules.d/50-argvus-accounts.rules`) يقرر من يجب عليه المصادقة:

- يؤكد مستخدم في مجموعة المسؤولين (`wheel`، أو Arch الافتراضي، أو `sudo`) بنفسه
  كلمة المرور؛
- يقوم أي مستخدم آخر بالتأكيد باستخدام كلمة مرور الجذر.

| عملية | بدون إذن | بإذن |
| --- | --- | --- |
| قائمة المستخدمين، إظهار الحساب، قائمة المجموعات | ✓ | ✓ |
| تغيير الصورة الرمزية الخاصة بك | ✓ | ✓ |
| قم بتغيير اسم العرض الخاص بك | — | ✓ |
| قم بتغيير كلمة المرور الخاصة بك | — | ✓ (بعد إثبات كلمة المرور الحالية) |
| قم بتغيير اسم مستخدم آخر أو الصورة الرمزية أو المجموعات أو كلمة المرور | — | ✓ |

يؤدي تغيير كلمة المرور الخاصة بك أولاً إلى التحقق من كلمة المرور الحالية. تم رفض كلمة مرور خاطئة
قبل ظهور مطالبة التفويض.

ليست هناك حاجة لتشغيل الأمر باستخدام `sudo`. يطلب الأمر من polkit الحصول على إذن متى
يحتاجها.

## الحدود الأمنية

- `passwd` يقرأ كلمات المرور الحالية والجديدة وكلمات المرور الخاصة بالتأكيد من الإدخال القياسي، واحدة لكل سطر،
  ولا تتخذها أبدًا كحجج. قد يكون سطر كلمة المرور الحالي فارغًا عندما يكون مسؤولاً
  يعيد تعيين كلمة المرور لمستخدم آخر. مثال:

  ```sh
  printf '%s\n%s\n%s\n' "$OLD" "$NEW" "$NEW" | argvus accounts passwd ghost
  ```

  بشكل تفاعلي، اكتب الأسطر الثلاثة واضغط على Enter بعد كل سطر. يظهر الإدخال على
  المحطة كما هو مكتوب.
- تم رفض تحرير حساب أقل من UID 1000 (و `nobody`)، وبالتالي يتم قفله وحذفه
  وانتهاء صلاحية كلمة المرور الخاصة به. لا يمكن قفل حسابك الخاص أو حذفه.
- إعادة تسمية أو حذف مجموعة أقل من GID 1000، أو مجموعة المسؤولين (`wheel`، `sudo`)، هي
  رفض؛ لا يزال من الممكن تغيير أعضائها، باستثناء أنه لا يمكنك إزالة نفسك من ملف
  مجموعة المسؤول.
- عضوية المسؤول هي مجموعة `wheel` أو `sudo`. يضيف مركز التحكم الجديد
  المسؤولين إلى `wheel` عندما يكون النظام موجودًا، وإلى `sudo` بخلاف ذلك. إزالة الخاصة بك
  تم رفض عضوية المسؤول من خلال التعديل.

## أمثلة

```sh
# Set your display name and avatar after the first login
argvus accounts self name "William Canin"
argvus accounts self avatar ~/Pictures/profile.png

# Check your groups, then add another user to the wheel group (authorization required)
argvus accounts self groups
argvus accounts groups ghost --add wheel

# Administrator: show a user's details and reset that user's name
argvus accounts show ghost
argvus accounts name ghost "Ghost Account"
```

## استكشاف الأخطاء وإصلاحها

- `pkexec was not found`: قم بتثبيت `polkit`. تعتمد الحزمة `argvus` عليها.
- لا تظهر مطالبة التفويض: تأكد من تشغيل وكيل مصادقة polkit
  في جلستك. تعتمد الحزمة `argvus` على `hyprpolkitagent`، الذي يوفر واحدًا.
- أخطاء `unix_chkpwd` عند تغيير كلمة المرور: توفر حزمة `pam` المساعد.
- قم بتشغيل `argvus accounts --verbose ...` لطباعة خطوات الأمر إلى stderr.

للحصول على ملاحظات المطور حول حدود الامتياز، راجع `docs/internal/argvus-accounts/`.عندما يقوم المسؤول بتأمين أو إزالة الحساب الذي تم تكوينه لتسجيل الدخول التلقائي، يقوم موفر الحساب أولاً بتعطيل إدخال تسجيل الدخول التلقائي الذي تم الترحيب به. إذا فشلت عملية التنظيف هذه، فسيتم رفض عملية الحساب. إذا فشل القفل أو الإزالة لاحقًا، فسيظل تسجيل الدخول التلقائي معطلاً. يتم الاحتفاظ بتسجيل الدخول التلقائي لحساب آخر؛ فتح الحساب لا يؤدي إلى تمكين تسجيل الدخول التلقائي. لا يتم إغلاق الجلسات الحالية ولا يتم إعادة تشغيل الترحيب.