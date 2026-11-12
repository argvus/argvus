---
title: অ্যাকাউন্ট
description: স্থানীয় ব্যবহারকারীর প্রোফাইল, ডিসপ্লে নাম, অবতার এবং গ্রুপ সদস্যতা, আরগভাস অ্যাকাউন্টের মাধ্যমে।
---

`argvus accounts` ARGVUS ডেস্কটপের স্থানীয় ব্যবহারকারী অ্যাকাউন্টগুলিকে তালিকাভুক্ত করে এবং সম্পাদনা করে। এটি একটি ব্যক্তিগত
`argvus ` প্যাকেজের এক্সিকিউটেবল, `/usr/lib/argvus/commands/accounts` এ ইনস্টল করা। দ
`argvus` কমান্ড এটি শুরু করে, এবং শুধুমাত্র ক্রিয়াকলাপ যা অন্যান্য অ্যাকাউন্ট বা সিস্টেম ডেটা পরিবর্তন করে
রুট হিসাবে চালান।

## কমান্ড

```text
argvus accounts list [--all]                   list users (--all includes system accounts)
argvus accounts show USER                      show account details
argvus accounts name USER "DISPLAY NAME"       change another user's display name
argvus accounts avatar USER IMAGE              set another user's avatar
argvus accounts avatar USER --remove           remove another user's avatar
argvus accounts groups USER                    list groups
argvus accounts groups USER --add G --remove G change groups (authorization required)
argvus accounts passwd USER                    change a password (reads three lines from stdin)
argvus accounts self                           show your own account
argvus accounts self name "DISPLAY NAME"       change your own display name
argvus accounts self avatar IMAGE              set your avatar
argvus accounts self avatar --remove           remove your avatar
argvus accounts self groups                    list your groups
argvus accounts --verbose ...                  print diagnostic information to stderr
```

## নাম এবং অবতার প্রদর্শন করুন

প্রদর্শনের নামটি স্ট্যান্ডার্ড GECOS ক্ষেত্রে সংরক্ষণ করা হয়। ইউনিক্স ব্যবহারকারীর নাম কখনই পরিবর্তন করা হয় না।

একটি অবতার হল PNG, JPEG বা WebP ফর্ম্যাটে 20 MiB পর্যন্ত একটি ছবি। চিত্রটির আকার 256×256 করা হয়েছে
পিক্সেল এবং `0644 ` মোড সহ PNG হিসাবে `~/.face`-তে সংরক্ষিত, ফ্রিডেস্কটপ অবস্থান
অভিবাদন এবং বেশিরভাগ ডিসপ্লে ম্যানেজার দ্বারা। আপনার নিজের অবতার আপনার হোম ডিরেক্টরির ভিতরে লেখা আছে এবং
অনুমোদনের প্রয়োজন নেই।

## অনুমোদন

শুধুমাত্র `list `, ` show `, ` self `, ` self groups` এবং আপনার নিজস্ব অবতার কাজ অনুমোদন ছাড়াই। প্রতিটি
অন্য পরিবর্তন ডেস্কটপের পোলকিট এজেন্টকে অনুমোদনের জন্য জিজ্ঞাসা করে। ARGVUS নিয়ম
(`/usr/share/polkit-1/rules.d/50-argvus-accounts.rules`) সিদ্ধান্ত নেয় কে অবশ্যই প্রমাণীকরণ করবে:

- একটি অ্যাডমিনিস্ট্রেটর গ্রুপের একজন ব্যবহারকারী (`wheel `, আর্চ ডিফল্ট, বা ` sudo`) তাদের নিজস্ব দ্বারা নিশ্চিত করে
  পাসওয়ার্ড;
- অন্য কোনো ব্যবহারকারী রুট পাসওয়ার্ড দিয়ে নিশ্চিত করে।

| অপারেশন | অনুমোদন ছাড়া | অনুমোদন সহ |
| --- | --- | --- |
| ব্যবহারকারীদের তালিকা করুন, একটি অ্যাকাউন্ট দেখান, গ্রুপ তালিকা করুন | ✓ | ✓ |
| আপনার নিজের অবতার পরিবর্তন করুন | ✓ | ✓ |
| আপনার নিজের প্রদর্শন নাম পরিবর্তন করুন | — | ✓ |
| আপনার নিজের পাসওয়ার্ড পরিবর্তন করুন | — | ✓ (বর্তমান পাসওয়ার্ড প্রমাণ করার পরে) |
| অন্য ব্যবহারকারীর নাম, অবতার, গোষ্ঠী বা পাসওয়ার্ড পরিবর্তন করুন | — | ✓ |

আপনার নিজের পাসওয়ার্ড পরিবর্তন করা প্রথমে বর্তমান পাসওয়ার্ড চেক করে। একটি ভুল পাসওয়ার্ড প্রত্যাখ্যান করা হয়েছে
অনুমোদন প্রম্পট প্রদর্শিত হওয়ার আগে।

`sudo` দিয়ে কমান্ড চালানোর দরকার নেই। কমান্ড কখন অনুমোদনের জন্য পোলকিটকে জিজ্ঞাসা করে
এটা প্রয়োজন.

## নিরাপত্তা সীমা

- `passwd` স্ট্যান্ডার্ড ইনপুট থেকে বর্তমান, নতুন এবং নিশ্চিতকরণ পাসওয়ার্ড পড়ে, প্রতি লাইনে একটি,
  এবং তাদের যুক্তি হিসাবে নেয় না। বর্তমান পাসওয়ার্ড লাইন খালি হতে পারে যখন একজন প্রশাসক
  অন্য ব্যবহারকারীর পাসওয়ার্ড রিসেট করে। উদাহরণ:

  ```sh
  printf '%s\n%s\n%s\n' "$OLD" "$NEW" "$NEW" | argvus accounts passwd ghost
  ```

  ইন্টারেক্টিভভাবে, তিনটি লাইন টাইপ করুন এবং প্রতিটির পরে এন্টার টিপুন। ইনপুট দেখানো হয়
  টার্মিনাল যেমন টাইপ করা হয়।
- UID 1000 (এবং `nobody`) এর নীচে একটি অ্যাকাউন্ট সম্পাদনা করা প্রত্যাখ্যান করা হয়েছে, এবং তাই এটি লক করা হচ্ছে, এটি মুছে ফেলা হচ্ছে
  এবং এর পাসওয়ার্ডের মেয়াদ শেষ হচ্ছে। আপনার নিজের অ্যাকাউন্ট লক বা মুছে ফেলা যাবে না.
- GID 1000 এর নিচের একটি গোষ্ঠী বা প্রশাসক গোষ্ঠী (`wheel `, ` sudo`) এর নাম পরিবর্তন করা বা মুছে ফেলা হল
  প্রত্যাখ্যান এর সদস্যদের এখনও পরিবর্তন করা যেতে পারে, আপনি নিজেকে একটি থেকে সরাতে পারবেন না
  প্রশাসক গ্রুপ।
- প্রশাসক সদস্যপদ হল `wheel ` বা ` sudo` গ্রুপ। কন্ট্রোল সেন্টার নতুন যোগ করে
  প্রশাসক `wheel ` যখন সিস্টেমে থাকে, এবং অন্যথায় ` sudo`-এর কাছে। আপনার নিজের অপসারণ
  একটি সম্পাদনা মাধ্যমে প্রশাসক সদস্যপদ প্রত্যাখ্যান করা হয়.

## উদাহরণ

```sh
# Set your display name and avatar after the first login
argvus accounts self name "William Canin"
argvus accounts self avatar ~/Pictures/profile.png

# Check your groups, then add another user to the wheel group (authorization required)
argvus accounts self groups
argvus accounts groups ghost --add wheel

# Administrator: show a user's details and reset that user's name
argvus accounts show ghost
argvus accounts name ghost "Ghost Account"
```

## সমস্যা সমাধান

- `pkexec was not found `: ` polkit ` ইনস্টল করুন। ` argvus` প্যাকেজ এটির উপর নির্ভর করে।
- অনুমোদন প্রম্পট প্রদর্শিত হয় না: একটি পোলকিট প্রমাণীকরণ এজেন্ট চলছে কিনা তা পরীক্ষা করুন
  আপনার অধিবেশনে। `argvus ` প্যাকেজ ` hyprpolkitagent` এর উপর নির্ভর করে, যা একটি প্রদান করে।
- একটি পাসওয়ার্ড পরিবর্তন করার সময় `unix_chkpwd ` ত্রুটি: ` pam` প্যাকেজ সাহায্যকারী প্রদান করে।
- stderr এ কমান্ডের ধাপগুলি প্রিন্ট করতে `argvus accounts --verbose ...` চালান।

বিশেষাধিকারের সীমানা সম্পর্কে বিকাশকারী নোটের জন্য, `docs/internal/argvus-accounts/` দেখুন।যখন একজন প্রশাসক স্বয়ংক্রিয় লগইনের জন্য কনফিগার করা অ্যাকাউন্টটি লক করে বা সরিয়ে দেয়, তখন অ্যাকাউন্ট প্রদানকারী প্রথমে তার স্বয়ংক্রিয়-লগইন এন্ট্রি অক্ষম করে। সেই পরিচ্ছন্নতা ব্যর্থ হলে, অ্যাকাউন্ট অপারেশন প্রত্যাখ্যান করা হয়। লকিং বা অপসারণ পরে ব্যর্থ হলে, স্বয়ংক্রিয় লগইন নিষ্ক্রিয় থাকে। অন্য অ্যাকাউন্টের জন্য স্বয়ংক্রিয় লগইন সংরক্ষিত হয়; একটি অ্যাকাউন্ট আনলক করা স্বয়ংক্রিয় লগইন সক্ষম করে না। বিদ্যমান সেশন বন্ধ করা হয় না এবং অভিবাদন পুনরায় আরম্ভ করা হয় না।
