---
title: শেল সারফেস
description: টাস্কবার, কন্ট্রোল প্যানেল এবং টেলিমেট্রি মালিকানা।
---

`argvus-panels` প্যাচ করা `Quickshell Taskbar` প্রদান করে। `argvus-panels` টাস্কবার অ্যাকশন এবং ইউএক্সের মালিক; টাস্কবার JSONC এবং CSS এটি `config.json` এর অনুমান, এবং `argvus-config` `data/generated/taskbar/` এর অধীনে সক্রিয় স্তর এবং `data/taskbar/` এর অধীনে একটি মিররড কপি লিখে। `argvus-panels` Quickshell কার্ডের মালিক। `argvus-panels` provides the optional Telemetry surface, while the `argvus widget-telemetry` command in `argvus` retains block-control semantics; এটির JSONC এবং CSS একটি চেহারা পরিবর্তনের জন্য প্যাকেজ করা ডিফল্ট থেকে `data/taskbar/` তে লেখা হয়, তারপরে `argvus-config` শুধুমাত্র সীমাবদ্ধ `ARGVUS_TELEMETRY_*` এবং ফন্ট ব্লকগুলি পুনরায় লেখা হয়। `argvus-session` তাদের ব্যবহারকারী পরিষেবার মালিক৷

এই রানটাইম মডেলে কোন বর্তমান `argvus-shell` প্যাকেজ বা `argvus-shell.service` নেই।