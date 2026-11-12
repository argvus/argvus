---
title: কমান্ড লাইন
description: আরগভাস কমান্ড, এর সাবকমান্ড, কী-বাইন্ডিংয়ের জন্য রাখা পতাকা এবং প্রতিটি কোথায় চলে।
---

`/usr/bin/argvus ` হল ARGVUS-এর সর্বজনীন প্রবেশ বিন্দু। এটি ` argvus-cli` ক্রেট থেকে নির্মিত একটি মরিচা প্রোগ্রাম। প্রতিটি সাবকমান্ড হয় সেই প্রোগ্রামের ভিতরে চলে, কমান্ড ডিরেক্টরি থেকে এক্সিকিউটেবল একটি হেল্পার চালায় বা প্যাকেজড স্ক্রিপ্ট চালায়।

আপনার সংস্করণের সাথে ইনস্টল করা তালিকার জন্য `argvus --help ` চালান। সাবকমান্ডগুলি অগ্রণী `--` ছাড়াই লেখা হয়। `--` ফর্মগুলি শুধুমাত্র সেখানে রাখা হয় যেখানে একটি কীবাইন্ডিং বা একটি ডেস্কটপ এন্ট্রি তাদের ব্যবহার করে।

## যেখানে প্রতিটি কমান্ড চলে- **In-process:** the command is a library of the workspace and runs inside `argvus `. No separate executable exists. These are ` config `, ` theme `, ` i18n `, ` keys `, ` gpg `, ` projects `, ` terminal `, ` system-monitor `, ` network `, ` bluetooth `, ` focus status `, ` widget-telemetry `,` tui-terminal `, ` sensors ` এবং ` setup `।- **সাহায্যকারী:** `/usr/lib/argvus/commands/`-তে একটি পৃথক এক্সিকিউটেবল, যা ` argvus ` প্যাকেজ দ্বারা ইনস্টল করা হয়। এগুলি হল ` control-center ` (টার্মিনাল অ্যাপ্লিকেশন), ` argvus-du ` (` argvus du ` দিয়ে চালু করা হয়) এবং GTK 4 অ্যাপ্লিকেশনগুলি ` taskbar-calendar `, ` removable-devices `, ` focus ` এবং ` clock`।
- **ব্যক্তিগত কার্যকরযোগ্য ফাইল:** একটি কার্যকরযোগ্য ফাইল যা অন্য একটি প্যাকেজ `/usr/lib/argvus/commands/` তে ইনস্টল করে। ` argvus ` সেটি শুধুমাত্র চালু করে। এর মধ্যে রয়েছে ` session `, ` start `, ` tty `, ` sessionctl `, ` launcher `, ` snippets `, ` ssh `, ` git `, ` app-profiles-yazi-keymap `, ` notifications `, ` displayctl `, ` firewall `, ` accounts `, ` taskbar-actions ` এবং ` greeter-*` এন্ট্রি পয়েন্ট।
- **স্ক্রিপ্ট:** একটি প্যাকেজড শেল স্ক্রিপ্ট `/bin/sh` দিয়ে চালিত হয়। এইগুলি নীচে তালিকাভুক্ত হাইপ্রল্যান্ড স্ক্রিপ্ট কমান্ড।

`ARGVUS_COMMANDS_DIR ` পরীক্ষা এবং ` DESTDIR ` চেকের জন্য কমান্ড ডিরেক্টরি পরিবর্তন করে। ` PATH`-এ খুঁজে পাওয়া যায় না এমন একটি ব্যক্তিগত এক্সিকিউটেবল।

## সাবকমান্ড

| আদেশ | রান | নোট |
| --- | --- | --- |
| `argvus config ...` | প্রক্রিয়াধীন | কনফিগারেশন মডেল এবং সরঞ্জাম। এছাড়াও ` theme-switch ` এবং ` effects-toggle` দ্বারা পৌঁছেছে। |
| `argvus theme ...` | প্রক্রিয়াধীন | ` apply ID ` হল থিম সুইচ (` theme-switch ` এর মতোই)। থিম তৈরি করে: ` list ` ইনস্টল করা থিম, ` check DIRECTORY ` ইনস্টল করা টেমপ্লেটগুলির বিরুদ্ধে একটি থিম, ` render ID [--variant sticky\|spaced] [--accent #RRGGBB] [--output DIRECTORY]`। দেখুন [আবির্ভাব এবং থিম](/docs/developer-guide/subsystems/appearance-and-themes/)। |
| `argvus i18n ...` | প্রক্রিয়াধীন | অনুবাদ: ` get DOMAIN KEY [name=value...]`, ` locale `, ` list `, ` validate [LOCALE]`। || ` argvus keys ...` | প্রক্রিয়াধীন | SSH কী ম্যানেজার। ` gpg`GPG উপ-কমান্ডগুলির জন্য একটি ডাকনাম। |
| `argvus gpg ...` | In-process | GPG keys: ` list `, ` status `, ` generate `, ` export-public `, ` export-private `, ` import `, ` import-private `, ` delete `, ` edit `, ` keyserver`. |
| `argvus projects ...` | প্রক্রিয়াধীন | প্রজেক্ট পিকার। |
| `argvus terminal ...` | প্রক্রিয়াধীন | ARGVUS টার্মিনাল প্রোফাইল। |
| `argvus system-monitor ...` | প্রক্রিয়াধীন | btop সিস্টেম মনিটর; `--gpu ` শুধুমাত্র GPU-র বাক্সগুলি নির্বাচন করে, এবং `--panel [--cpu|--mem|--gpu]` এটিকে ARGVUS TUI শেলে এম্বেড করে৷ |
| `argvus du ` | হেল্পার (টার্মিনাল অ্যাপ্লিকেশন) | একটি ডেডিকেটেড কিটি উইন্ডোতে ARGVUS ডিস্ক ব্যবহার বিশ্লেষক খোলে এবং ncdu চালায়। || ` argvus network ...` | প্রক্রিয়াধীন | নেটওয়ার্ক ম্যানেজার স্থিতি এবং নিয়ন্ত্রণ (` networkctl ` একটি উপনাম হিসাবে রাখা হয়)। || ` argvus bluetooth ...` | প্রক্রিয়াধীন | ব্লুটুথ স্থিতি এবং নিয়ন্ত্রণ (` bluetoothctl ` একটি উপনাম হিসাবে রাখা হয়)। || ` argvus widget-telemetry block ...` | প্রক্রিয়াধীন | ওয়েবার টেলিমেট্রি ব্লক। এই বারের উত্তপ্ত পথ। || ` argvus widget-telemetry toggle ...` | প্রক্রিয়াধীন | টেলিমেট্রি ব্লকগুলি চালু এবং বন্ধ করে, এবং তাদের ক্রম। |
| `argvus game-snake ` | প্রাইভেট এক্সিকিউটেবল | টার্মিনাল স্নেক গেম, স্বাধীন ` argvus-game-snake` প্রকল্প থেকে, একটি ঐচ্ছিক প্যাকেজ। একটি টার্মিনালের বাইরে এটি একটি খোলে। |
| `argvus control-center [page]` | হেল্পার (টার্মিনাল অ্যাপ্লিকেশন) | কন্ট্রোল সেন্টার খোলে বা ইতিমধ্যে খোলা উইন্ডোটিকে ফোকাস করে। |
| `argvus welcome [--if-needed]` | প্রেরণকারী → নিয়ন্ত্রণ কেন্দ্র | কন্ট্রোল সেন্টারে ইন্টিগ্রেটেড ওয়েলকাম পেজ খোলে। `--if-needed` একবার খোলে যদি না প্রতি-ব্যবহারকারী অগ্রগতি রেকর্ড করে যে স্বাগতম ইতিমধ্যেই প্রদর্শিত, সম্পূর্ণ বা খারিজ করা হয়েছে। |
| `argvus taskbar-calendar [args]` | GTK 4 আবেদন | ` toggle` এ ডিফল্ট। |
| `argvus removable-devices [args]` | GTK 4 আবেদন | ` menu` এ ডিফল্ট। |
| `argvus focus [--toggle] [--start\|--pause] [--x X --y Y] [--bar-bottom PX --bar-right PX]` | GTK 4 আবেদন | ARGVUS ফোকাস পপআপ। প্রতি সেশনে একটি উদাহরণ। |
| `argvus focus status --waybar [--follow]` | প্রক্রিয়াধীন | ওয়েবার JSON হিসাবে ফোকাস টাস্কবার সূচক নির্গত করে; `--follow` GTK লোড না করেই স্ট্রিম পরিবর্তন করে। |
| `argvus clock [--toggle] [--background] [--page PAGE] [--x X --y Y] [--bar-bottom PX --bar-right PX]` | GTK 4 আবেদন | ARGVUS ঘড়ি পপআপ। প্রতি সেশনে একটি উদাহরণ; `--background` পপআপ ছাড়াই অ্যালার্ম সক্রিয় রাখে। |
| `argvus loading-theme ...` | /usr/lib/argvus/loading-theme/splash এ প্যাকেজ করা ` splash` | সরাসরি সেশন রিলোডের ওভারলে; একটি থিম সুইচ এটি ব্যবহার করে না। |
|  `/usr/lib/argvus/loading-theme/splash ` ` argvus session `, ` start `, ` tty `, ` sessionctl` | প্রাইভেট এক্সিকিউটেবল | সেশন এন্ট্রি পয়েন্ট এবং সেশন কন্ট্রোলার। |
| `argvus greeter `, ` greeter-session `, ` greeter-tui `, ` greeter-setup` | প্রাইভেট এক্সিকিউটেবল | গ্রিটার এন্ট্রি পয়েন্ট. |
| `argvus accounts `, ` firewall` | প্রাইভেট এক্সিকিউটেবল | অ্যাকাউন্ট এবং ফায়ারওয়াল সরঞ্জাম। |
| `argvus launcher `, ` snippets `, ` ssh `, ` git` | প্রাইভেট এক্সিকিউটেবল | লঞ্চার এবং পিকার টুল। |
| `argvus tui-terminal` | প্রক্রিয়াধীন | TUI অ্যাপ্লিকেশনের জন্য টার্মিনাল মোড়ক। |
| `argvus sensors ` | প্রক্রিয়াধীন | ` sensors -j` থেকে লাইভ তাপমাত্রা ওভারভিউ। |
| `app-profiles-yazi-keymap` | প্রাইভেট এক্সিকিউটেবল | কীম্যাপ সাহায্যকারী। |
| `argvus notifications `, ` displayctl `, ` taskbar-actions` | প্রাইভেট এক্সিকিউটেবল | বিজ্ঞপ্তি, প্রদর্শন এবং টাস্কবার অ্যাকশন। |
| `argvus setup [options]` | প্রক্রিয়াধীন | ব্যবহারকারী কনফিগারেশনে প্যাকেজ করা ডিফল্ট কপি করে। নিচে দেখুন. |

স্পেস এবং খালি মান সহ আর্গুমেন্ট অপরিবর্তিত ফরোয়ার্ড করা হয়। স্ট্যান্ডার্ড ইনপুট, আউটপুট, ত্রুটি, সংকেত এবং প্রস্থান অবস্থা যে কমান্ডটি চালায় তার অন্তর্গত। একটি অজানা নাম `argvus --help` চালানোর ইঙ্গিত সহ ব্যর্থ হয়৷ একটি কমান্ড যা প্রক্রিয়া চলাকালীন একটি যুক্তি প্রত্যাখ্যান করে যেটি একটি পরিবর্তিত মান পাস করার পরিবর্তে বৈধ UTF-8 (প্রস্থান অবস্থা 2) নয়; যে কমান্ডগুলি একটি পৃথক প্রোগ্রাম চালায় সেগুলি অপরিবর্তিত বাইটগুলি গ্রহণ করে।

## পতাকা এবং উপনাম

এই নামগুলি বিদ্যমান যাতে কী-বাইন্ডিং, ডেস্কটপ এন্ট্রি এবং পুরানো স্ক্রিপ্টগুলি কাজ করে। নতুন কোডে সাবকমান্ড ফর্ম পছন্দ করুন।

| পতাকা বা উপনাম | সমতুল্য |
| --- | --- |
| `--control-center `, `--about `, `--default-apps ` | ` control-center ` (` about ` এবং ` apps` পৃষ্ঠা সহ) |
| `--displays `, `--monitors ` পৃষ্ঠা | ` control-center displays` |
| `--network-interfaces ` | ` control-center network interfaces` |
| `--system-monitor `, `--terminal `, `--game-snake ` | ` system-monitor `, ` terminal `, ` game-snake` |
| `--calendar `, `--removable-devices `, `--storage ` | ` taskbar-calendar `, ` removable-devices` |
| `--focus `, `--clock ` | ` focus `, ` clock` |
| `--spf `, `--superfile ` | ফাইল ম্যানেজার (` spf`) |
| `--yazi `, `--yazy ` | ফাইল ব্রাউজার (` yazi`) |
| `--setup ` | ` setup` |
| `--start-desktop ` | ` tty` শুরু হয় (কোন আর্গুমেন্ট ফরওয়ার্ড করা হয় না) |
| `--version `, `-V `, ` version ` | ` ARGVUS <version>` প্রিন্ট করে
| `help `, `--help `, `-h ` | ব্যবহার প্রিন্ট করে || ` networkctl `, ` bluetoothctl ` | ` network `, ` bluetooth` |
| `widget-telemetry-toggle `, ` widget-telemetry-block ` | ` widget-telemetry toggle `, ` widget-telemetry block` |
| `theme-switch ` | ` config theme-apply` |
| `effects-toggle ` | ` config effects ` |কন্ট্রোল সেন্টার তার প্রথম আর্গুমেন্ট হিসাবে লিগ্যাসি পৃষ্ঠা ফ্ল্যাগ গ্রহণ করে এবং সেগুলিকে পৃষ্ঠা নামের সাথে মানচিত্র করে। উদাহরণ: `--apps ` হয়ে যায় ` apps `, `--keyboard-layout ` হয়ে যায় ` locale `, এবং `--hostname ` হয়ে যায় ` system`.

## Hyprland স্ক্রিপ্ট কমান্ডগুলো

এই রান প্যাকেজ স্ক্রিপ্ট এবং Hyprland কনফিগারেশন দ্বারা ব্যবহৃত হয়:`cheatsheets-hypr `, ` cheatsheets-launcher `, ` scratchpad `, ` hypr-screenshot `, ` spaces-switch `, ` emoji-picker `, ` toggle-sidebar `, ` weather-location `, ` layout-mode-menu `, ` brightness-switch `, ` toggle-mode `, ` idle-timeout `, ` keep-awake `, ` hypr-power-menu`.

ব্যবহারকারীর কনফিগারেশনের একটি স্ক্রিপ্ট প্যাকেজ করাকে ওভাররাইড করে। লুকআপ অর্ডারটি hyprland.lua-তে `_config_path `-এর মতোই: ব্যবহারকারীর কনফিগারেশন কপি, তারপর ARGVUS ডেটা কপি, তারপর স্টেট কপি, তারপর জেনারেট করা কপি এবং শুধুমাত্র তারপর  ` hyprland.lua ` `/usr/share/argvus` এর অধীনে প্যাকেজ করা ফাইল।## উদাহরণ

```sh
argvus control-center displays
argvus taskbar-calendar toggle
argvus widget-telemetry block header system
argvus widget-telemetry toggle status
argvus i18n get widget-telemetry label.device
argvus config get /appearance/theme
argvus network status
argvus bluetooth status
argvus gpg list
```

## সেটআপ `argvus setup` প্যাকেজ করা ডিফল্টগুলিকে `/usr/share/argvus/<app>` থেকে `~/.config/argvus/data/<app>` এ কপি করে, যাতে আপনি সেগুলি কাস্টমাইজ করতে পারেন। এটি কখনই প্যাকেজ করা ফাইল সম্পাদনা করে না।

```sh
argvus setup --copy <app>          # copy one packaged app
argvus setup --copy-all            # copy every packaged app directory
argvus setup --copy <app> --force  # back up the existing destination first
argvus setup --copy <app> --dry-run
```

ইতিমধ্যেই বিদ্যমান একটি গন্তব্য রিপোর্ট করা হয়েছে এবং একা ছেড়ে দেওয়া হয়েছে, যদি না `--force ` দেওয়া হয়। `--force ` এর সাথে, অনুলিপি করার আগে বিদ্যমান ডিরেক্টরির নাম পরিবর্তন করে `<name>.bak-<timestamp>` করা হয়েছে।

`argvus ` অ্যাপটি বিশেষ: এটি তার প্যাকেজ করা ডিরেক্টরির বিষয়বস্তু কনফিগারেশন রুটে কপি করে। যেহেতু সেটআপ চলাকালীন কনফিগারেশন রুট ইতিমধ্যেই বিদ্যমান থাকে, বর্তমান কোডে `--copy argvus `-এর `--force` প্রয়োজন৷

সেটআপ কিছু পরিবর্তন করার আগে প্রতিটি বিকল্প যাচাই করে। অবৈধ অ্যাপের নাম (খালি, `.` দিয়ে শুরু, বা `/` আছে) প্রত্যাখ্যান করা হয়েছে।

## সেশন এবং পুনরায় লোড

`SUPER + SHIFT + R ` ` argvus sessionctl reload ` চালায়। সেশন কন্ট্রোলার জেনারেট করা কনফিগারেশন প্রয়োগ করে এবং শুধুমাত্র তার মালিকানাধীন উপাদানগুলি পুনরায় চালু করে। এটি হোম ডিরেক্টরিতে `/usr/share/argvus` কপি করে না এবং এটি একটি বিকৃত JSON ফাইল প্রতিস্থাপন করে না।

## রাতের আলো

**ডেস্কটপ ও ইনপুট > ডিসপ্লে > নাইট লাইট** খুলুন অথবা `argvus control-center displays night-light ` চালান। ` argvus config display night-light` এর মাধ্যমে তাপমাত্রা, ম্যানুয়াল অ্যাক্টিভেশন এবং নির্দিষ্ট স্থানীয় সময়সূচী কনফিগার করুন। কমান্ড, অবস্থা এবং রানটাইম প্রয়োজনীয়তার জন্য [নাইট লাইট](/docs/user-guide/night-light/) দেখুন।

`argvus tui-terminal --frame TITLE FOOTER COMMAND [ARG...]` শেয়ার করা ARGVUS টার্মিনাল ফ্রেমের ভিতরে একটি ইন্টারেক্টিভ কমান্ড চালায়। Ctrl+E ফ্রেম বন্ধ করে; এর কলার স্থানীয় ফুটার সরবরাহ করে। এটিকে একটি টার্মিনালের ভিতরে ব্যবহার করুন, বা ` argvus tui-terminal --term kitty -- ...`-তে পাঠানো কমান্ড হিসাবে।` argvus tui-terminal --term kitty --profile ai-agents ` লুকানো ট্যাব এবং শুধুমাত্র ক্লিপবোর্ড শর্টকাট সহ একটি ডেডিকেটেড কিটি কনফিগারেশন ব্যবহার করে। কলার কমান্ড এবং শিরোনাম সরবরাহ করে; ` argvus-ai` ক্লাসের জন্য উইন্ডো প্লেসমেন্ট হাইপ্রল্যান্ড নিয়ম দ্বারা পরিচালিত হয়।
