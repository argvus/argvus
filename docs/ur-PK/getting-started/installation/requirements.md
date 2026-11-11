---
title: کے تقاضے
description: سسٹم کے تقاضے اور ARGVUS کے لیے ضروری شرائط۔
---

ARGVUS آرک لینکس کو نشانہ بناتا ہے اور اسے Wayland کے قابل گرافکس اسٹیک، Hyprland اور پیکجوں کے ذریعے کھینچے گئے رن ٹائم انحصار کی ضرورت ہوتی ہے۔ ایک ڈسپلے مینیجر اختیاری ہے: ARGVUS greetd اور دستی TTY بہاؤ کے ذریعے گرافیکل لاگ ان کے بہاؤ کی حمایت کرتا ہے۔

تنصیب سے پہلے، یقینی بنائیں کہ:

- سسٹم مکمل طور پر `pacman` کے ساتھ اپ ڈیٹ ہے۔
- ایک ورکنگ کرنل اور گرافکس ڈرائیور نصب ہیں۔
- صارف Wayland سیشن چلا سکتا ہے۔
- نیٹ ورکنگ تب دستیاب ہوتی ہے جب پیکیج ریپوزٹری کو کنفیگر کیا جاتا ہے۔

Virtual machines may need the compatibility settings described in [Virtual machines](/docs/user-guide/virtual-machines/).