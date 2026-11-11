---
title: সেশন
description: একটি ARGVUS গ্রাফিকাল সেশন শুরু এবং পরিচালনা করুন।
---

`argvus-session` সেশন এন্ট্রি পয়েন্ট এবং `systemd --user` জীবনচক্রের মালিক। এটি `argvus start` এর মাধ্যমে হাইপ্রল্যান্ড শুরু করে, সেশন পরিবেশ আমদানি করে এবং ARGVUS টার্গেট পরিচালনা করে।

- [গ্রাফিক্যাল সেশন](/docs/argvus-session/graphical-session/)
- [TTY সেশন](/docs/argvus-session/tty-session/)
- [গ্রিটার](/docs/components/argvus-greeter/)

দরকারী ডায়গনিস্টিকস:

```sh
argvus sessionctl status
argvus sessionctl logs
argvus sessionctl restart waybar
```

পরিষেবার মালিকানার জন্য ডেভেলপার [রানটাইম লাইফসাইকেল](/docs/developer-guide/architecture/runtime-lifecycle/) দেখুন।