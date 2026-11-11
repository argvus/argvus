---
title: শেল সারফেস
description: টাস্কবার, কন্ট্রোল প্যানেল এবং টেলিমেট্রি মালিকানা।
---

`argvus-waybar` প্যাচ করা `/usr/bin/waybar` প্রদান করে। `argvus-taskbar` টাস্কবার অ্যাকশন এবং ইউএক্সের মালিক; টাস্কবার JSONC এবং CSS এটি `config.json` এর অনুমান, এবং `argvus-config` `data/generated/waybar/` এর অধীনে সক্রিয় স্তর এবং `data/waybar/` এর অধীনে একটি মিররড কপি লিখে। `argvus-control-panel` Quickshell কার্ডের মালিক। `argvus-widget-telemetry` ঐচ্ছিক টেলিমেট্রি বার এবং এর ব্লক শব্দার্থবিদ্যার মালিক; এটির JSONC এবং CSS একটি চেহারা পরিবর্তনের জন্য প্যাকেজ করা ডিফল্ট থেকে `data/waybar/` তে লেখা হয়, তারপরে `argvus-config` শুধুমাত্র সীমাবদ্ধ `ARGVUS_TELEMETRY_*` এবং ফন্ট ব্লকগুলি পুনরায় লেখা হয়। `argvus-session` তাদের ব্যবহারকারী পরিষেবার মালিক৷

এই রানটাইম মডেলে কোন বর্তমান `argvus-shell` প্যাকেজ বা `argvus-shell.service` নেই।