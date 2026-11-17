---
title: ARGVUS कैलकुलेटर
description: ARGVUS कैलकुलेटर — GTK4 / ARGVUS
---

# ARGVUS कैलकुलेटर

`argvus calculator` · launcher: **ARGVUS कैलकुलेटर**

- GTK4 / Relm4; ARGVUS theme (`time`), IBM Plex Mono.
- `+`, `-`, `*`, `/`, `%`, `±`; `50% = 0.5`.
- **वैज्ञानिक**: `sqrt`, `log`, `ln`, `exp`, `sin`, `cos`, `tan`, `asin`, `acos`, `atan`, `pi`, `e`, `^`, `(`, `)`.
- **रेडियन**: अचयनित: डिग्री.
- `-2^2 = -4`; `2^3^2 = 512`; `2*pi`; `1.5 = 1,5`.
- Enter: `=`; Escape: **साफ़ करें**; Backspace: **अक्षर मिटाएँ**; Tab / Shift+Tab.
- **परिणाम कॉपी करें**; **इतिहास**: 100; **इतिहास साफ़ करें**.
- IEEE 754 / f64; 15 digits; 4096 bytes; 64 levels.

## इतिहास

बंद करने पर भी अधिकतम 100 परिणाम सहेजे रहते हैं। इतिहास साफ़ करने से सहेजे रिकॉर्ड भी हटते हैं। सेटिंग्स में ↑/↓ से चलें, Enter या क्लिक से सक्रिय करें और Escape से लौटें।

`$XDG_DATA_HOME/argvus/calculator/history.json` (`~/.local/share/argvus/calculator/history.json`)

हमेशा बीच में छोटी तैरती विंडो (370 × 420) में खुलता है; वैज्ञानिक मोड इसे बड़ा कर सकता है।

मुख्य स्क्रीन पर केवल व्यंजक, परिणाम और कीपैड हैं। मेनू से वैज्ञानिक मोड, कोण इकाइयाँ, कॉपी और अलग इतिहास स्क्रीन खोलें। वापस या Escape से गणना पर लौटें; इतिहास का परिणाम चुनने पर उसका पुनः उपयोग होता है। बुनियादी मोड व्यंजक बनाए रखते हुए छोटी विंडो लौटाता है।

`SUPER + C` → `argvus calculator`

`.desktop`: `/usr/share/applications/argvus-calculator.desktop` · `Name=ARGVUS Calculator` · `Exec=argvus calculator`

ARGVUS: `focus_on_activate = true` (`org.argvus.*`, `sh.argvus.*`, `com.argvus.*`, `argvus-*`, `argvus.*`).

विंडो बढ़ाने पर बटन बड़े होते हैं। हर नई शुरुआत में डिफ़ॉल्ट आकार रहता है। अन्य स्क्रीन के ऊपर वापस जाने का तीर है।

`s Settings` → `settings` · Escape / ←

Min: **370 × 420** / **440 × 640** · `0–9`: **16 px**.

`2+2` → Enter / `=` → `4` (input) · `4*3` → Enter → `12`.

चुना गया बुनियादी या वैज्ञानिक मोड तुरंत सहेजा जाता है और अगली शुरुआत में लौटता है।

`$XDG_CONFIG_HOME/argvus/calculator/settings.json` (`~/.config/argvus/calculator/settings.json`).
