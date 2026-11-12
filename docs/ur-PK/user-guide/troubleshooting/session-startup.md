---
title: سیشن اسٹارٹ اپ ٹربل شوٹنگ
description: گریٹر، ہائپرلینڈ اور سسٹمڈ یوزر سیشن کی ناکامیوں کی تشخیص کریں۔
---

چلائیں:

```sh
argvus sessionctl status
argvus sessionctl logs
systemctl --user status argvus-session.target
```

گریٹر، `argvus start`، Hyprland اور متعلقہ صارف یونٹ کو الگ الگ چیک کریں۔ ایک کامیاب پیکیج کی تعمیر یہ ثابت نہیں کرتی ہے کہ انسٹال شدہ پے لوڈ یا فعال صارف اوور رائڈ موجودہ ہے۔

اس ترتیب میں چیک استعمال کریں:

1. تصدیق کریں کہ مبارکباد دینے والا نصب شدہ ARGVUS greetd سیٹ اپ استعمال کر رہا ہے۔
2. سیشن لاگ کو پڑھیں اور گمشدہ یا غلط Hyprland Lua کنفیگریشن تلاش کریں۔
3. `systemctl --user status argvus-session.target` کے ساتھ تیاری اور ہدف کی حالت چیک کریں۔
4. ہر سروس کو ایک ساتھ دوبارہ شروع کرنے کے بجائے ناکام ہونے والے جزو کا معائنہ کریں۔

اگر لوڈنگ اوورلے فعال ہونے کے دوران `argvus-session.target` غیر فعال ہے، تو معائنہ کریں
خدمات کو دوبارہ شروع کرنے سے پہلے پروجیکشن کی خرابی:

```sh
argvus config project
journalctl --user -u argvus-session.target -u argvus-session-prepare.service -b --no-pager
```

`argvus-config ` ` data/generated/` کے نیچے واحد مصنف ہے، لہذا ` project` ہے
تشخیص کرتے وقت نسل کو دوبارہ پیدا کرنے کا مستند طریقہ۔ سیشن کا آغاز
اسے چلاتا ہے کیونکہ ابھی تک کوئی پرعزم منصوبہ موجود نہیں ہے۔ منصوبہ جس پر یہ لکھتا ہے۔
`${XDG_STATE_HOME:-$HOME/.local/state}/argvus/config-projection.json` کیا ہے a
بعد میں `argvus sessionctl reload` دوبارہ پیش کیے بغیر لاگو ہوتا ہے۔

مہلک تیاری یا ہدف کے آغاز پر سیشن لوڈنگ اوورلے کو روکتا ہے۔
ناکام ایک بصری صارف کی ناکامی کو یا تو اس کا دستاویزی فال بیک استعمال کرنا چاہیے یا ہونا چاہیے۔
واضح طور پر اطلاع دی گئی؛ اسے کامیاب سیشن کے طور پر چھپایا نہیں جانا چاہیے۔

## سست لاگ ان

اگر `greeter.handoff-takeover-failed ` ` handoff.master-acquired` کے بغیر ظاہر ہوتا ہے،
حصول کے لیے `handoff.master-unavailable card=<name> errno=<n>` کا معائنہ کریں۔
غلطی ایک مبصر نے کھولا جب کہ دوسرا کمپوزر ماسٹر تھا غلطی سے واپس آ سکتا ہے۔
13 (`EACCES`) اس کے بعد بھی کمپوزر ماسٹر کو جاری کرتا ہے۔ مددگار دوبارہ کھولتا ہے۔
واضح طور پر ماسٹر حاصل کرنے کے لئے ایک ہی GPU؛ `handoff.master-reopened` اسے ریکارڈ کرتا ہے۔
ریکوری اور اس ڈسکرپٹر پر ایک نیا فریم بفر تیار کیا جاتا ہے۔ سلام کرنے والا ناکام مددگار (`greeter.handoff-cancelled`) کو منسوخ کرتا ہے، لہذا یہ
صارف کمپوزر کے شروع ہونے سے پہلے ایک اور انتظار شامل نہیں کر سکتا۔ ایک سیشن بھیجنا
`release ` جبکہ حصول زیر التواء ہے ` handoff.release-before-master` ملتا ہے:
مددگار تسلیم کرنے سے پہلے اپنے DRM ڈسکرپٹرز کو بند کر دیتا ہے۔ ایک سلامی کمپوزیٹر
Hyprland/Aquamarine میں حادثہ ایک الگ تلاش ہے۔ مددگار کو منسوخ کرنا
یہ قائم نہ کریں کہ ڈرائیور یا کمپوزر کا مسئلہ حل ہو گیا ہے۔

لاگ ان ہونے پر `greeter.vt-switch-returned active=<tty>` فعال VT کو ریکارڈ کرتا ہے۔
کال کی واپسی کو تبدیل کریں۔ ڈرائنگ ٹائم آؤٹ بھی اس میں `active-vt=<tty>` ریکارڈ کرتا ہے۔
غلطی یہ سنیپ شاٹس ہیں، مکمل DRM ہینڈ اوور کا ثبوت نہیں۔

مقابلے سے پہلے/بعد کے لیے، کام کو محفوظ کریں اور فی ورژن پر پانچ لاگ ان استعمال کریں۔
ایک ہی مشین، تھیم اور مانیٹر سیٹ اپ۔ ہر مندرجہ ذیل لاگ آؤٹ سے پہلے، کاپی کریں۔
`/run/argvus-greeter/timing.log`، سیشن لاگ اور بوٹ جرنل کو الگ
`/tmp/argvus/` کے تحت نمونہ ڈائریکٹری؛ اگلا گریٹر رن اپنے وقت کی جگہ لے لیتا ہے۔
لاگ ہر نمونے کے ساتھ بوٹ ID ریکارڈ کریں اور صرف اس کے اندر ہی ٹائم سٹیمپ کا موازنہ کریں۔
بوٹ ہینڈ آف فرسٹ فریم، سیشن ریپر انٹری کے لیے توثیق کی پیمائش کریں،
فائنل ہائپرلینڈ لانچ اور لوڈنگ اسپنر اسٹاپ۔ درمیانی اور بدترین کا موازنہ کریں۔
توثیق سے اسپنر اسٹاپ کا وقت اور سیاہ فریموں کے لیے بصری طور پر چیک کریں،
چمکتا ہے یا پھنس گیا اسپنر۔ اکیلے ڈیسک ٹاپ سطحوں کی مدت نہیں ہے۔
آخر سے آخر تک لاگ ان کا وقت۔ ڈسپوزایبل سیشن میں ناکامی اور بازیابی کے ٹیسٹ دہرائیں،
غیر محفوظ شدہ کام پر مشتمل سیشن کو ختم کیے بغیر۔

ہر سٹارٹ اپ مرحلہ سیشن لاگ میں اپنی مدت ریکارڈ کرتا ہے۔ آخری لاگ ان کے مراحل دیکھنے کے لیے، پہلے سب سے سست:

```sh
grep timing ~/.local/state/argvus/session.log | tail -30 | sort -k4 -rn
systemd-analyze --user critical-chain argvus-session.target
```

گریٹر سے ڈیسک ٹاپ پر سوئچ کرنے کے لیے، جریدے کے ساتھ گریٹر اور ہینڈ آف مارکس کو پڑھیں۔ یہ سب `CLOCK_MONOTONIC` استعمال کرتے ہیں، اس لیے وہ قطار میں کھڑے ہیں:

```sh
cat /run/argvus-greeter/timing.log
journalctl -b -o short-monotonic --since -2min | grep -E 'greetd|Hyprland|coredump|argvus-greeter-handoff|session-loading'
grep monotonic_ns ~/.local/state/argvus/session.log | tail -5
```

`timing.log ` holds the last login only: the greeter, the DRM handoff surface (` handoff.master-acquired `, ` handoff.output-found `, ` handoff.first-frame `, ` handoff.release-received `, ` handoff.replaced-by-compositor `) and ` greeter.*` marks. ` handoff.crtc ` lines record every CRTC change while the handoff waits for DRM master; a` fb=0 ` یا ` mode_valid=false `، یا ` handoff.output-found ... framebuffer=0 ` کے ساتھ لائن کا مطلب ہے کہ ہینڈ آف لینے سے پہلے ہی گریٹر کمپوزر نے آؤٹ پٹ کو آف کر دیا، اس لیے مانیٹر کو دوبارہ مطابقت پذیر ہونا پڑا۔ ` greeter.watchdog-stop-requested ` کا مطلب ہے کہ سلامی سیشن اپنے کمپوزر کا انتظار کیے بغیر ختم ہو گیا، اور ` greeter.compositor-exit-deferred ` کہدیہینڈ آف سطح ختم ہونے کے بعد کمپوزر سے باہر نکلنے کو کہا جاتا ہے۔ ` greeter.compositor-exit-requested ` اکیلے کا مطلب ہے کہ اسے ایک ساتھ باہر نکلنے کے لیے کہا گیا تھا۔ اگر جرنل میں گریٹیڈ اور لاگ ان لائنیں گریٹر سیشن کے بند ہونے اور صارف کے سیشن کے کھلنے کے درمیان تقریباً آدھے سیکنڈ کا وقفہ دکھاتی ہیں، تو لاگ ان کے دوران گریٹر کمپوزر کا ٹیر ڈاؤن چلا گیا۔ اس کی توقع کی جاتی ہے جبکہ نشان ` compositor-in-foreground=true ` دکھاتا ہے: گریٹر کے ٹرمینل کا ہینگ اپ موخر ایگزٹ سے پہلے کمپوزر کو ختم کرتا ہے۔ ایسا تب ہوتا ہے جب سلام کرنے والا اپنے کمپوزر کے باہر ہونے سے پہلے اپنا VT نہیں چھوڑ سکتا تھا: ` greeter.vt-switch-requested ` کو تلاش کریں` greeter.handoff-drawing `;جب دوسرا غائب ہو، ` greeter.handoff-takeover-failed error=<reason>` کہتا ہے کیوں۔ ` handoff.page-flip ` کا مطلب ہے کہ ہینڈ آف نے گریٹر کے آخری فریم کو بغیر موڈ سیٹ کے بدل دیا۔ ` handoff.page-flip-failed errno=<n>` اس کے بعد ` handoff.modeset ` کا مطلب ہے کہ کرنل نے صفحہ پلٹنے سے انکار کر دیا۔ ` handoff.overlay plane=<id>` اور پھر ` handoff.retired ` کا مطلب ہے اوورلے نے ہائپرلینڈ کے پہلے فریموں کو اس وقت تک چھپا دیا جب تک کہ سیشن اسپنر اسکرین پر نہ ہو۔ ` handoff.overlay-unavailable ` کا مطلب ہے کہ CRTC کے پاس کوئی مفت اوورلے جہاز نہیں تھا، اور ` handoff.retire-timeout ` کا مطلب ہے کہ سیشن اسپنر نے کبھی بھی ` retire ` نہیں بھیجا تھا۔ سیشن اسپنر (` argvus-greeter-handoff `) جریدے کو ` first layer-shell frame callback ` اور ` stop requested` لکھتا ہے۔

ایک `project.theme-switch ` یا غیر صفر ` prepare.startup-theme ` اندراج کی ظاہری شکل میں تبدیلی، تھیم فائلوں میں تبدیلی، ایک نیا لہجہ یا پرانے اسٹارٹ اپ سٹیمپ کی منتقلی کے بعد متوقع ہے۔ ایک جیسی فائلوں کو دوبارہ انسٹال کرنا مواد پر مبنی اسٹارٹ اپ کیشے کو محفوظ رکھتا ہے۔ اگر یہ ہر لاگ ان پر ظاہر ہوتا ہے، تو مادی تھیم فائلیں غائب ہیں۔ ` argvus theme-switch <theme>` ایک بار چلائیں۔سیشن لاگ کو `$XDG_STATE_HOME/argvus/session.log ` (عام طور پر `~/.local/state/argvus/session.log`) کے تحت رکھا جاتا ہے۔ ایک صارف Hyprland اوور رائڈ کو تبدیل کرنے کے بعد، لاگ آؤٹ کریں اور اس طرح سیشن انٹری پوائنٹ کی توثیق ہوتی ہے اور اسے دوبارہ منتخب کرتا ہے۔ عام آغاز کے بہاؤ کے لیے [گرافیکل سیشن](/docs/argvus-session/graphical-session/) دیکھیں۔
