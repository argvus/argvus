---
title: फ़ाइल स्थान
description: महत्वपूर्ण स्थापित ARGVUS फ़ाइलें और निर्देशिकाएँ।
---

- `/usr/bin/argvus*` - सार्वजनिक आदेश।
- `/usr/share/argvus/hyprland/` - हाइप्रलैंड कॉन्फ़िगरेशन और सहायक।
- `/usr/share/argvus/control-panel/` - कंट्रोल पैनल QML और स्क्रिप्ट।
- `/usr/share/argvus/taskbar/` - टास्कबार कॉन्फ़िगरेशन और स्क्रिप्ट।
- `/usr/share/argvus/lock/` - हाइप्रलॉक कॉन्फ़िगरेशन।
- `/usr/share/argvus/portal/` - पोर्टल एकीकरण फ़ाइलें।
- `/usr/share/backgrounds/argvus/` — वॉलपेपर संपत्ति।
- `/etc/argvus/` - सिस्टम प्रदाता कॉन्फ़िगरेशन।
- `/etc/argvus/greeter.toml` — अभिवादक विन्यास।
- `/etc/argvus/removable-devices/config.json` - हटाने योग्य भंडारण के लिए सिस्टम डिफ़ॉल्ट।
- `$XDG_CONFIG_HOME/argvus/config.json` — विहित पोर्टेबल प्राथमिकताएँ।
- `$XDG_CONFIG_HOME/argvus/data/` - प्रबंधित घटक डेटा, बैकअप, लॉक और जेनरेटेड आउटपुट।
- `$XDG_CONFIG_HOME/waybar/argvus-taskbar.{jsonc,css}` - वैकल्पिक पूर्ण देशी वेबार टास्कबार ओवरराइड।
- `$XDG_CONFIG_HOME/argvus/data/taskbar/argvus-taskbar.{jsonc,css}` - प्रबंधित ARGVUS टास्कबार प्रतियां।
- `$XDG_CONFIG_HOME/argvus/data/generated/waybar/` - `argvus-taskbar.css` और `argvus-widget-telemetry.{jsonc,css}` सहित जेनरेटेड वेबार प्रोफाइल; इन्हें उपरोक्त प्रतियों पर प्राथमिकता दी जाती है।
- `$XDG_CONFIG_HOME/argvus/data/generated/waybar/argvus-widget-telemetry.{jsonc,css}` - `data/waybar/` के अंतर्गत भी प्रतिबिंबित होता है, जिसे उपस्थिति परिवर्तन पर पैक किए गए डिफ़ॉल्ट से बदल दिया जाता है और फिर इसके `ARGVUS_TELEMETRY_*` और फ़ॉन्ट ब्लॉक को फिर से लिखा जाता है।
- `$XDG_CONFIG_HOME/argvus/data/generated/yazi/` - याज़ी कॉन्फ़िगरेशन ट्री का अनुमान लगाया गया है, उन विषयों के लिए वैरिएंट-अवेयर फॉलबैक के साथ, जिनमें कोई याज़ी स्वाद नहीं है।
- `$XDG_CONFIG_HOME/argvus/data/generated/superfile/` - अनुमानित सुपरफाइल थीम सेट।
- `$XDG_CONFIG_HOME/argvus/data/generated/terminal/` - अनुमानित टर्मिनल प्रोफाइल।
- `$XDG_CONFIG_HOME/argvus/data/generated/qt6ct/` - प्रक्षेपित Qt6ct पैलेट।
- `$XDG_CONFIG_HOME/argvus/data/generated/gtk/` - अनुमानित GTK थीम फ़ाइलें।
- `$XDG_CONFIG_HOME/argvus/data/rofi/` - Rofi कॉन्फ़िगरेशन, थीम और मोड की उपयोगकर्ता प्रतियां। ARGVUS प्रत्येक प्रतिलिपि तब बनाता है जब वह पहली बार फ़ाइल बदलता है; अपरिवर्तित फ़ाइलें `/usr/share/argvus/launcher/config/` से पढ़ी जाती हैं।
- `$XDG_CONFIG_HOME/argvus/data/generated/removable-devices/theme.css` - अनुमानित हटाने योग्य डिवाइस स्टाइलशीट।
- `$XDG_CONFIG_HOME/argvus/data/focus/settings.json`, `$XDG_DATA_HOME/argvus/focus/history.json`, `$XDG_STATE_HOME/argvus/focus/session.json` - ARGVUS फोकस सेटिंग्स, सत्र इतिहास और वर्तमान-सत्र चेकपॉइंट।
- `$XDG_RUNTIME_DIR/argvus/focus/status.json` - टास्कबार द्वारा उपभोग की जाने वाली क्षणिक ARGVUS फोकस स्थिति; फोकस बाहर निकलने पर हटा दिया जाता है।
- `$XDG_CONFIG_HOME/argvus/data/clock/settings.json`, `$XDG_DATA_HOME/argvus/clock/alarms.json`, `$XDG_STATE_HOME/argvus/clock/timers.json` - ARGVUS क्लॉक सेटिंग्स और विश्व घड़ियां, अलार्म और टाइमर/स्टॉपवॉच चेकपॉइंट।
- `$XDG_CONFIG_HOME/argvus/data/generated/theme/current/time/theme.css` - ARGVUS फोकस और ARGVUS क्लॉक (थीम उपभोक्ता `time`) के रंग।
- `/usr/share/argvus/sounds/` - ARGVUS फोकस और ARGVUS क्लॉक की ध्वनियाँ (`argvus-chime.oga`, `argvus-beacon.oga`, `argvus-bell.oga`)।
- `$XDG_CONFIG_HOME/argvus/data/generated/theme/` - थीम बिल्ड (`builds/<id>-<variant>-<hash>/`), `argvus theme render` द्वारा लिखित और प्रत्येक थीम स्विच और उच्चारण परिवर्तन द्वारा, और सक्रिय थीम के निर्माण के लिए `current` सिम्लिंक।
- `$XDG_CONFIG_HOME/argvus/data/hypr/` - उपयोगकर्ता हाइप्रलैंड लुआ ओवरराइड और देशी हाइप्रलैंड अनुमान।
- `$XDG_CONFIG_HOME/hypr/hyprland.lua` - `argvus start` द्वारा चयनित वैकल्पिक पूर्ण देशी हाइप्रलैंड कॉन्फ़िगरेशन; यह कोई वृद्धिशील ओवरले नहीं है.
- `$XDG_STATE_HOME/argvus/config-projection.json` - प्रक्षेपण प्रकट: पीढ़ी, प्रभावी-कॉन्फिग हैश, परिवर्तित अनुभाग और लंबित रनटाइम कार्य। `argvus sessionctl reload` द्वारा पढ़ें; सत्य का स्रोत कभी नहीं।
- `$XDG_STATE_HOME/argvus/widget-telemetry-blocks` - लीगेसी टेलीमेट्री ब्लॉक प्राथमिकताएँ। केवल माइग्रेशन इनपुट; विहित मान `config.json` में रहते हैं और यह फ़ाइल अब लिखी नहीं जाती है।
- `$XDG_STATE_HOME/argvus/session.log` - ग्राफिकल सत्र लॉग (सामान्यतः `~/.local/state/argvus/session.log`)।
- `/usr/share/wayland-sessions/argvus.desktop` - ग्राफिकल सत्र प्रविष्टि।
- `/usr/share/plymouth/themes/argvus/` - पैकेज्ड बूट स्प्लैश एसेट।

सटीक पेलोड पैकेज-विशिष्ट हैं; डेवलपर घटक संदर्भ और प्रासंगिक पैकेज मैनिफ़ेस्ट देखें।