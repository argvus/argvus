---
title: शैल सतहें
description: टास्कबार, नियंत्रण कक्ष और टेलीमेट्री स्वामित्व।
---

`argvus-shell` पैच किया गया `Quickshell Taskbar` प्रदान करता है। `argvus-shell` के पास टास्कबार एक्शन और UX है; टास्कबार JSONC और CSS इसे `config.json` के प्रक्षेपण हैं, और `argvus-config` `data/generated/taskbar/` के तहत सक्रिय परत लिखता है और `data/taskbar/` के तहत एक प्रतिबिंबित प्रतिलिपि लिखता है। `argvus-shell` के पास क्विकशेल कार्ड हैं। `argvus-shell` provides the optional Telemetry surface, while the `argvus widget-telemetry` command in `argvus` retains block-control semantics; इसके JSONC और CSS को उपस्थिति परिवर्तन पर पैकेज्ड डिफ़ॉल्ट से `data/taskbar/` पर लिखा जाता है, जिसके बाद `argvus-config` केवल सीमांकित `ARGVUS_TELEMETRY_*` और फ़ॉन्ट ब्लॉक को फिर से लिखता है। `argvus-session` उनकी उपयोगकर्ता सेवाओं का स्वामी है।

इस रनटाइम मॉडल में कोई मौजूदा `argvus-shell` पैकेज या `argvus-shell.service` नहीं है।