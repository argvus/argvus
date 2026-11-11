---
title: कमांड लाइन
description: argvus कमांड, इसके उपकमांड, कीबाइंडिंग के लिए रखे गए झंडे, और जहां प्रत्येक चलता है।
---

`/usr/bin/argvus` ARGVUS का सार्वजनिक प्रवेश बिंदु है। यह `argvus-cli` क्रेट से निर्मित एक रस्ट प्रोग्राम है। प्रत्येक उपकमांड या तो उस प्रोग्राम के अंदर चलता है, कमांड निर्देशिका से एक सहायक निष्पादन योग्य चलाता है, या एक पैकेज्ड स्क्रिप्ट चलाता है।

अपने संस्करण के साथ स्थापित सूची के लिए `argvus --help` चलाएँ। उपकमांड अग्रणी `--` के बिना लिखे गए हैं। `--` फॉर्म केवल वहीं रखे जाते हैं जहां कीबाइंडिंग या डेस्कटॉप प्रविष्टि उनका उपयोग करती है।

## जहां प्रत्येक कमांड चलती है- **In-process:** the command is a library of the workspace and runs inside `argvus`. No separate executable exists. These are `config`, `theme`, `i18n`, `keys`, `gpg`, `projects`, `terminal`, `system-monitor`, `network`, `bluetooth`, `focus status`, `widget-telemetry`,`tui-terminal`, `sensors` और `setup`।- **सहायक:** `/usr/lib/argvus/commands/` में एक अलग निष्पादन योग्य, `argvus` पैकेज द्वारा स्थापित। ये हैं `control-center` (टर्मिनल एप्लिकेशन), `argvus-du` (`argvus du` के साथ लॉन्च) और GTK 4 एप्लिकेशन `taskbar-calendar`, `removable-devices`, `focus` और `clock`।- **निजी निष्पादन योग्य फ़ाइल:** एक निष्पादन योग्य फ़ाइल जिसे कोई और पैकेज `/usr/lib/argvus/commands/` में इंस्टॉल करता है। `argvus` केवल इसे शुरू करता है। इसमें `session`, `start`, `tty`, `sessionctl`, `launcher`, `snippets`, `ssh`, `git`, `app-profiles-yazi-keymap`, `notifications`, `displayctl`, `firewall`, `accounts`, `taskbar-actions` और `greeter-*` एंट्री पॉइंट शामिल हैं।
- **स्क्रिप्ट:** एक पैकेज्ड शेल स्क्रिप्ट जो `/bin/sh` के साथ चलती है। ये नीचे सूचीबद्ध हाइप्रलैंड स्क्रिप्ट कमांड हैं।

`ARGVUS_COMMANDS_DIR` परीक्षण और `DESTDIR` जांच के लिए कमांड डायरेक्टरी को बदलता है। एक निजी निष्पादन योग्य जो वहां नहीं मिला है उसे `PATH` पर देखा गया है।

## उपकमांड

| आदेश | चलता है | नोट्स |
| --- | --- | --- |
| `argvus config ...` | प्रक्रिया में | कॉन्फ़िगरेशन मॉडल और उपकरण. `theme-switch` और `effects-toggle` द्वारा भी पहुंचा गया। |
| `argvus theme ...` | प्रक्रिया में | `apply ID` थीम स्विच है (`theme-switch` के समान)। थीम बिल्ड: `list` स्थापित थीम, `check DIRECTORY` इंस्टॉल किए गए टेम्पलेट्स के विरुद्ध एक थीम, `render ID [--variant sticky\|spaced] [--accent #RRGGBB] [--output DIRECTORY]`। [उपस्थिति और थीम](/docs/developer-guide/subsystems/appearance-and-themes/) देखें। |
| `argvus i18n ...` | प्रक्रिया में | अनुवाद: `get DOMAIN KEY [name=value...]`, `locale`, `list`, `validate [LOCALE]`। |
| `argvus keys ...` | प्रक्रिया में | एसएसएच कुंजी प्रबंधक। `gpg` GPG उपकमांड के लिए एक उपनाम है। |
| `argvus gpg ...` | प्रक्रिया में | GPG कुंजियाँ: `list`, `status`, `generate`, `export-public`, `export-private`, `import`, `import-private`, `delete`, `edit`, `keyserver`। |
| `argvus projects ...` | प्रक्रिया में | प्रोजेक्ट पिकर. |
| `argvus terminal ...` | प्रक्रिया में | ARGVUS टर्मिनल प्रोफ़ाइल। |
| `argvus system-monitor ...` | प्रक्रिया में | बीटॉप सिस्टम मॉनिटर; `--gpu` केवल GPU बॉक्स का चयन करता है, और `--panel [--cpu|--mem|--gpu]` इसे ARGVUS TUI शेल में एम्बेड करता है। |
| `argvus du` | हेल्पर (टर्मिनल एप्लिकेशन) | एक समर्पित किटी विंडो में ARGVUS डिस्क उपयोग विश्लेषक खोलता है और एनसीडीयू चलाता है। |
| `argvus network ...` | प्रक्रिया में | नेटवर्कमैनेजर स्थिति और नियंत्रण (`networkctl` को उपनाम के रूप में रखा गया है)। |
| `argvus bluetooth ...` | प्रक्रिया में | ब्लूटूथ स्थिति और नियंत्रण (`bluetoothctl` को उपनाम के रूप में रखा गया है)। || `argvus widget-telemetry block ...` | प्रक्रिया में | वेबार टेलीमेट्री ब्लॉक। यह बार का गर्म रास्ता है. || `argvus widget-telemetry toggle ...` | प्रक्रिया में | टेलीमेट्री ब्लॉकों को चालू और बंद और उनके क्रम को स्विच करता है। |
| `argvus game-snake` | निजी निष्पादन योग्य | स्वतंत्र `argvus-game-snake` प्रोजेक्ट से टर्मिनल स्नेक गेम, एक वैकल्पिक पैकेज। एक टर्मिनल के बाहर यह एक को खोलता है। |
| `argvus control-center [page]` | हेल्पर (टर्मिनल एप्लिकेशन) | नियंत्रण केंद्र खोलता है, या पहले से खुली हुई विंडो पर फ़ोकस करता है। |
| `argvus welcome [--if-needed]` | डिस्पैचर → नियंत्रण केंद्र | नियंत्रण केंद्र में एकीकृत स्वागत पृष्ठ खोलता है। `--if-needed` एक बार खुलता है जब तक कि प्रति-उपयोगकर्ता प्रगति रिकॉर्ड नहीं करता कि वेलकम पहले ही प्रदर्शित, पूर्ण या खारिज कर दिया गया था। |
| `argvus taskbar-calendar [args]` | जीटीके 4 एप्लीकेशन | डिफ़ॉल्ट रूप से `toggle` है। |
| `argvus removable-devices [args]` | जीटीके 4 एप्लीकेशन | डिफ़ॉल्ट रूप से `menu`। |
| `argvus focus [--toggle] [--start\|--pause] [--x X --y Y] [--bar-bottom PX --bar-right PX]` | जीटीके 4 एप्लीकेशन | ARGVUS फोकस पॉपअप। प्रति सत्र एक उदाहरण. |
| `argvus focus status --waybar [--follow]` | प्रक्रिया में | फ़ोकस टास्कबार संकेतक को वेबार JSON के रूप में उत्सर्जित करता है; `--follow` स्ट्रीम GTK लोड किए बिना बदल जाती है। |
| `argvus clock [--toggle] [--background] [--page PAGE] [--x X --y Y] [--bar-bottom PX --bar-right PX]` | जीटीके 4 एप्लीकेशन | ARGVUS क्लॉक पॉपअप। प्रति सत्र एक उदाहरण; `--background` बिना पॉपअप के अलार्म को सक्रिय रखता है। |
| `argvus loading-theme ...` | /usr/lib/argvus/loading-theme/splash पर पैक किया गया `splash` | प्रत्यक्ष सत्र पुनः लोड का ओवरले; थीम स्विच इसका उपयोग नहीं करता है. |
|  `/usr/lib/argvus/loading-theme/splash` `argvus session`, `start`, `tty`, `sessionctl` | निजी निष्पादनयोग्य | सत्र प्रवेश बिंदु और सत्र नियंत्रक। |
| `argvus greeter`, `greeter-session`, `greeter-tui`, `greeter-setup` | निजी निष्पादनयोग्य | स्वागतकर्ता प्रवेश बिंदु. |
| `argvus accounts`, `firewall` | निजी निष्पादनयोग्य | खाते और फ़ायरवॉल उपकरण. |
| `argvus launcher`, `snippets`, `ssh`, `git` | निजी निष्पादनयोग्य | लांचर और पिकर उपकरण. |
| `argvus tui-terminal` | प्रक्रिया में | टीयूआई अनुप्रयोगों के लिए टर्मिनल रैपर। |
| `argvus sensors` | प्रक्रिया में | `sensors -j` से लाइव तापमान अवलोकन। |
| `app-profiles-yazi-keymap` | निजी निष्पादन योग्य | कीमैप सहायक. |
| `argvus notifications`, `displayctl`, `taskbar-actions` | निजी निष्पादनयोग्य | सूचनाएं, प्रदर्शन और टास्कबार क्रियाएँ। |
| `argvus setup [options]` | प्रक्रिया में | पैकेज्ड डिफॉल्ट्स को उपयोगकर्ता कॉन्फ़िगरेशन में कॉपी करता है। नीचे देखें। |

रिक्त स्थान और रिक्त मान सहित तर्क अपरिवर्तित अग्रेषित किए जाते हैं। मानक इनपुट, आउटपुट, त्रुटि, सिग्नल और निकास स्थिति चलने वाले कमांड से संबंधित हैं। एक अज्ञात नाम `argvus --help` चलाने के संकेत के साथ विफल रहता है। एक कमांड जो इन-प्रोसेस चलता है, एक परिवर्तित मान को पारित करने के बजाय एक तर्क को अस्वीकार कर देता है जो वैध यूटीएफ -8 (निकास स्थिति 2) नहीं है; एक अलग प्रोग्राम चलाने वाले कमांड को बाइट्स अपरिवर्तित प्राप्त होते हैं।

## झंडे और उपनाम

ये नाम मौजूद हैं ताकि कीबाइंडिंग, डेस्कटॉप प्रविष्टियाँ और पुरानी स्क्रिप्ट काम करती रहें। नए कोड में उपकमांड फॉर्म को प्राथमिकता दें।

| झंडा या उपनाम | समतुल्य |
| --- | --- |
| `--control-center`, `--about`, `--default-apps` | `control-center` (`about` और `apps` पेजों के साथ) |
| `--displays`, `--monitors` पेज | `control-center displays` |
| `--network-interfaces` | `control-center network interfaces` |
| `--system-monitor`, `--terminal`, `--game-snake` | `system-monitor`, `terminal`, `game-snake` |
| `--calendar`, `--removable-devices`, `--storage` | `taskbar-calendar`, `removable-devices` |
| `--focus`, `--clock` | `focus`, `clock` |
| `--spf`, `--superfile` | फ़ाइल प्रबंधक (`spf`) |
| `--yazi`, `--yazy` | फ़ाइल ब्राउज़र (`yazi`) |
| `--setup` | `setup` |
| `--start-desktop` | `tty` प्रारंभ होता है (कोई तर्क अग्रेषित नहीं किया जाता) |
| `--version`, `-V`, `version` | `ARGVUS <version>` प्रिंट करता है |
| `help`, `--help`, `-h` | उपयोग प्रिंट करता है || `networkctl`, `bluetoothctl` | `network`, `bluetooth` |
| `widget-telemetry-toggle`, `widget-telemetry-block` | `widget-telemetry toggle`, `widget-telemetry block` |
| `theme-switch` | `config theme-apply` |
| `effects-toggle` | `config effects` |

नियंत्रण केंद्र लीगेसी पेज फ़्लैग को अपने पहले तर्क के रूप में स्वीकार करता है और उन्हें पेज नामों पर मैप करता है। उदाहरण: `--apps` `apps` बन जाता है, `--keyboard-layout` `locale` बन जाता है, और `--hostname` `system` बन जाता है।

## हाइप्रलैंड स्क्रिप्ट कमांड

ये पैकेज्ड स्क्रिप्ट चलाते हैं और हाइप्रलैंड कॉन्फ़िगरेशन द्वारा उपयोग किए जाते हैं:

`cheatsheets-hypr`, `cheatsheets-launcher`, `scratchpad`, `hypr-screenshot`, `spaces-switch`, `emoji-picker`, `toggle-sidebar`, `weather-location`, `layout-mode-menu`, `brightness-switch`, `toggle-mode`, `idle-timeout`, `keep-awake`, `hypr-power-menu`।

उपयोगकर्ता के कॉन्फ़िगरेशन में एक स्क्रिप्ट पैकेज्ड स्क्रिप्ट को ओवरराइड करती है। लुकअप ऑर्डर hyprland.lua में `_config_path` के समान है: उपयोगकर्ता की कॉन्फ़िगरेशन कॉपी, फिर ARGVUS डेटा कॉपी, फिर स्टेट कॉपी, फिर जेनरेट की गई कॉपी, और उसके बाद ही  `hyprland.lua` `/usr/share/argvus` के तहत पैक की गई फ़ाइल।

## उदाहरण

```sh
argvus control-center displays
argvus taskbar-calendar toggle
argvus widget-telemetry block header system
argvus widget-telemetry toggle status
argvus i18n get widget-telemetry label.device
argvus config get /appearance/theme
argvus network status
argvus bluetooth status
argvus gpg list
```

## सेटअप

`argvus setup` पैकेज्ड डिफॉल्ट्स को `/usr/share/argvus/<app>` से `~/.config/argvus/data/<app>` में कॉपी करता है, ताकि आप उन्हें कस्टमाइज़ कर सकें। यह कभी भी पैक की गई फ़ाइलों को संपादित नहीं करता है।

```sh
argvus setup --copy <app>          # copy one packaged app
argvus setup --copy-all            # copy every packaged app directory
argvus setup --copy <app> --force  # back up the existing destination first
argvus setup --copy <app> --dry-run
```

जो गंतव्य पहले से मौजूद है उसकी सूचना दी जाती है और उसे अकेला छोड़ दिया जाता है, जब तक कि `--force` नहीं दिया जाता है। `--force` के साथ, कॉपी से पहले मौजूदा निर्देशिका का नाम बदलकर `<name>.bak-<timestamp>` कर दिया जाता है।

`argvus` ऐप विशेष है: यह अपनी पैकेज्ड निर्देशिका की सामग्री को कॉन्फ़िगरेशन रूट में कॉपी करता है। क्योंकि सेटअप चलने पर कॉन्फ़िगरेशन रूट पहले से मौजूद होता है, `--copy argvus` को वर्तमान कोड में `--force` की आवश्यकता होती है।

सेटअप कुछ भी बदलने से पहले हर विकल्प को मान्य करता है। अमान्य ऐप नाम (खाली, `.` से शुरू होने वाले, या `/` वाले) अस्वीकार कर दिए जाते हैं।

## सत्र और पुनः लोड करें

`SUPER + SHIFT + R` `argvus sessionctl reload` चलाता है। सत्र नियंत्रक जेनरेट किए गए कॉन्फ़िगरेशन को लागू करता है और केवल उसके स्वामित्व वाले घटकों को पुनरारंभ करता है। यह `/usr/share/argvus` को होम डायरेक्टरी में कॉपी नहीं करता है, और यह विकृत JSON फ़ाइल को प्रतिस्थापित नहीं करता है।

## रात की रोशनी

**डेस्कटॉप और इनपुट > डिस्प्ले > नाइट लाइट** खोलें या `argvus control-center displays night-light` चलाएं। `argvus config display night-light` के माध्यम से तापमान, मैन्युअल सक्रियण और निश्चित स्थानीय शेड्यूल कॉन्फ़िगर करें। कमांड, स्थिति और रनटाइम आवश्यकताओं के लिए [नाइट लाइट](/docs/user-guide/night-light/) देखें।

`argvus tui-terminal --frame TITLE FOOTER COMMAND [ARG...]` साझा ARGVUS टर्मिनल फ्रेम के अंदर एक इंटरैक्टिव कमांड चलाता है। Ctrl+E फ़्रेम बंद कर देता है; इसका कॉलर स्थानीयकृत फ़ुटर की आपूर्ति करता है। इसे टर्मिनल के अंदर या `argvus tui-terminal --term kitty -- ...` को दिए गए आदेश के रूप में उपयोग करें।

`argvus tui-terminal --term kitty --profile ai-agents` छिपे हुए टैब और केवल क्लिपबोर्ड शॉर्टकट के साथ एक समर्पित किट्टी कॉन्फ़िगरेशन का उपयोग करता है। कॉल करने वाला आदेश और शीर्षक प्रदान करता है; कक्षा `argvus-ai` के लिए विंडो प्लेसमेंट को हाइप्रलैंड नियम द्वारा नियंत्रित किया जाता है।