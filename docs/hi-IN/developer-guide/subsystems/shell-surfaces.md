---
title: शैल सतहें
description: टास्कबार, नियंत्रण कक्ष और टेलीमेट्री स्वामित्व।
---

`argvus-waybar` पैच किया गया `/usr/bin/waybar` प्रदान करता है। `argvus-taskbar` के पास टास्कबार एक्शन और UX है; टास्कबार JSONC और CSS इसे `config.json` के प्रक्षेपण हैं, और `argvus-config` `data/generated/waybar/` के तहत सक्रिय परत लिखता है और `data/waybar/` के तहत एक प्रतिबिंबित प्रतिलिपि लिखता है। `argvus-control-panel` के पास क्विकशेल कार्ड हैं। `argvus-widget-telemetry` वैकल्पिक टेलीमेट्री बार और इसके ब्लॉक सिमेंटिक्स का मालिक है; इसके JSONC और CSS को उपस्थिति परिवर्तन पर पैकेज्ड डिफ़ॉल्ट से `data/waybar/` पर लिखा जाता है, जिसके बाद `argvus-config` केवल सीमांकित `ARGVUS_TELEMETRY_*` और फ़ॉन्ट ब्लॉक को फिर से लिखता है। `argvus-session` उनकी उपयोगकर्ता सेवाओं का स्वामी है।

इस रनटाइम मॉडल में कोई मौजूदा `argvus-shell` पैकेज या `argvus-shell.service` नहीं है।