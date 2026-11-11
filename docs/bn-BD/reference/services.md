---
title: পরিষেবা
description: ARGVUS সিস্টেমড এবং ব্যবহারকারী পরিষেবা।
---

ফায়ারওয়াল হল  `argvus-session.target`  `argvus-taskbar.service`  `argvus-control-panel.service`  `argvus-widget-telemetry.service`  `argvus-hyprland.service`  `argvus-dunst.service`  `argvus-wallpaper.service`  `argvus-hypridle.service`  `argvus-session-loading.service` `argvus-firewall.service` নামে একটি সিস্টেম পরিষেবা। ক্যালেন্ডার প্যাকেজ `argvus-taskbar-calendar.service` ব্যবহারকারী ইউনিট হিসাবে ইনস্টল করে এবং `argvus-clock.service` (ডিফল্টরূপে অক্ষম) ARGVUS ঘড়ি লুকানো শুরু করে যখন **সেশন দিয়ে শুরু হয়** চালু থাকে।

এর সাথে সক্রিয় ইউনিটগুলি পরিদর্শন করুন:

```sh
systemctl --user status argvus-session.target
systemctl --user list-units 'argvus-*.service'
```

## সেশন মালিকানাধীন ইউনিটসেশন প্যাকেজ এই ব্যবহারকারী ইউনিটগুলিকে `/usr/lib/systemd/user` এর অধীনে ইনস্টল করে (সোর্স পেলোড সেশন প্যাকেজের শেয়ার্ড কনফিগারেশন ট্রির অধীনে):

| একক | ভূমিকা |
| --- | --- |
| `argvus-session.target` | ARGVUS গ্রাফিকাল সেশনের জন্য আজীবন সীমানা। |
| `argvus-session-prepare.service` | এক-শট স্টার্টআপ প্রস্তুতি এবং তৈরি কনফিগারেশন। |
| `argvus-control-panel.service` | কুইকশেল কন্ট্রোল প্যানেল। |
| `argvus-taskbar.service` | প্রধান ARGVUS টাস্কবার। |
| `argvus-widget-telemetry.service` | ঐচ্ছিক টেলিমেট্রি/sysinfo পৃষ্ঠ। |
| `argvus-hyprland.service` | অন-ডিমান্ড ওয়ান-শট ইউনিট যা `hyprctl reload` চালায়। |
| `argvus-config.service` | ক্যানোনিকাল কনফিগারেশন প্রজেক্ট করে (`argvus config project`); পুনরায় লোড `argvus sessionctl reload-config-transaction` চালায়। || `argvus-session-reload.service` | লগইন রিলোড (`argvus sessionctl reload-desktop --login`): হাইপ্রল্যান্ডকে শুধুমাত্র তখনই পুনরায় লোড করে যখন প্রজেকশন এটিকে প্রভাবিত করে। |
| `argvus-ssh-agent.service` | সেশনের জন্য SSH এজেন্ট। |
| `argvus-secret-service.service` | `argvus sessionctl start-secret-service` এর মাধ্যমে গোপন পরিষেবা শুরু করে। || `argvus-wallpaper.service` | সক্রিয় ওয়ালপেপার ব্যাকএন্ড। |
| `argvus-dunst.service` | Dunst বিজ্ঞপ্তি ডেমন. |
| `argvus-hypridle.service` | নিষ্ক্রিয় এবং লক নীতি. |
| `argvus-clipboard-text.service` / `argvus-clipboard-image.service` | ক্লিপবোর্ড ইতিহাস পর্যবেক্ষক. |
| `argvus-keyboard-layout.service` | কীবোর্ড-লেআউট বিজ্ঞপ্তি। |
| `argvus-polkit.service` | পলিসিকিট এজেন্ট ইন্টিগ্রেশন। |
| `argvus-session-loading.service` | কম্পোজিটর হ্যান্ডঅফ থেকে ডেস্কটপ উপাদানগুলি শুরু না হওয়া পর্যন্ত সেশন-লোডিং ওভারলে। |

`argvus-blueman-applet.service` সেশন প্যাকেজ দ্বারা পাঠানো হয়। কিছু ইউনিট ঐচ্ছিক বা শর্তসাপেক্ষে শুরু হয়; তারা প্রতিটি সেশনে সক্রিয় নয়।

শুধুমাত্র তার উৎস চেকআউটের পরিবর্তে একটি ইউনিটের কার্যকর সংজ্ঞা পরিদর্শন করতে:

```sh
systemctl --user cat argvus-session.target
systemctl --user status argvus-session.target
journalctl --user -u argvus-session.target -u 'argvus-*.service'
```

সিস্টেম-স্তরের `argvus-firewall.service` `argvus-firewall` এর অন্তর্গত। ক্যালেন্ডার প্যাকেজ আলাদাভাবে একটি ব্যবহারকারী ইউনিট হিসাবে `argvus-taskbar-calendar.service` পাঠায়। `argvus-clock.service` হল `argvus` প্যাকেজের একটি ব্যবহারকারী ইউনিট, `WantedBy=graphical-session.target`, ARGVUS ঘড়ি সেটিংস থেকে সক্ষম করা হয়েছে।

## সেশন ম্যানেজমেন্ট কমান্ড

`argvus sessionctl` কমান্ড সেশন লাইফসাইকেল পরিচালনা করে এবং লগ আউট না করে নির্দিষ্ট উপাদানগুলি পুনরায় চালু করতে পারে:

### সেশন জীবনচক্র

```sh
argvus sessionctl start               # Start a new session
argvus sessionctl stop                # Stop the current session
argvus sessionctl restart             # Restart the entire session
argvus sessionctl reload              # Reload configuration and active services
argvus sessionctl status              # Show session status
argvus sessionctl logs                # Display session logs
```

### কম্পোনেন্ট-নির্দিষ্ট রিস্টার্ট

সম্পূর্ণ সেশনকে প্রভাবিত না করেই পৃথক পরিষেবাগুলি পুনরায় চালু করুন:

```sh
# Desktop surfaces
argvus sessionctl restart waybar
argvus sessionctl restart wallpaper
argvus sessionctl restart shell

# Notifications and switcher
argvus sessionctl restart dunst

# Clipboard and input
argvus sessionctl restart clipboard keyboard-layout

# Bluetooth applet (optional)
argvus sessionctl restart blueman-applet
```

### পরিবেশ সিঙ্ক্রোনাইজেশন

সেশনে পরিবেশ ভেরিয়েবল আমদানি করুন:

```sh
argvus sessionctl import-environment
```

এটি শেল এবং সিস্টেমড ব্যবহারকারী পরিষেবাগুলির মধ্যে ওয়েল্যান্ড, হাইপ্রল্যান্ড, Qt, কার্সার এবং DBus ভেরিয়েবলগুলিকে সিঙ্ক্রোনাইজ করে।

## পরিষেবা ইউনিটের রেফারেন্স

অতিরিক্ত ব্যবহারকারী ইউনিট সরাসরি `systemctl --user` দিয়ে পরিচালিত হতে পারে:

```sh
systemctl --user status argvus-session.target
systemctl --user start argvus-session.target
systemctl --user stop argvus-session.target
systemctl --user restart <unit>

# View logs for a specific service
journalctl --user -u argvus-wallpaper.service -b -f
```

প্রাথমিক সেশন বুটস্ট্র্যাপের লগগুলি এখানে সংরক্ষণ করা হয়:

```
${XDG_STATE_HOME:-$HOME/.local/state}/argvus/session.log
```
