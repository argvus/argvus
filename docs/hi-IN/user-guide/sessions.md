---
title: सत्र
description: एक ARGVUS ग्राफ़िकल सत्र प्रारंभ और प्रबंधित करें।
---

`argvus-session` सत्र प्रवेश बिंदु और `systemd --user` जीवनचक्र का मालिक है। यह `argvus start` के माध्यम से हाइपरलैंड शुरू करता है, सत्र वातावरण आयात करता है और ARGVUS लक्ष्य का प्रबंधन करता है।

- [ग्राफ़िकल सत्र](/docs/argvus-session/graphical-session/)
- [TTY सत्र](/docs/argvus-session/tty-session/)
- [अभिवादक](/docs/components/argvus-greeter/)

उपयोगी निदान:

```sh
argvus sessionctl status
argvus sessionctl logs
argvus sessionctl restart waybar
```

सेवा स्वामित्व के लिए डेवलपर [रनटाइम जीवनचक्र](/docs/developer-guide/architecture/runtime-lifecycle/) को देखें।