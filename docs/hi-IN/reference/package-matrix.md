---
title: पैकेज मैट्रिक्स
description: ARGVUS पैकेज जिम्मेदारियाँ।
---

| पैकेज | जिम्मेदारी |
| --- | --- |
| `argvus` | मास्टर पैकेज: डिस्पैचर, इन-प्रोसेस कमांड (कॉन्फ़िगरेशन, अनुवाद, कुंजी, प्रोजेक्ट, टर्मिनल, सिस्टम मॉनिटर, नेटवर्क, ब्लूटूथ, टेलीमेट्री, टीयूआई टर्मिनल, सेटअप), साझा डेटा और i18n कैटलॉग। प्रत्येक घटक पैकेज इस पर निर्भर करता है। |
| `argvus` (सहायक) | एक ही पैकेज में सहायक: नियंत्रण केंद्र और डिस्क उपयोग विश्लेषक (टर्मिनल एप्लिकेशन), साथ ही टास्कबार कैलेंडर के लिए जीटीके 4 एप्लिकेशन, हटाने योग्य डिवाइस, लोडिंग ओवरले, एआरजीवीयूएस फोकस और एआरजीवीयूएस क्लॉक। |
| `argvus` (अभिवादक) | अभिवादन के लिए लॉगिन अभिवादक, एक ही पैकेज में भेजा गया और `crates/argvus-greeter` से निर्मित। |
| `argvus-desktop` | `argvus` मास्टर पैकेज द्वारा आवश्यक डेस्कटॉप और रोजमर्रा की एप्लिकेशन प्रोफ़ाइल। यह `argvus` पर निर्भर नहीं है, इसलिए निर्भरता एकतरफ़ा रहती है। |
| `argvus-devtools`, `argvus-pentest` | विकास उपकरण और अधिकृत सुरक्षा ऑडिटिंग के लिए वैकल्पिक स्वतंत्र मेटापैकेज। मास्टर पैकेज़ इन प्रोफ़ाइलों को स्थापित नहीं करता है. |
| `argvus-session` | सत्र प्रवेश बिंदु और उपयोगकर्ता जीवनचक्र। |
| `argvus-hyprland` | हाइप्रलैंड विन्यास. |
| `argvus-control-panel` | क्विकशेल नियंत्रण कक्ष. |
| `argvus-taskbar`, `argvus-waybar` | वेबार कॉन्फ़िगरेशन और पैच बाइनरी। |
| `argvus-branding` | लोगो और वर्डमार्क संपत्ति. |
| `argvus-appearance`, `argvus-wallpapers`, `argvus-fonts`, `argvus-icons` | उपस्थिति स्थिति और दृश्य संपत्ति। |
| `argvus-display`, `argvus-power`, `argvus-notifications` | डेस्कटॉप प्रदाता. |
| `argvus-lock`, `argvus-boot-splash` | लॉक स्क्रीन और बूट स्प्लैश। |
| `argvus-firewall` | फ़ायरवॉल, अपने विशेषाधिकारों के साथ। |
| `argvus-launcher` | लॉन्चर कॉन्फ़िगरेशन और प्रवेश बिंदु। |
| `argvus-app-profiles` | बंडल किए गए टर्मिनल अनुप्रयोगों के लिए डेटा-केवल प्रोफ़ाइल। |
| `argvus-portal` | XDG पोर्टल और वेलैंड/DBus डिफ़ॉल्ट। |
| `argvus-theme-*` | प्रति आधिकारिक थीम परिवार एक वैकल्पिक पैकेज। |
| `argvus-game-snake`, `argvus-games` | वैकल्पिक। साँप एक स्वतंत्र परियोजना है; `argvus-games` वैकल्पिक खेलों को समूहित करता है। |

## बदले गए पैकेज

`argvus` अपने द्वारा अवशोषित पैकेजों के लिए `replaces`, `conflicts` और `provides` की घोषणा करता है, इसलिए अपग्रेड उन्हें हटा देता है और उनकी फ़ाइलें अनाथ के रूप में नहीं रहती हैं:

`argvus-config`, `argvus-i18n` (`argvus-language-en-us` और `argvus-language-pt-br` के साथ), `argvus-keys`, `argvus-projects`, `argvus-terminal`, `argvus-system-monitor`, `argvus-network`, `argvus-widget-telemetry` और `argvus-du`। `argvus`, `argvus-control-center`, `argvus-taskbar-calendar`, `argvus-loading-theme`, `argvus-removable-devices` और पूर्व विभाजित पैकेज `argvus-gtk` और को भी प्रतिस्थापित करता है। `argvus-greeter`, जो यह अभी भी प्रदान करता है ताकि उनके आधार पर पैकेज हल होते रहें। खाता प्रबंधक (`argvus accounts`) `crates/argvus-accounts` से बनाया गया है और इसके पोलकिट नियम के साथ `/usr/lib/argvus/commands/accounts` के रूप में स्थापित किया गया है। डिस्क उपयोग विश्लेषक `crates/argvus-du` से बनाया गया है और `/usr/lib/argvus/commands/argvus-du` के रूप में स्थापित किया गया है।

`/usr/bin/argvus-terminal`, `argvus terminal` के लिए संगतता लांचर के रूप में स्थापित रहता है। हाइप्रलैंड का डिफ़ॉल्ट टर्मिनल, कंट्रोल सेंटर कैटलॉग और सहेजे गए डिफ़ॉल्ट एप्लिकेशन अभी भी उस कमांड को नाम देते हैं।

हटाए गए पैकेज `argvus-pwvucontrol` और `argvus-snappy-switcher` किसी भी पैकेज द्वारा प्रदान नहीं किए गए हैं।

ARGVUS घटक पैकेजों के बीच किसी इंस्टाल ऑर्डर की आवश्यकता नहीं है: `argvus` उनका मास्टर पैकेज है, और प्रत्येक घटक इस पर निर्भर करता है। `argvus` इंस्टॉल करने से `argvus-desktop` प्रोफ़ाइल भी इंस्टॉल हो जाती है; `argvus-devtools` और `argvus-pentest` ऑप्ट-इन बने रहेंगे। इस तालिका में एक पैकेज की उपस्थिति का मतलब यह नहीं है कि यह पूरी सुविधा का मालिक है: उदाहरण के लिए, टास्कबार `argvus-waybar`, `argvus-taskbar`, `argvus-session` और डोमेन प्रदाताओं को जोड़ता है।