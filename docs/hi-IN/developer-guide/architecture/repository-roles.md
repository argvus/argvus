---
title: रिपॉजिटरी भूमिकाएँ
description: कार्यक्षेत्र समेकन के बाद ARGVUS स्रोत रिपॉजिटरी की जिम्मेदारियां।
---

रिपॉजिटरी को नेविगेशन पेज के बजाय जिम्मेदारी के आधार पर समूहीकृत किया गया है:

- कार्यक्षेत्र: `argvus` (डिस्पैचर, इन-प्रोसेस कमांड, GTK 4 हेल्पर्स, साझा क्रेट, `crates/argvus-greeter` में स्वागतकर्ता और कमांड का डेटा);
- सत्र और एकीकरण: `argvus-session`, `argvus-hyprland`, `argvus-portal`, `argvus-lock`, `argvus-boot-splash`, और अभिवादक (`argvus-greeter`, से निर्मित) `argvus/crates/argvus-greeter`);
- कार्यक्षेत्र के बाहर यूआई: कंट्रोल पैनल, टास्कबार (`argvus-taskbar`), लॉन्चर और वेबार;
- प्रदाता: डिस्प्ले, पावर, फ़ायरवॉल, सूचनाएं;
- संपत्ति और पैकेजिंग: उपस्थिति, वॉलपेपर, फ़ॉन्ट, आइकन, प्रोफाइल।

## कार्यक्षेत्र में समेकित

इन रिपॉजिटरी को `de/argvus` में विलय कर दिया गया था। उनका दस्तावेज़ीकरण अब कार्यक्षेत्र के `docs/` के अंतर्गत रहता है, और उनके भंडार केवल इतिहास के लिए रखे गए हैं:

| पूर्व भंडार | अब | दस्तावेज़ीकरण |
| --- | --- | --- |
| `argvus-config` | `argvus config` | [कमांड लाइन](/docs/reference/command-line/), [कॉन्फ़िगरेशन फ़ाइलें](/docs/reference/configuration-files/) |
| `argvus-i18n` | `argvus i18n` | [अंतर्राष्ट्रीयकरण घटक](/docs/components/argvus-i18n/) |
| `argvus-keys`, `argvus-projects` | `argvus keys`, `argvus projects` | कमांड लाइन |
| `argvus-terminal` | `argvus terminal` | [टर्मिनल घटक](/docs/components/argvus-terminal/) |
| `argvus-system-monitor` | `argvus system-monitor` | [सिस्टम मॉनिटर घटक](/docs/components/argvus-system-monitor/) |
| `argvus-network` | `argvus network`, `argvus bluetooth` | [नेटवर्क घटक](/docs/components/argvus-network/) |
| `argvus-widget-telemetry` | `argvus widget-telemetry` | कमांड लाइन |
| `argvus-control-center` | `argvus control-center` (टर्मिनल एप्लिकेशन) | [नियंत्रण केंद्र घटक](/docs/components/argvus-control-center/) |
| `argvus-taskbar-calendar` | `argvus taskbar-calendar` (GTK 4 हेल्पर) | [टास्कबार कैलेंडर घटक](/docs/components/argvus-taskbar-calendar/) |
| `argvus-removable-devices` | `argvus removable-devices` (GTK 4 हेल्पर) | [हटाने योग्य उपकरण घटक](/docs/components/argvus-removable-devices/) |
| `argvus-loading-theme` | `argvus loading-theme` (`splash` सहायक) | कमांड लाइन |
| `argvus-tui-terminal` | `argvus tui-terminal` (कार्यक्षेत्र में टोकरा) | कमांड लाइन |
| `argvus-tui`, `argvus-theme` | साझा क्रेट `crates/argvus-tui` और `crates/argvus-theme` | डेवलपर नोट्स |
| `argvus-greeter` | स्रोत `crates/argvus-greeter` में, `argvus` पैकेज में भेजा गया | [अभिवादक घटक](/docs/components/argvus-greeter/) |
| `argvus-themes` | सेवानिवृत्त। प्रत्येक आधिकारिक थीम का अपना `argvus-theme-*` पैकेज है। | [थीम इंस्टॉल करना](/docs/user-guide/appearance/) |

समेकित घटकों के डेवलपर नोट `docs/internal/<repository>/` में हैं।

भाषा स्वामित्व सीमा नहीं है: प्रत्येक घटक अपने रनटाइम सहायकों को अपने पैकेज के साथ रखता है। एक कठिन नियम जो सभी पैकेजों में लागू होता है: केवल `argvus-config`, `data/generated/` के नीचे लिखता है। एक घटक जिसे ऐसे एप्लिकेशन तक पहुंचने की आवश्यकता होती है जो `config.json` नहीं पढ़ सकता है, अपने स्वयं के सहायक में एक एडाप्टर जोड़ता है और ऑर्केस्ट्रेटर को कैनोनिकल प्रतिबद्धता के बाद इसे कॉल करने देता है; यह कभी भी जेनरेट की गई फ़ाइल नहीं लिखता है। सार्वजनिक सीएलआई `/usr/bin` में हैं; आंतरिक सहायक `/usr/lib/argvus/<component>` के अंतर्गत या कमांड निर्देशिका में होते हैं जब रनटाइम पर उनकी आवश्यकता होती है; आर्किटेक्चर-स्वतंत्र डेटा `/usr/share/argvus/<component>` के अंतर्गत रहता है।

स्थापित पैकेज सीमा के लिए [पैकेज मैट्रिक्स](/docs/reference/package-matrix/) देखें।