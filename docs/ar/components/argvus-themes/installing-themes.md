---
title: تثبيت السمات
description: قم بتثبيت حزم سمات ARGVUS الرسمية باستخدام pacman.
---

يتم نشر السمات الرسمية كحزم Arch Linux. كل سمة لها حزمة خاصة بها تسمى `argvus-theme-<THEME_NAME>`.

## أين يمكن الحصول على المواضيع

مصدر كل موضوع رسمي موجود في منظمة مشروع ARGVUS:

- كل سمة عبارة عن مستودع باسم `argvus-theme-<THEME_NAME>` (على سبيل المثال `argvus-theme-dracula`)، تم إنشاؤه كحزمة بنفس الاسم.

تعتمد حزم السمات على `argvus-appearance`. قم بتثبيت ARGVUS أولاً، كما هو موضح في [التثبيت](/docs/getting-started/installation/)، بحيث يكون مكون المظهر موجودًا.

## تثبيت سمة واحدة

لتثبيت كل قالب رسمي مرة واحدة، قم بتثبيت الحزمة التعريفية `argvus-themes`:

```bash
sudo pacman -S argvus-themes
```

يعتمد ذلك على جميع حزم `argvus-theme-*` المنشورة. ARGVUS داكن و ARGVUS
تم شحن الضوء بالفعل بواسطة `argvus-appearance`.

قم بتثبيت سمة واحدة باسم الحزمة الخاصة بها:

```bash
sudo pacman -S argvus-theme-<THEME_NAME>
```

على سبيل المثال:

```bash
sudo pacman -S argvus-theme-dracula
sudo pacman -S argvus-theme-rose-pine
```

تقوم الحزمة بتثبيت بيان السمة، `/usr/share/argvus/themes/<theme-id>/theme.toml` (الاسم والفئة وورق الحائط ولوحة الألوان، مع `overrides/` الاختياري)، وخلفية السمة ضمن `/usr/share/backgrounds/argvus/`. يتم عرض ملفات كل مكون من اللوحة عند تطبيق السمة. فهو لا يكتب في الدليل الرئيسي الخاص بك، ولا يغير المظهر الحالي الخاص بك. حدده من مركز التحكم بعد ذلك، راجع [الموضوعات الرسمية](/docs/components/argvus-themes/official-themes/).

## تثبيت العديد من المواضيع

لا توجد حزمة تعريفية تقوم بتثبيت كل سمة. قم بتثبيت السمات التي تريدها في أمر واحد:

```bash
sudo pacman -S argvus-theme-dracula argvus-theme-sunset
```
## الحزم المتوفرة

الحزم أدناه هي حزم السمات الرسمية المنشورة اليوم.

**موضوعات داكنة**

| الحزمة | موضوع |
| --- | --- |
| `argvus-theme-one-dark` | واحد مظلم |
| `argvus-theme-dracula` | دراكولا |
| `argvus-theme-silver-dark` | فضي داكن |
| `argvus-theme-slate-dark` | لائحة الظلام |
| `argvus-theme-universe` | الكون |
| `argvus-theme-gruvbox-high-dark` | جروف بوكس ​​هاي دارك |
| `argvus-theme-gruvbox-dark` | جروف بوكس ​​دارك |
| `argvus-theme-rose-pine` | روزي باين |
| `argvus-theme-tokyo-night` | ليلة طوكيو |
| `argvus-theme-solitude` | العزلة |
| `argvus-theme-sunset` | غروب الشمس |
| `argvus-theme-matrix` | مصفوفة |
| `argvus-theme-monokai-dark` | مونوكاي دارك |

**موضوعات خفيفة**

| الحزمة | موضوع |
| --- | --- |
| `argvus-theme-github-light` | جيثب لايت |
| `argvus-theme-solarized-light` | ضوء شمسي |
| `argvus-theme-nord-light` | نورد لايت |
| `argvus-theme-one-light` | نور واحد |
| `argvus-theme-everforest-light` | ايفرفوريست لايت |
| `argvus-theme-catppuccin-latte` | كاتبوتشين لاتيه |
| `argvus-theme-gruvbox-light` | جروف بوكس ​​لايت |
| `argvus-theme-watercolor` | ألوان مائية |

لا تحتاج سمات **ARGVUS Dark** و**ARGVUS Light** المضمنة إلى حزمة منفصلة؛ أنها تأتي مع `argvus-appearance`.

## تحقق مما تم تثبيته

قم بإدراج حزم السمات المثبتة مع:

```bash
pacman -Qs argvus-theme
```

افتح مركز التحكم وانتقل إلى **المظهر → السمات → السمات الرسمية** لرؤية السمات المتوفرة حاليًا في أداة التحديد. السمة التي تم تثبيتها ولكنها مفقودة من تلك القائمة تمثل مشكلة في التغليف؛ راجع قسم استكشاف الأخطاء وإصلاحها في [دليل السمات](/docs/components/argvus-themes/).

## أضف سمة خاصة بك

السمة عبارة عن بيان، لذا فإن السمة الخاصة بك لا تحتاج إلى حزمة. إنشاء
`~/.local/share/argvus/themes/<id>/theme.toml`، حيث يكون اسم الدليل هو
معرف الموضوع (الأحرف الصغيرة والأرقام والواصلات المفردة):

```toml
schema = 2
id = "mist"
name = "Mist"
category = "dark"
wallpaper = "/usr/share/backgrounds/argvus/argvus-dark.jxl"

[palette]
bg = "#101418"
fg = "#D8DEE9"
accent = "#88C0D0"
```

مطلوب فقط `bg` و`fg` و`accent`: كل لون آخر له لون افتراضي
مشتق منها، ولا يعرف البيان إلا تغييره. التحقق من ذلك،
ثم قم بتطبيقه مثل أي موضوع آخر:

```bash
argvus theme check ~/.local/share/argvus/themes/mist
argvus theme apply mist
```

يعرض `argvus theme list` موضوعاتك بجوار السمات الرسمية (مسؤول
الموضوع الذي يحمل نفس المعرف يفوز). تظهر السمات الخاصة بك في قائمة السمات
(`SUPER + Shift + T`)؛ لا يمكن لمرحب تسجيل الدخول المسبق قراءة الدليل الرئيسي الخاص بك، لذلك
فهو يحتفظ بالسمة الرسمية الأخيرة التي قمت بتطبيقها أثناء نشاط أحد السمات الخاصة بك.

النسخ الخاصة بكل سمة والتي كتبت تحتها إصدارات ARGVUS الأقدملم تعد `~/.config/argvus/data/<app>/themes/` قابلة للقراءة. تبديل الموضوع الأول
بعد التحديث ينقلهم إلى `~/.config/argvus/data/backups/theme-copies-<time>/`؛
للاحتفاظ بالملف الذي قمت بتحريره، ضعه في سمة خاصة بك ضمن `overrides/` و
قم بتسميته في جدول `[overrides]` الخاص بالبيان.

## إزالة الموضوع

قم بإزالة حزمة سمة واحدة باستخدام:

```bash
sudo pacman -R argvus-theme-<THEME_NAME>
```

لا تؤدي إزالة حزمة السمات إلى حذف ملفات التعريف المستوردة. إذا كانت السمة التي تمت إزالتها هي السمة المطبقة حاليًا، فحدد سمة أخرى من مركز التحكم قبل إزالتها.