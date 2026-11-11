---
title: ফাইলের অবস্থান
description: গুরুত্বপূর্ণ ইনস্টল করা ARGVUS ফাইল এবং ডিরেক্টরি।
---

- `/usr/bin/argvus*` — পাবলিক কমান্ড।
- `/usr/share/argvus/hyprland/` — হাইপ্রল্যান্ড কনফিগারেশন এবং সাহায্যকারী।
- `/usr/share/argvus/control-panel/` — কন্ট্রোল প্যানেল QML এবং স্ক্রিপ্ট।
- `/usr/share/argvus/taskbar/` — টাস্কবার কনফিগারেশন এবং স্ক্রিপ্ট।
- `/usr/share/argvus/lock/` — হাইপ্রলক কনফিগারেশন।
- `/usr/share/argvus/portal/` — পোর্টাল ইন্টিগ্রেশন ফাইল।
- `/usr/share/backgrounds/argvus/` — ওয়ালপেপার সম্পদ।
- `/etc/argvus/` — সিস্টেম প্রদানকারী কনফিগারেশন।
- `/etc/argvus/greeter.toml` — শুভেচ্ছা কনফিগারেশন।
- `/etc/argvus/removable-devices/config.json` — অপসারণযোগ্য স্টোরেজের জন্য সিস্টেম ডিফল্ট।
- `$XDG_CONFIG_HOME/argvus/config.json` — ক্যানোনিকাল পোর্টেবল পছন্দ।
- `$XDG_CONFIG_HOME/argvus/data/` — পরিচালিত উপাদান ডেটা, ব্যাকআপ, লক এবং উৎপন্ন আউটপুট।
- `$XDG_CONFIG_HOME/waybar/argvus-taskbar.{jsonc,css}` — ঐচ্ছিক সম্পূর্ণ নেটিভ ওয়েবার টাস্কবার ওভাররাইড করে।
- `$XDG_CONFIG_HOME/argvus/data/taskbar/argvus-taskbar.{jsonc,css}` — পরিচালিত ARGVUS টাস্কবার কপি।
- `$XDG_CONFIG_HOME/argvus/data/generated/waybar/` — জেনারেটেড ওয়েবার প্রোফাইল, `argvus-taskbar.css` এবং `argvus-widget-telemetry.{jsonc,css}` সহ; এগুলি উপরের কপিগুলির উপর প্রাধান্য পায়।
- `$XDG_CONFIG_HOME/argvus/data/generated/waybar/argvus-widget-telemetry.{jsonc,css}` — এছাড়াও `data/waybar/` এর অধীনে মিরর করা হয়েছে, যা একটি উপস্থিতি পরিবর্তনে প্যাকেজ করা ডিফল্ট থেকে প্রতিস্থাপিত হয় এবং তারপরে এর `ARGVUS_TELEMETRY_*` এবং ফন্ট ব্লকগুলি পুনরায় লেখা হয়।
- `$XDG_CONFIG_HOME/argvus/data/generated/yazi/` — প্রজেক্টেড ইয়াজি কনফিগারেশন ট্রি, থিমগুলির জন্য একটি বৈকল্পিক-সচেতন ফলব্যাক সহ যা কোন ইয়াজি ফ্লেভার পাঠায় না।
- `$XDG_CONFIG_HOME/argvus/data/generated/superfile/` — প্রক্ষিপ্ত সুপারফাইল থিম সেট।
- `$XDG_CONFIG_HOME/argvus/data/generated/terminal/` — প্রক্ষিপ্ত টার্মিনাল প্রোফাইল।
- `$XDG_CONFIG_HOME/argvus/data/generated/qt6ct/` — প্রক্ষিপ্ত Qt6ct প্যালেট।
- `$XDG_CONFIG_HOME/argvus/data/generated/gtk/` — প্রক্ষিপ্ত GTK থিম ফাইল।
- `$XDG_CONFIG_HOME/argvus/data/rofi/` — Rofi কনফিগারেশন, থিম এবং মোডের ব্যবহারকারীর কপি। ARGVUS প্রতিটি কপি তৈরি করে যখন এটি প্রথম ফাইল পরিবর্তন করে; অপরিবর্তিত ফাইলগুলি `/usr/share/argvus/launcher/config/` থেকে পড়া হয়।
- `$XDG_CONFIG_HOME/argvus/data/generated/removable-devices/theme.css` — প্রক্ষিপ্ত অপসারণযোগ্য-ডিভাইস স্টাইলশীট।
- `$XDG_CONFIG_HOME/argvus/data/focus/settings.json`, `$XDG_DATA_HOME/argvus/focus/history.json`, `$XDG_STATE_HOME/argvus/focus/session.json` — ARGVUS ফোকাস সেটিংস, সেশন ইতিহাস এবং বর্তমান-সেশন চেকপয়েন্ট৷
- `$XDG_RUNTIME_DIR/argvus/focus/status.json` — টাস্কবার দ্বারা ক্ষয়প্রাপ্ত ক্ষণস্থায়ী ARGVUS ফোকাস স্থিতি; ফোকাস প্রস্থান করার সময় সরানো হয়।
- `$XDG_CONFIG_HOME/argvus/data/clock/settings.json`, `$XDG_DATA_HOME/argvus/clock/alarms.json`, `$XDG_STATE_HOME/argvus/clock/timers.json` — ARGVUS ঘড়ি সেটিংস এবং বিশ্ব ঘড়ি, অ্যালার্ম এবং টাইমার/স্টপওয়াচ চেকপয়েন্ট।
- `$XDG_CONFIG_HOME/argvus/data/generated/theme/current/time/theme.css` — ARGVUS ফোকাস এবং ARGVUS ঘড়ির রঙ (থিম গ্রাহক `time`)।
- `/usr/share/argvus/sounds/` — ARGVUS ফোকাস এবং ARGVUS ঘড়ির শব্দ (`argvus-chime.oga`, `argvus-beacon.oga`, `argvus-bell.oga`)।
- `$XDG_CONFIG_HOME/argvus/data/generated/theme/` — থিম বিল্ড (`builds/<id>-<variant>-<hash>/`), `argvus theme render` দ্বারা লিখিত এবং প্রতিটি থিম সুইচ এবং অ্যাকসেন্ট পরিবর্তনের মাধ্যমে, এবং সক্রিয় থিম তৈরিতে `current` সিমলিংক।
- `$XDG_CONFIG_HOME/argvus/data/hypr/` — ব্যবহারকারী হাইপ্রল্যান্ড লুয়া ওভাররাইড করে এবং নেটিভ হাইপ্রল্যান্ড অনুমান।
- `$XDG_CONFIG_HOME/hypr/hyprland.lua` — `argvus start` দ্বারা নির্বাচিত ঐচ্ছিক সম্পূর্ণ নেটিভ হাইপ্রল্যান্ড কনফিগারেশন; এটি একটি বর্ধিত ওভারলে নয়।
- `$XDG_STATE_HOME/argvus/config-projection.json` — প্রজেকশন ম্যানিফেস্ট: জেনারেশন, ইফেক্টিভ-কনফিগ হ্যাশ, পরিবর্তিত বিভাগ এবং মুলতুবি রানটাইম কাজ। `argvus sessionctl reload` দ্বারা পড়া; কখনোই সত্যের উৎস নয়।
- `$XDG_STATE_HOME/argvus/widget-telemetry-blocks` — লিগ্যাসি টেলিমেট্রি ব্লক পছন্দসমূহ। শুধুমাত্র মাইগ্রেশন ইনপুট; ক্যানোনিকাল মানগুলি `config.json`-এ থাকে এবং এই ফাইলটি আর লেখা হয় না।
- `$XDG_STATE_HOME/argvus/session.log` — গ্রাফিক্যাল সেশন লগ (সাধারণত `~/.local/state/argvus/session.log`)।
- `/usr/share/wayland-sessions/argvus.desktop` — গ্রাফিক্যাল সেশন এন্ট্রি।
- `/usr/share/plymouth/themes/argvus/` — প্যাকেজড বুট স্প্ল্যাশ সম্পদ।

সঠিক পেলোডগুলি প্যাকেজ-নির্দিষ্ট; ডেভেলপার কম্পোনেন্ট রেফারেন্স এবং প্রাসঙ্গিক প্যাকেজ ম্যানিফেস্ট দেখুন।