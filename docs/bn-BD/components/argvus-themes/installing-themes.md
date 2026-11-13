---
title: থিম ইনস্টল করা হচ্ছে
description: প্যাকম্যানের সাথে অফিসিয়াল ARGVUS থিম প্যাকেজ ইনস্টল করুন।
---

অফিসিয়াল থিমগুলি আর্চ লিনাক্স প্যাকেজ হিসাবে প্রকাশিত হয়। প্রতিটি থিম `argvus-theme-<THEME_NAME>` নামের নিজস্ব প্যাকেজ।

## কোথায় থিম পাবেন

প্রতিটি অফিসিয়াল থিমের উত্স ARGVUS প্রকল্প সংস্থায় থাকে:

- প্রতিটি থিম হল `argvus-theme-<THEME_NAME>` (যেমন ` argvus-theme-dracula`) নামের একটি সংগ্রহস্থল, একই নামের একটি প্যাকেজ হিসাবে নির্মিত৷

থিম প্যাকেজ `argvus-appearance` এর উপর নির্ভর করে। প্রথমে ARGVUS ইনস্টল করুন, যেমনটি [ইনস্টলেশন](/docs/getting-started/installation/) এ বর্ণিত হয়েছে, যাতে উপস্থিতি উপাদানটি উপস্থিত থাকে।


Git দিয়ে ইনস্টল করতে শুধু `src/`-এর থিম ফাইল লাগে। `packaging/` ঐচ্ছিক; এটি Arch Linux প্যাকেজ তৈরির জন্য।

## একটি একক থিম ইনস্টল করুন

প্রতিটি অফিসিয়াল থিম একবারে ইনস্টল করতে, `argvus-themes` মেটাপ্যাকেজ ইনস্টল করুন:

```bash
sudo pacman -S argvus-themes
```

এটি সমস্ত প্রকাশিত `argvus-theme-*` প্যাকেজের উপর নির্ভর করে। ARGVUS অন্ধকার এবং ARGVUS
আলো ইতিমধ্যে `argvus-appearance` দ্বারা পাঠানো হয়েছে.

প্যাকেজের নাম দিয়ে একটি থিম ইনস্টল করুন:

```bash
sudo pacman -S argvus-theme-<THEME_NAME>
```

যেমন:

```bash
sudo pacman -S argvus-theme-dracula
sudo pacman -S argvus-theme-rose-pine
```

প্যাকেজটি থিম ম্যানিফেস্ট, `/usr/share/argvus/themes/<theme-id>/theme.toml ` (এর নাম, বিভাগ, ওয়ালপেপার এবং প্যালেট, ঐচ্ছিক ` overrides/` সহ) এবং `/usr/share/backgrounds/argvus/` এর অধীনে থিমের ওয়ালপেপার ইনস্টল করে৷ থিম প্রয়োগ করা হলে প্রতিটি উপাদানের ফাইল প্যালেট থেকে রেন্ডার করা হয়। এটি আপনার হোম ডিরেক্টরিতে লিখবে না এবং এটি আপনার বর্তমান থিম পরিবর্তন করবে না। পরে নিয়ন্ত্রণ কেন্দ্র থেকে এটি নির্বাচন করুন, [অফিসিয়াল থিম](/docs/components/argvus-themes/official-themes/) দেখুন।

## বেশ কয়েকটি থিম ইনস্টল করুন

এমন কোনো মেটা-প্যাকেজ নেই যা প্রতিটি থিম ইনস্টল করে। একটি কমান্ডে আপনি যে থিমগুলি চান তা ইনস্টল করুন:

```bash
sudo pacman -S argvus-theme-dracula argvus-theme-sunset
```
## উপলব্ধ প্যাকেজ

নীচের প্যাকেজগুলি আজ প্রকাশিত অফিসিয়াল থিম প্যাকেজ।

**অন্ধকার থিম**

| প্যাকেজ | থিম |
| --- | --- |
| `argvus-theme-one-dark` | এক অন্ধকার |
| `argvus-theme-dracula` | ড্রাকুলা |
| `argvus-theme-silver-dark` | সিলভার ডার্ক |
| `argvus-theme-slate-dark` | স্লেট ডার্ক |
| `argvus-theme-universe` | মহাবিশ্ব |
| `argvus-theme-gruvbox-high-dark` | গ্রুভবক্স হাই ডার্ক |
| `argvus-theme-gruvbox-dark` | গ্রুভবক্স ডার্ক |
| `argvus-theme-rose-pine` | রোজ পাইন |
| `argvus-theme-tokyo-night` | টোকিও নাইট |
| `argvus-theme-solitude` | নির্জনতা |
| `argvus-theme-sunset` | সূর্যাস্ত |
| `argvus-theme-matrix` | ম্যাট্রিক্স |
| `argvus-theme-monokai-dark` | মনোকাই অন্ধকার |

**হালকা থিম**

| প্যাকেজ | থিম |
| --- | --- |
| `argvus-theme-github-light` | গিটহাব লাইট |
| `argvus-theme-solarized-light` | সোলারাইজড লাইট |
| `argvus-theme-nord-light` | নর্ড লাইট |
| `argvus-theme-one-light` | এক আলো |
| `argvus-theme-everforest-light` | এভারফরেস্ট আলো |
| `argvus-theme-catppuccin-latte` | Catppuccin Latte |
| `argvus-theme-gruvbox-light` | গ্রুভবক্স লাইট |
| `argvus-theme-watercolor` | জলরঙ |

অন্তর্নির্মিত **ARGVUS Dark** এবং **ARGVUS Light** থিমগুলির জন্য আলাদা প্যাকেজের প্রয়োজন নেই; তারা `argvus-appearance` এর সাথে আসে।

## কি ইনস্টল করা আছে তা পরীক্ষা করুন

ইনস্টল করা থিম প্যাকেজগুলির সাথে তালিকা করুন:

```bash
pacman -Qs argvus-theme
```

কন্ট্রোল সেন্টার খুলুন এবং **এপিয়ারেন্স → থিম → অফিসিয়াল থিম** এ যান যে থিমগুলি বর্তমানে সিলেক্টরে পাওয়া যাচ্ছে। একটি থিম যা ইনস্টল করা আছে কিন্তু সেই তালিকা থেকে অনুপস্থিত একটি প্যাকেজিং সমস্যা; [থিম গাইড](/docs/components/argvus-themes/) এর সমস্যা সমাধানের বিভাগটি দেখুন।

## আপনার নিজস্ব একটি থিম যোগ করুন

একটি থিম একটি ম্যানিফেস্ট, তাই আপনার নিজের একটি থিমের কোনো প্যাকেজের প্রয়োজন নেই৷ তৈরি করুন
`~/.local/share/argvus/themes/<id>/theme.toml`, যেখানে ডিরেক্টরির নামটি
থিম আইডি (ছোট হাতের অক্ষর, সংখ্যা এবং একক হাইফেন):

```toml
schema = 2
id = "mist"
name = "Mist"
category = "dark"
wallpaper = "/usr/share/backgrounds/argvus/argvus-dark.jxl"

[palette]
bg = "#101418"
fg = "#D8DEE9"
accent = "#88C0D0"
```

শুধুমাত্র `bg `, ` fg ` এবং ` accent` প্রয়োজন: প্রতিটি অন্য রঙের একটি ডিফল্ট আছে
তাদের থেকে উদ্ভূত, এবং ম্যানিফেস্ট শুধুমাত্র এটি পরিবর্তন করার জন্য একটি সংজ্ঞায়িত করে। পরীক্ষা করে দেখুন,
তারপর অন্য কোন থিমের মত এটি প্রয়োগ করুন:

```bash
argvus theme check ~/.local/share/argvus/themes/mist
argvus theme apply mist
```

`argvus theme list` অফিসিয়াল থিমগুলির পাশে আপনার থিমগুলি দেখায় (একজন কর্মকর্তা
একই আইডি সহ থিম জেতে)। আপনার থিমগুলি থিম মেনুতে প্রদর্শিত হবে৷
(`SUPER + Shift + T`); প্রি-লগইন গ্রিটার আপনার হোম ডিরেক্টরি পড়তে পারে না, তাই
আপনার থিমগুলির একটি সক্রিয় থাকাকালীন এটি আপনার প্রয়োগ করা শেষ অফিসিয়াল থিমটিকে রাখে৷

পুরানো ARGVUS সংস্করণের অধীনে লেখা প্রতি-থিম অনুলিপি`~/.config/argvus/data/<app>/themes/` আর পড়া হয় না। প্রথম থিম সুইচ
আপডেট তাদের `~/.config/argvus/data/backups/theme-copies-<time>/` এ নিয়ে যাওয়ার পর;
আপনার সম্পাদনা করা একটি ফাইল রাখতে, এটিকে `overrides/` এর অধীনে আপনার নিজস্ব একটি থিমে রাখুন এবং
ম্যানিফেস্টের `[overrides]` টেবিলে এটির নাম দিন।

## একটি থিম সরান

এর সাথে একটি একক থিম প্যাকেজ সরান:

```bash
sudo pacman -R argvus-theme-<THEME_NAME>
```

একটি থিম প্যাকেজ সরানো আপনার আমদানি করা প্রোফাইল মুছে ফেলবে না। যদি সরানো থিমটি বর্তমানে প্রয়োগ করা হয় তবে এটি সরানোর আগে নিয়ন্ত্রণ কেন্দ্র থেকে অন্য একটি থিম নির্বাচন করুন৷

## skeleton দিয়ে থিম তৈরি

সামঞ্জস্যপূর্ণ থিম বানাতে `argvus-skeleton-theme` সংগ্রহস্থল দিয়ে শুরু করুন। `make new ID=ocean-night NAME="Ocean Night" CATEGORY=dark` চালান, তারপর রং ও ওয়ালপেপার সম্পাদনা করুন। `make check` দিয়ে যাচাই এবং `make try` দিয়ে পরীক্ষা করুন। সংগ্রহস্থল প্রকাশের পর `argvus theme install URL` দিয়ে ইনস্টল করা যায়। থিম থাকে `src/`-এ; Arch প্যাকেজ বানাতে `packaging/` ঐচ্ছিক।

```sh
git clone https://github.com/argvus/argvus-skeleton-theme.git argvus-theme-ocean-night
cd argvus-theme-ocean-night
make new ID=ocean-night NAME="Ocean Night" CATEGORY=dark
make check
make try
argvus theme apply ocean-night
```

## Git থেকে থিম ইনস্টল

বিদ্যমান Git পরিচয়পত্র দিয়ে HTTPS বা SSH-তে skeleton সংগ্রহস্থল ইনস্টল করুন। প্রতি ব্যবহারকারীর জন্য ইনস্টল হয়, স্বয়ংক্রিয়ভাবে প্রয়োগ হয় না। `--ref` শাখা বা ট্যাগ বেছে নেয়; `--replace` বিদ্যমান নিজস্ব থিম বদলায়। ইনস্টল করা সরকারি ID সুরক্ষিত।

```sh
argvus theme install https://github.com/USER/argvus-theme-NAME.git
argvus theme install https://gitlab.com/USER/argvus-theme-NAME.git --ref v1.0
argvus theme install git@host:USER/argvus-theme-NAME.git --replace
argvus theme apply NAME
argvus theme uninstall NAME
```

থিম **Custom themes**-এ দেখা যাবে; `r` দিয়ে হালনাগাদ করুন। CLI বা Control Center-এ সক্রিয় থিম মুছতে আগে অন্য থিম প্রয়োগ করুন। প্রতিস্থাপনের পর আবার প্রয়োগ করুন।

`src/usr/share/argvus/themes/<id>/`-এ মিলযুক্ত ID সহ একটি schema 2 থিম থাকতে হবে। ছবি থিমে বা `src/usr/share/`-এ রাখুন। স্ক্রিপ্ট চলে না; সাবমডিউল ও Git LFS সমর্থিত নয়। `git` এবং SSH-এর জন্য `openssh` প্রয়োজন। গন্তব্য `$XDG_DATA_HOME/argvus/themes/<id>`। সরকারি প্যাকেজ pacman দিয়ে পরিচালিত হয়।
