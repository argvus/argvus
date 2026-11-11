---
title: اکاؤنٹس
description: مقامی صارف پروفائلز، ڈسپلے نام، اوتار اور گروپ کی رکنیت، آرگوس اکاؤنٹس کے ذریعے۔
---

`argvus accounts` ARGVUS ڈیسک ٹاپ کے مقامی صارف اکاؤنٹس کی فہرست اور ترمیم کرتا ہے۔ یہ ایک نجی ہے۔
`argvus` پیکیج کے قابل عمل، `/usr/lib/argvus/commands/accounts` پر نصب۔ دی
`argvus` کمانڈ اسے شروع کرتی ہے، اور صرف وہ کارروائیاں جو دوسرے اکاؤنٹس یا سسٹم ڈیٹا کو تبدیل کرتی ہیں۔
جڑ کے طور پر چلائیں.

## احکامات

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

## نام اور اوتار ڈسپلے کریں۔

ڈسپلے کا نام معیاری GECOS فیلڈ میں محفوظ کیا جاتا ہے۔ یونکس کا صارف نام کبھی تبدیل نہیں ہوتا ہے۔

اوتار PNG، JPEG یا WebP فارمیٹ میں 20 MiB تک کی تصویر ہے۔ تصویر کا سائز 256×256 کر دیا گیا ہے۔
پکسلز اور `0644` موڈ کے ساتھ PNG کے طور پر `~/.face` پر محفوظ کیا گیا، فری ڈیسک ٹاپ مقام
greeter اور زیادہ تر ڈسپلے مینیجرز کے ذریعے۔ آپ کا اپنا اوتار آپ کی ہوم ڈائرکٹری کے اندر لکھا ہوا ہے اور
اجازت کی ضرورت نہیں ہے.

## اجازت

صرف `list`, `show`, `self`, `self groups` اور آپ کا اپنا اوتار بغیر اجازت کے کام کرتا ہے۔ ہر
دوسری تبدیلی ڈیسک ٹاپ کے پولکیٹ ایجنٹ سے اجازت طلب کرتی ہے۔ ARGVUS اصول
(`/usr/share/polkit-1/rules.d/50-argvus-accounts.rules`) فیصلہ کرتا ہے کہ کس کی توثیق کرنی چاہیے:

- ایڈمنسٹریٹر گروپ میں ایک صارف (`wheel`، آرک ڈیفالٹ، یا `sudo`) اپنے ساتھ تصدیق کرتا ہے
  پاس ورڈ
- کوئی دوسرا صارف روٹ پاس ورڈ کے ساتھ تصدیق کرتا ہے۔

| آپریشن | اجازت کے بغیر | اجازت کے ساتھ |
| --- | --- | --- |
| صارفین کی فہرست بنائیں، اکاؤنٹ دکھائیں، گروپس کی فہرست بنائیں | ✓ | ✓ |
| اپنا اوتار تبدیل کریں | ✓ | ✓ |
| اپنا ڈسپلے نام تبدیل کریں | - | ✓ |
| اپنا پاس ورڈ تبدیل کریں | - | ✓ (موجودہ پاس ورڈ ثابت کرنے کے بعد) |
| دوسرے صارف کا نام، اوتار، گروپس یا پاس ورڈ تبدیل کریں۔ - | ✓ |

اپنا پاس ورڈ تبدیل کرنے سے پہلے موجودہ پاس ورڈ چیک ہوتا ہے۔ ایک غلط پاس ورڈ سے انکار کر دیا گیا ہے۔
اجازت دینے کا اشارہ ظاہر ہونے سے پہلے۔

`sudo` کے ساتھ کمانڈ چلانے کی ضرورت نہیں ہے۔ کمانڈ پولکیٹ سے اجازت مانگتی ہے جب
اس کی ضرورت ہے.

## حفاظتی حدود

- `passwd` معیاری ان پٹ سے موجودہ، نئے اور تصدیقی پاس ورڈ پڑھتا ہے، فی لائن ایک،
  اور انہیں کبھی دلیل کے طور پر نہیں لیتا۔ موجودہ پاس ورڈ لائن خالی ہو سکتی ہے جب منتظم ہو۔
  دوسرے صارف کا پاس ورڈ دوبارہ ترتیب دیتا ہے۔ مثال:

  ```sh
  printf '%s\n%s\n%s\n' "$OLD" "$NEW" "$NEW" | argvus accounts passwd ghost
  ```

  انٹرایکٹو طور پر، تین لائنیں ٹائپ کریں اور ہر ایک کے بعد Enter دبائیں۔ ان پٹ پر دکھایا گیا ہے۔
  ٹرمینل جیسا کہ اسے ٹائپ کیا گیا ہے۔
- UID 1000 (اور `nobody`) سے نیچے اکاؤنٹ میں ترمیم کرنے سے انکار کر دیا گیا ہے، اور اسی طرح اسے لاک کر رہے ہیں، اسے حذف کر رہے ہیں
  اور اس کے پاس ورڈ کی میعاد ختم ہو رہی ہے۔ آپ کا اپنا اکاؤنٹ لاک یا ڈیلیٹ نہیں کیا جا سکتا۔
- GID 1000 سے نیچے کسی گروپ کا نام تبدیل کرنا یا حذف کرنا، یا ایڈمنسٹریٹر گروپ (`wheel`, `sudo`) ہے
  انکار کر دیا اس کے اراکین کو اب بھی تبدیل کیا جا سکتا ہے، سوائے اس کے کہ آپ اپنے آپ کو ایک سے نہیں ہٹا سکتے
  ایڈمنسٹریٹر گروپ
- ایڈمنسٹریٹر کی رکنیت `wheel` یا `sudo` گروپ ہے۔ کنٹرول سینٹر نیا اضافہ کرتا ہے۔
  `wheel` کے منتظمین جب سسٹم کے پاس ہوں، اور دوسری صورت میں `sudo` کو۔ اپنے آپ کو ہٹانا
  ترمیم کے ذریعے منتظم کی رکنیت سے انکار کر دیا گیا ہے۔

## مثالیں۔

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

## خرابی کا سراغ لگانا

- `pkexec was not found`: `polkit` انسٹال کریں۔ `argvus` پیکیج اس پر منحصر ہے۔
- اجازت دینے کا اشارہ ظاہر نہیں ہوتا ہے: چیک کریں کہ پولکیٹ تصدیقی ایجنٹ چل رہا ہے۔
  آپ کے سیشن میں. `argvus` پیکیج `hyprpolkitagent` پر منحصر ہے، جو ایک فراہم کرتا ہے۔
- `unix_chkpwd` پاس ورڈ تبدیل کرتے وقت غلطیاں: `pam` پیکیج مددگار فراہم کرتا ہے۔
- stderr پر کمانڈ کے مراحل کو پرنٹ کرنے کے لیے `argvus accounts --verbose ...` چلائیں۔

استحقاق کی حد کے بارے میں ڈویلپر کے نوٹس کے لیے، `docs/internal/argvus-accounts/` دیکھیں۔جب ایڈمنسٹریٹر خودکار لاگ ان کے لیے تشکیل کردہ اکاؤنٹ کو لاک یا ہٹاتا ہے، تو اکاؤنٹ فراہم کرنے والا سب سے پہلے اس کی گریٹیڈ آٹومیٹک لاگ ان انٹری کو غیر فعال کر دیتا ہے۔ اگر وہ صفائی ناکام ہو جاتی ہے، تو اکاؤنٹ آپریشن سے انکار کر دیا جاتا ہے۔ اگر بعد میں لاکنگ یا ہٹانا ناکام ہوجاتا ہے، تو خودکار لاگ ان غیر فعال رہتا ہے۔ دوسرے اکاؤنٹ کے لیے خودکار لاگ ان محفوظ ہے۔ اکاؤنٹ کھولنا خودکار لاگ ان کو فعال نہیں کرتا ہے۔ موجودہ سیشن بند نہیں ہیں اور سلام دوبارہ شروع نہیں کیا گیا ہے۔