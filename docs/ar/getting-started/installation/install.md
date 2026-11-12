---
title: قم بتثبيت ARGVUS
description: تثبيت ARGVUS وتحديثه وإزالته وتكوينه.
---

يتم توزيع ARGVUS كحزمة Arch Linux من خلال مستودع حزم ARGVUS الرسمي.

## قبل التثبيت

قم بتمكين مستودع Arch Linux `multilib` الرسمي قبل تثبيت ARGVUS. قم بتحرير `/etc/pacman.conf` وقم بإلغاء التعليق على كل من قسم `[multilib]` والسطر `Include` الخاص به:

```ini
[multilib]
Include = /etc/pacman.d/mirrorlist
```

ثم قم بمزامنة قواعد بيانات الحزمة وإجراء ترقية كاملة للنظام:

```sh
sudo pacman -Syu
```

يقوم برنامج التثبيت السريع بتكوين مستودع ARGVUS، لكنه لا يقوم بتمكين `multilib` لك.

## التثبيت السريع

في تثبيت Arch Linux الجديد، يمكنك تثبيت ARGVUS مباشرةً من جهاز TTY دون تثبيت بيئة سطح مكتب أخرى أولاً.

تشغيل:

```sh
curl -fsSL https://argvus.github.io/install.sh | sh
```

المثبت:

1. يقوم باستيراد مفتاح توقيع مستودع ARGVUS والتوقيع عليه محليًا.
2. تكوين مستودع حزمة ARGVUS.
3. يقوم بتحديث قاعدة بيانات الحزمة.
4. تثبيت الحزمة `argvus` وتبعياتها.

بعد التثبيت، قم بتسجيل الخروج وحدد **ARGVUS** من مدير العرض لديك.

> **ملاحظة:** يتطلب برنامج التثبيت السريع `curl`، و`sudo`، وأدوات إدارة الحزم القياسية لـ Arch Linux.

## تكوين المستودع فقط

لتكوين مستودع حزمة ARGVUS دون تثبيت سطح المكتب، قم بتشغيل:

```sh
curl -fsSL https://argvus.github.io/repo-install.sh | sh
```

قم بتثبيت ARGVUS يدويًا بعد ذلك:

```sh
sudo pacman -Syu argvus
```

## التثبيت اليدوي

إذا كنت تفضل تكوين المستودع خطوة بخطوة، فاستخدم الأوامر التالية.

### 1. قم بتنزيل مفتاح توقيع المستودع

```sh
curl -fsSLo /tmp/argvus.gpg https://argvus.github.io/packages/arch/argvus.gpg
```

### 2. قم باستيراد مفتاح التوقيع

```sh
sudo pacman-key --add /tmp/argvus.gpg
```

### 3. قم بالتوقيع على المفتاح محليًا

```sh
ARGVUS_KEY="$(gpg --show-keys --with-colons /tmp/argvus.gpg | grep '^pub:' | head -n1 | cut -d: -f5)"
sudo pacman-key --lsign-key "$ARGVUS_KEY"
```

### 4. قم بتكوين مستودع ARGVUS

```sh
curl -fsSL https://argvus.github.io/packages/arch/argvus.conf \
  | sudo tee /etc/pacman.d/argvus.conf
```

أضف تكوين المستودع إلى `/etc/pacman.conf`:

```sh
echo "Include = /etc/pacman.d/argvus.conf" \
  | sudo tee -a /etc/pacman.conf
```

يعمل تكوين المستودع على تمكين مستودعات ARGVUS، بما في ذلك `argvus` و`argvus-extras`، مع `SigLevel = Required TrustedOnly`. يقبل Pacman الحزم فقط عندما تكون توقيعاتها صالحة وموثوقة من خلال حلقة المفاتيح المحلية.

### 5. قم بتثبيت ARGVUS

```sh
sudo pacman -Syu argvus
```

بعد التثبيت، قم بتسجيل الخروج وحدد **ARGVUS** من مدير العرض لديك.

## تحديث

قم بتحديث ARGVUS مع بقية أنظمة Arch Linux:

```sh
sudo pacman -Syu
```

لتحديث حزمة ARGVUS فقط:

```sh
sudo pacman -S argvus
```

> **التوصية:** توصي Arch Linux عمومًا بترقية النظام بالكامل باستخدام `sudo pacman -Syu` بدلاً من تحديث الحزم الفردية.

## إزالة ARGVUS

قم بإزالة ARGVUS مع الاحتفاظ بالتبعيات التي لا تزال مطلوبة بواسطة الحزم المثبتة الأخرى:

```sh
sudo pacman -R argvus
```

قم بإزالة ARGVUS والتبعيات التي لم تعد مطلوبة بواسطة أي حزمة مثبتة:

```sh
sudo pacman -Rns argvus
```

### قم بإزالة مستودع ARGVUS

إذا كنت تريد أيضًا إزالة مستودع الحزمة، فقم بإزالة التكوين الخاص به:

```sh
sudo rm /etc/pacman.d/argvus.conf
```

ثم قم بإزالة السطر المقابل من `/etc/pacman.conf`:

```text
Include = /etc/pacman.d/argvus.conf
```

يمكنك تعديل الملف باستخدام:

```sh
sudo nano /etc/pacman.conf
```

### قم بإزالة مفتاح التوقيع

لإزالة مفتاح توقيع مستودع ARGVUS من حلقة مفاتيح pacman، قم أولاً بإدراج المفاتيح:

```sh
sudo pacman-key --list
```

ثم قم بإزالة مفتاح ARGVUS بمعرف المفتاح الخاص به:

```sh
sudo pacman-key --delete KEY_ID
```

استبدل `KEY_ID` بمعرف مفتاح التوقيع ARGVUS.

> **تحذير:** قم بإزالة هذا المفتاح فقط عندما تقوم أيضًا بإزالة مستودع ARGVUS. لا تقم بإزالة المفاتيح التي لا تزال مطلوبة من قبل المستودعات الأخرى.

## تكوين المستخدم

لا يؤدي تثبيت الحزمة إلى كتابة التكوين في `$HOME`. يقرأ ARGVUS الإعدادات الافتراضية المجمعة من `/usr/share/argvus` ويعمل دون نسخها إلى دليلك الرئيسي.

يتبع تكوين وقت التشغيل هذه الأولوية العامة:

```text
~/.config/argvus/data/generated/<app>  ->  ~/.config/argvus/data/<component>/<app>  ->  $XDG_CONFIG_HOME/<app>  ->  /usr/share/argvus/<app>  ->  upstream defaults
```

`argvus-config` هو المكون الوحيد الذي يكتب `~/.config/argvus/data/generated/`، وهذا الإسقاط هو الطبقة النشطة: النسخة المُدارة أو الأصلية تصبح سارية المفعول فقط عندما يكون الملف الذي تم إنشاؤه المقابل غائبًا. يقع التكوين المُدار بواسطة ARGVUS ضمن `~/.config/argvus/`. يظل تجاوز `$XDG_CONFIG_HOME/<app>` الاختياري متاحًا للتطبيقات التي تدعم تكوين المستخدم الأصلي.

يعتبر جهاز الإرسال `argvus --setup` اختياريًا وهو مخصص للدليل الصريح
التجاوزات، وليس الانتعاش. إذا تم حذف `~/.config/argvus`، `SUPER + SHIFT + R`
يعيد إنشاء ملف التعريف الأساسي وإنشاء حالة وقت التشغيل من خلال الوضع العادي
إعادة تحميل خط الأنابيب.

```sh
argvus --setup --copy foot-tui
argvus --setup --copy kitty-tui
argvus --setup --copy-all
```

`--copy-all` عبارة عن قائمة مسموح بها صريحة لملفات ملف تعريف TUI المدعومة؛ ذلك
لا يستنسخ `/usr/share/argvus` ولا ينسخ البرامج النصية والخدمات والأصول،
الملفات التي تم إنشاؤها أو `config.json`. قم بتشغيل `argvus --setup --help` لرؤية التيار
خيارات. بمجرد نسخها، تصبح الملفات تجاوزات وحزمًا مملوكة للمستخدم
الترقيات لا تحل محلها.

لاستبدال تكوين منسوخ موجود، استخدم `--force`. يقوم ARGVUS بإنشاء نسخة احتياطية ذات طابع زمني قبل استبدالها:

```sh
argvus --setup --copy <app> --force
```

## تجاوزات مستخدم Hyprland

للحصول على دليل تجاوز Hyprland الكامل، بما في ذلك الفرق بين التكوين الأصلي الكامل وأجزاء ARGVUS التزايدية، راجع [تجاوزات Hyprland](/docs/argvus-hyprland/hyprland-overrides/).

```sh
mkdir -p ~/.config/argvus/data/hypr
nano ~/.config/argvus/data/hypr/user.lua
```

### احتياطي الآلة الافتراضية

قد تواجه بعض الأجهزة الافتراضية مشكلات في العرض مع Hyprland، أو Kitty، أو Quickshell. يكتشف ARGVUS الأجهزة الافتراضية التي تستخدم برنامج تشغيل الرسومات `vmwgfx` ويتيح عرض البرامج عند الضرورة.

إذا لم يعمل الكشف على جهازك الظاهري، فقم بتمكين الإجراء الاحتياطي يدويًا في `user.lua`:

```lua
-- Virtual machine compatibility
hl.env("LIBGL_ALWAYS_SOFTWARE", "1")
```

بعد إنشاء أو تعديل `user.lua`، قم بتسجيل الخروج من Hyprland وقم بتسجيل الدخول مرة أخرى بحيث يتم تطبيق المتغير على التطبيقات التي يتم تشغيلها بواسطة الجلسة.

التحقق من ذلك مع:

```sh
echo $LIBGL_ALWAYS_SOFTWARE
```

الناتج المتوقع هو :

```text
1
```

> **ملاحظة:** `LIBGL_ALWAYS_SOFTWARE=1` يفرض على تطبيقات Mesa/OpenGL استخدام عرض البرامج، عادةً من خلال `llvmpipe`. يمكن أن يقلل من الأداء الرسومي ويجب استخدامه كبديل للأجهزة الافتراضية المتأثرة.
