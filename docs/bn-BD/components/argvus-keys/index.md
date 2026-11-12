---
title: কী
description: SSH এবং GPG কীগুলি argvus কী এবং argvus gpg এর মাধ্যমে পরিচালিত হয়।
---

`argvus keys ` SSH কী জোড়া পরিচালনা করে এবং ` argvus gpg` GPG কীরিং পরিচালনা করে। উভয়ই
`argvus` প্যাকেজের ইন-প্রসেস কমান্ড এবং কন্ট্রোল সেন্টারের SSH এবং GPG পৃষ্ঠাগুলি সেগুলি ব্যবহার করে।

## কমান্ড

```text
argvus keys                              show the help
argvus keys list                         list the key pairs in ~/.ssh and whether they are loaded
argvus keys generate NAME [--type ed25519|rsa] [--comment C] [--passphrase]
argvus keys load NAME                    add the key to the agent
argvus keys unload NAME                  remove the key from the agent
argvus keys unload-all                   remove every key from the agent
argvus keys delete NAME                  delete the key pair from ~/.ssh
```

কীগুলি কখনই মুদ্রিত হয় না। একটি পাসফ্রেজ টার্মিনালে টাইপ করা হয় যা `ssh-keygen` চালায় বা
`ssh-add `; এটি একটি যুক্তি হিসাবে পাস করা হয় না. অন্য ডিরেক্টরি ব্যবহার করতে ` ARGVUS_SSH_DIR` সেট করুন
`~/.ssh` এর পরিবর্তে।

## জিপিজি

```text
argvus gpg list                          keys as TSV
argvus gpg status                        API version, gnupg and pinentry
argvus gpg import FILE                   public keys only
argvus gpg import-private FILE           a private key, kept protected
argvus gpg delete FINGERPRINT --yes
argvus gpg keyserver send|recv FINGERPRINT [--yes]
```

ধ্বংসাত্মক এবং প্রকাশনা কমান্ডের প্রয়োজন `--yes`; নিয়ন্ত্রণ কেন্দ্র তার নিজের পরে এটি পাস
নিশ্চিতকরণ ARGVUS পিনেন্ট্রি একটি ব্লকের ভিতরে `gpg-agent.conf` এ লেখা হয় যা ARGVUS
মালিক; আপনার নিজের লাইন রাখা হয়.
