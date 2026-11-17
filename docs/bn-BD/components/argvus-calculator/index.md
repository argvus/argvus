---
title: ARGVUS ক্যালকুলেটর
description: ARGVUS ক্যালকুলেটর — GTK4 / ARGVUS
---

# ARGVUS ক্যালকুলেটর

`argvus calculator` · launcher: **ARGVUS ক্যালকুলেটর**

- GTK4 / Relm4; ARGVUS theme (`time`), IBM Plex Mono.
- `+`, `-`, `*`, `/`, `%`, `±`; `50% = 0.5`.
- **বৈজ্ঞানিক**: `sqrt`, `log`, `ln`, `exp`, `sin`, `cos`, `tan`, `asin`, `acos`, `atan`, `pi`, `e`, `^`, `(`, `)`.
- **রেডিয়ান**: অনির্বাচিত: ডিগ্রি.
- `-2^2 = -4`; `2^3^2 = 512`; `2*pi`; `1.5 = 1,5`.
- Enter: `=`; Escape: **মুছুন**; Backspace: **অক্ষর মুছুন**; Tab / Shift+Tab.
- **ফলাফল কপি করুন**; **ইতিহাস**: 100; **ইতিহাস মুছুন**.
- IEEE 754 / f64; 15 digits; 4096 bytes; 64 levels.

## ইতিহাস

বন্ধ করার পরও সর্বোচ্চ ১০০টি ফলাফল থাকে। ইতিহাস মুছলে সংরক্ষিত রেকর্ডও মুছে যায়। সেটিংসে ↑/↓ দিয়ে চলুন, Enter বা ক্লিক দিয়ে সক্রিয় করুন, Escape দিয়ে ফিরুন।

`$XDG_DATA_HOME/argvus/calculator/history.json` (`~/.local/share/argvus/calculator/history.json`)

সবসময় কেন্দ্রে ছোট ভাসমান উইন্ডোতে (370 × 420) খোলে; বৈজ্ঞানিক মোড এটি বড় করতে পারে।

মূল পর্দায় শুধু রাশি, ফলাফল ও কীপ্যাড থাকে। মেনুতে বৈজ্ঞানিক মোড, কোণের একক, কপি এবং আলাদা ইতিহাস পর্দা আছে। ফিরে যান বা Escape গণনায় ফেরায়; ইতিহাসের ফলাফল বাছলে তা পুনর্ব্যবহার হয়। সাধারণ মোড রাশি রেখে ছোট উইন্ডো ফিরিয়ে আনে।

`SUPER + C` → `argvus calculator`

`.desktop`: `/usr/share/applications/argvus-calculator.desktop` · `Name=ARGVUS Calculator` · `Exec=argvus calculator`

ARGVUS: `focus_on_activate = true` (`org.argvus.*`, `sh.argvus.*`, `com.argvus.*`, `argvus-*`, `argvus.*`).

উইন্ডো বড় করলে বোতাম বড় হয়। প্রতিবার নতুন করে খোলার সময় ডিফল্ট আকার ব্যবহৃত হয়। অন্য পর্দার উপরে ফেরার তীর থাকে।

`s Settings` → `settings` · Escape / ←

Min: **370 × 420** / **440 × 640** · `0–9`: **16 px**.

`2+2` → Enter / `=` → `4` (input) · `4*3` → Enter → `12`.

নির্বাচিত সাধারণ বা বৈজ্ঞানিক মোড সঙ্গে সঙ্গে সংরক্ষিত হয় এবং পরেরবার খোলার সময় ফিরে আসে।

`$XDG_CONFIG_HOME/argvus/calculator/settings.json` (`~/.config/argvus/calculator/settings.json`).
