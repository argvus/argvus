---
title: সংগ্রহস্থলের ভূমিকা
description: ওয়ার্কস্পেস একত্রীকরণের পরে ARGVUS সোর্স রিপোজিটরিগুলির দায়িত্ব৷
---

সংগ্রহস্থলগুলি নেভিগেশন পৃষ্ঠার পরিবর্তে দায়িত্ব অনুসারে গোষ্ঠীভুক্ত করা হয়েছে:

- ওয়ার্কস্পেস: `argvus` (ডিসপ্যাচার, ইন-প্রসেস কমান্ড, GTK 4 হেল্পার, শেয়ার্ড ক্রেট, `crates/argvus-greeter`-এর গ্রিটার এবং কমান্ডের ডেটা);- সেশন এবং একীকরণ: `argvus-session`, `argvus-hyprland`, `argvus-portal`, `argvus-lock`, `argvus-boot-splash`, এবং গ্রিটার (`argvus-greeter`, `argvus/crates/argvus-greeter` থেকে তৈরি);
- ওয়ার্কস্পেসের বাইরে UI: কন্ট্রোল প্যানেল, টাস্কবার (`argvus-taskbar`), লঞ্চার এবং ওয়েবার;
- প্রদানকারী: ডিসপ্লে, পাওয়ার, ফায়ারওয়াল, নোটিফিকেশন;
- সম্পদ এবং প্যাকেজিং: চেহারা, ওয়ালপেপার, ফন্ট, আইকন, প্রোফাইল।## কর্মক্ষেত্রে একত্রিত হয়েছে

এই সংগ্রহস্থলগুলিকে `de/argvus`-এ একত্রিত করা হয়েছে৷ তাদের ডকুমেন্টেশন এখন কর্মক্ষেত্রের `docs/` এর অধীনে থাকে এবং তাদের সংগ্রহস্থলগুলি শুধুমাত্র ইতিহাসের জন্য রাখা হয়:

| প্রাক্তন ভান্ডার | এখন | ডকুমেন্টেশন |
| --- | --- | --- |
| `argvus-config` | `argvus config` | [কমান্ড লাইন](/docs/reference/command-line/), [কনফিগারেশন ফাইল](/docs/reference/configuration-files/) |
| `argvus-i18n` | `argvus i18n` | [আন্তর্জাতিককরণ উপাদান](/docs/components/argvus-i18n/) |
| `argvus-keys`, `argvus-projects` | `argvus keys`, `argvus projects` | কমান্ড লাইন |
| `argvus-terminal` | `argvus terminal` | [টার্মিনাল কম্পোনেন্ট](/docs/components/argvus-terminal/) |
| `argvus-system-monitor` | `argvus system-monitor` | [সিস্টেম মনিটর উপাদান](/docs/components/argvus-system-monitor/) || `argvus-network` | `argvus network`, `argvus bluetooth` | [নেটওয়ার্ক কম্পোনেন্ট](/docs/components/argvus-network/) || `argvus-widget-telemetry` | `argvus widget-telemetry` | কমান্ড লাইন |
| `argvus-control-center` | `argvus control-center` (টার্মিনাল অ্যাপ্লিকেশন) | [কন্ট্রোল সেন্টার কম্পোনেন্ট](/docs/components/argvus-control-center/) |
| `argvus-taskbar-calendar` | `argvus taskbar-calendar` (GTK 4 সহায়ক) | [টাস্কবার ক্যালেন্ডার উপাদান](/docs/components/argvus-taskbar-calendar/) |
| `argvus-removable-devices` | `argvus removable-devices` (GTK 4 সহায়ক) | [অপসারণযোগ্য ডিভাইস উপাদান](/docs/components/argvus-removable-devices/) |
| `argvus-loading-theme` | `argvus loading-theme` (`splash` সাহায্যকারী) | কমান্ড লাইন |
| `argvus-tui-terminal` | `argvus tui-terminal` (ওয়ার্কস্পেসে ক্রেট) | কমান্ড লাইন |
| `argvus-tui`, `argvus-theme` | শেয়ার্ড ক্রেট `crates/argvus-tui` এবং `crates/argvus-theme` | বিকাশকারী নোট |
| `argvus-greeter` | `crates/argvus-greeter`-এর উত্স, `argvus` প্যাকেজে পাঠানো হয়েছে | [অভিবাদনকারী উপাদান](/docs/components/argvus-greeter/) |
| `argvus-themes` | অবসরপ্রাপ্ত। প্রতিটি অফিসিয়াল থিম তার নিজস্ব `argvus-theme-*` প্যাকেজ। | [থিম ইনস্টল করা হচ্ছে](/docs/user-guide/appearance/) |

একত্রিত উপাদানগুলির বিকাশকারী নোটগুলি `docs/internal/<repository>/`-এ রয়েছে৷

ভাষা মালিকানার সীমানা নয়: প্রতিটি উপাদান তার রানটাইম সাহায্যকারীকে তার প্যাকেজ দিয়ে রাখে। একটি কঠিন নিয়ম যা প্যাকেজ জুড়ে কাটে: শুধুমাত্র `argvus-config` `data/generated/` এর নীচে লিখে। এমন একটি উপাদান যা `config.json` পড়তে পারে না এমন একটি অ্যাপ্লিকেশনে পৌঁছাতে হবে তার নিজস্ব সহায়কে একটি অ্যাডাপ্টার যোগ করে এবং অর্কেস্ট্রেটরকে ক্যানোনিকাল কমিটের পরে এটিকে কল করতে দেয়; এটি কখনই জেনারেট করা ফাইল লেখে না। পাবলিক CLIগুলি `/usr/bin`-এর অন্তর্গত; অভ্যন্তরীণ সাহায্যকারীরা `/usr/lib/argvus/<component>` এর অধীনে বা রানটাইমে প্রয়োজন হলে কমান্ড ডিরেক্টরির মধ্যে থাকে; স্থাপত্য-স্বাধীন ডেটা `/usr/share/argvus/<component>` এর অধীনে থাকে।ইনস্টল করা প্যাকেজ সীমানার জন্য [প্যাকেজ ম্যাট্রিক্স](/docs/reference/package-matrix/) দেখুন।