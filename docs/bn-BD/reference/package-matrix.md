---
title: প্যাকেজ ম্যাট্রিক্স
description: ARGVUS প্যাকেজ দায়িত্ব।
---

| প্যাকেজ | দায়িত্ব |
| --- | --- |
| `argvus` | মাস্টার প্যাকেজ: প্রেরণকারী, ইন-প্রসেস কমান্ড (কনফিগারেশন, অনুবাদ, কী, প্রকল্প, টার্মিনাল, সিস্টেম মনিটর, নেটওয়ার্ক, ব্লুটুথ, টেলিমেট্রি, টিইউআই টার্মিনাল, সেটআপ), ভাগ করা ডেটা এবং i18n ক্যাটালগ। প্রতিটি উপাদান প্যাকেজ এটির উপর নির্ভর করে। |
| `argvus` (সহায়ক) | একই প্যাকেজে সাহায্যকারী: কন্ট্রোল সেন্টার এবং ডিস্ক ব্যবহার বিশ্লেষক (টার্মিনাল অ্যাপ্লিকেশন), এবং টাস্কবার ক্যালেন্ডারের জন্য GTK 4 অ্যাপ্লিকেশন, অপসারণযোগ্য ডিভাইস, লোডিং ওভারলে, ARGVUS ফোকাস এবং ARGVUS ঘড়ি। |
| `argvus` (অভিবাদনকারী) | অভিবাদন এর জন্য লগইন করুন, একই প্যাকেজে পাঠানো হয়েছে এবং `crates/argvus-greeter` থেকে তৈরি করা হয়েছে। |
| `argvus-desktop` | `argvus` মাস্টার প্যাকেজের জন্য ডেস্কটপ এবং দৈনন্দিন অ্যাপ্লিকেশন প্রোফাইল প্রয়োজন। এটি `argvus` এর উপর নির্ভর করে না, তাই নির্ভরতা একমুখী থাকে। |
| `argvus-devtools`, `argvus-pentest` | উন্নয়ন সরঞ্জাম এবং অনুমোদিত নিরাপত্তা নিরীক্ষণের জন্য ঐচ্ছিক স্বাধীন মেটাপ্যাকেজ। মাস্টার প্যাকেজ এই প্রোফাইলগুলি ইনস্টল করে না। |
| `argvus-session` | সেশন এন্ট্রি পয়েন্ট এবং ব্যবহারকারীর জীবনচক্র। |
| `argvus-hyprland` | হাইপ্রল্যান্ড কনফিগারেশন। |
| `argvus-control-panel` | কুইকশেল কন্ট্রোল প্যানেল। |
| `argvus-taskbar`, `argvus-waybar` | ওয়েবার কনফিগারেশন এবং প্যাচড বাইনারি। |
| `argvus-branding` | লোগো এবং ওয়ার্ডমার্ক সম্পদ। |
| `argvus-appearance`, `argvus-wallpapers`, `argvus-fonts`, `argvus-icons` | চেহারা রাষ্ট্র এবং চাক্ষুষ সম্পদ. |
| `argvus-display`, `argvus-power`, `argvus-notifications` | ডেস্কটপ প্রদানকারী। |
| `argvus-lock`, `argvus-boot-splash` | লক স্ক্রিন এবং বুট স্প্ল্যাশ। |
| `argvus-firewall` | ফায়ারওয়াল, নিজস্ব সুবিধা সহ। |
| `argvus-launcher` | লঞ্চার কনফিগারেশন এবং এন্ট্রি পয়েন্ট। || `argvus-app-profiles` | বান্ডিল টার্মিনাল অ্যাপ্লিকেশনের জন্য শুধুমাত্র ডেটা প্রোফাইল। |
| `argvus-portal` | XDG পোর্টাল এবং Wayland/DBus ডিফল্ট। |
| `argvus-theme-*` | অফিসিয়াল থিম পরিবার প্রতি একটি ঐচ্ছিক প্যাকেজ। |
| `argvus-game-snake`, `argvus-games` | ঐচ্ছিক। স্নেক একটি স্বাধীন প্রকল্প; `argvus-games` ঐচ্ছিক গেমগুলিকে গোষ্ঠীভুক্ত করে৷ |

## প্রতিস্থাপিত প্যাকেজ

`argvus` এটি শোষিত প্যাকেজগুলির জন্য `replaces`, `conflicts` এবং `provides` ঘোষণা করে, তাই একটি আপগ্রেড তাদের সরিয়ে দেয় এবং তাদের ফাইলগুলি অনাথ হিসাবে থাকে না:`argvus-config`, `argvus-i18n` (with `argvus-language-en-us` and `argvus-language-pt-br`), `argvus-keys`, `argvus-projects`, `argvus-terminal`, `argvus-system-monitor`, `argvus-network`, `argvus-widget-telemetry` and `argvus-du`. `argvus` also replaces `argvus-control-center`, `argvus-taskbar-calendar`, `argvus-loading-theme`, `argvus-removable-devices`, and the former`argvus-gtk` এবং `argvus-greeter` বিভক্ত প্যাকেজ, যা এটি এখনও প্রদান করে যাতে প্যাকেজগুলি তাদের উপর নির্ভর করে সমাধান করতে থাকে। অ্যাকাউন্ট ম্যানেজার (`argvus accounts`) `crates/argvus-accounts` থেকে তৈরি করা হয়েছে এবং এটির পোলকিট নিয়মের সাথে `/usr/lib/argvus/commands/accounts` হিসাবে ইনস্টল করা হয়েছে। ডিস্ক ব্যবহার বিশ্লেষক `crates/argvus-du` থেকে নির্মিত এবং এটি হিসাবে ইনস্টল করা হয়েছে`/usr/lib/argvus/commands/argvus-du`.`/usr/bin/argvus-terminal` `argvus terminal` এর জন্য একটি সামঞ্জস্যপূর্ণ লঞ্চার হিসাবে ইনস্টল থাকে৷ হাইপ্রল্যান্ডের ডিফল্ট টার্মিনাল, কন্ট্রোল সেন্টার ক্যাটালগ এবং সংরক্ষিত ডিফল্ট অ্যাপ্লিকেশনগুলি এখনও সেই কমান্ডের নাম দেয়।

অপসারিত প্যাকেজ `argvus-pwvucontrol` এবং `argvus-snappy-switcher` কোন প্যাকেজ দ্বারা প্রদান করা হয় না।ARGVUS কম্পোনেন্ট প্যাকেজগুলির মধ্যে কোন ইনস্টল অর্ডারের প্রয়োজন নেই: `argvus` তাদের মাস্টার প্যাকেজ, এবং প্রতিটি উপাদান এটির উপর নির্ভর করে। `argvus` ইনস্টল করা `argvus-desktop` প্রোফাইলও ইনস্টল করে; `argvus-devtools` এবং `argvus-pentest` অপ্ট-ইন থাকবে। এই টেবিলে একটি প্যাকেজের উপস্থিতির অর্থ এই নয় যে এটি সম্পূর্ণ বৈশিষ্ট্যের মালিক: উদাহরণস্বরূপ, টাস্কবারটি `argvus-waybar`, `argvus-taskbar`, `argvus-session` এবং ডোমেন প্রদানকারীকে একত্রিত করে।