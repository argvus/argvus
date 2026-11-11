---
title: প্রথম সেশন
description: ইনস্টলেশনের পরে ARGVUS শুরু করুন।
---

একটি গ্রাফিকাল লগইনের জন্য, প্যাকেজড গ্রিটড ইন্টিগ্রেশনটি `argvus greeter-setup --help` এর সাথে কনফিগার করুন এবং [সেশন](/docs/user-guide/sessions/) এ বর্ণিত ডিসপ্লে-ম্যানেজার ফ্লো ব্যবহার করুন।

একটি TTY সেশনের জন্য, একটি ভার্চুয়াল টার্মিনালে লগ ইন করুন এবং চালান:

```sh
argvus --start-desktop
```

লগইন করার পরে, ARGVUS ব্যবহারকারীর লক্ষ্য পরিদর্শন করতে `argvus sessionctl status` ব্যবহার করুন৷ কম্পোজিটর বা শেল শুরু না হলে, [সেশন ট্রাবলশুটিং](/docs/user-guide/troubleshooting/session-startup/) অনুসরণ করুন।