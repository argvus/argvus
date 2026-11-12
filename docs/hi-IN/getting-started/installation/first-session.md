---
title: पहला सत्र
description: इंस्टालेशन के बाद ARGVUS प्रारंभ करें।
---

ग्राफिकल लॉगिन के लिए, पैकेज्ड greetd इंटीग्रेशन को कॉन्फ़िगर करें `argvus greeter-setup --help`और वर्णित डिस्प्ले-मैनेजर प्रवाह का उपयोग करें[सत्र](/docs/user-guide/sessions/).

TTY सत्र के लिए, वर्चुअल टर्मिनल पर लॉग इन करें और चलाएं:

```sh
argvus --start-desktop
```

लॉगिन करने के बाद, ARGVUS उपयोगकर्ता लक्ष्य का निरीक्षण करने के लिए `argvus sessionctl status` का उपयोग करें। यदि कंपोज़िटर या शेल प्रारंभ नहीं होता है, तो [सत्र समस्या निवारण](/docs/user-guide/troubleshooting/session-startup/) का पालन करें।
