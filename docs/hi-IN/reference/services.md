---
title: सेवाएँ
description: ARGVUS सिस्टम और उपयोगकर्ता सेवाएँ।
---

. Common user services include 

, 

, 

, 

, 

, 

, `argvus-session.target`, clipboard services and `argvus-taskbar.service`.

फ़ायरवॉल  argvus-session-loading.service  argvus-hypridle.service  argvus-wallpaper.service  argvus-dunst.service  argvus-hyprland.service  argvus-widget-telemetry.service  `argvus-control-panel.service`  `argvus-widget-telemetry.service`  `argvus-hyprland.service`  `argvus-dunst.service`  `argvus-wallpaper.service`  `argvus-hypridle.service`  `argvus-session-loading.service` `argvus-firewall.service` नामक एक सिस्टम सेवा है। कैलेंडर पैकेज `argvus-taskbar-calendar.service` को एक उपयोगकर्ता इकाई के रूप में स्थापित करता है, और `argvus-clock.service` (डिफ़ॉल्ट रूप से अक्षम) लॉगिन के बाद छिपा हुआ ARGVUS क्लॉक शुरू करता है जब **सत्र के साथ प्रारंभ करें** चालू होता है।

सक्रिय इकाइयों का निरीक्षण करें:

```sh
systemctl --user status argvus-session.target
systemctl --user list-units 'argvus-*.service'
```

## सत्र-स्वामित्व वाली इकाइयाँसत्र पैकेज इन उपयोगकर्ता इकाइयों को `/usr/lib/systemd/user` के अंतर्गत स्थापित करता है (स्रोत पेलोड सत्र पैकेज के साझा कॉन्फ़िगरेशन ट्री के अंतर्गत है):

| इकाई | भूमिका |
| --- | --- |
| `argvus-session.target` | ARGVUS ग्राफ़िकल सत्र के लिए आजीवन सीमा। |
| `argvus-session-prepare.service` | एक-शॉट स्टार्टअप तैयारी और उत्पन्न कॉन्फ़िगरेशन। |
| `argvus-control-panel.service` | क्विकशेल कंट्रोल पैनल। |
| `argvus-taskbar.service` | मुख्य ARGVUS टास्कबार। |
| `argvus-widget-telemetry.service` | वैकल्पिक टेलीमेट्री/sysinfo सतह। |
| `argvus-hyprland.service` | ऑन-डिमांड वन-शॉट इकाई जो `hyprctl reload` पर चलती है। |
| `argvus-config.service` | कैनोनिकल कॉन्फ़िगरेशन प्रोजेक्ट करता है (`argvus config project`); पुनः लोड `argvus sessionctl reload-config-transaction` चलता है। || `argvus-session-reload.service` | लॉगिन पुनः लोड करें (`argvus sessionctl reload-desktop --login`): हाइप्रलैंड को केवल तभी पुनः लोड करता है जब प्रक्षेपण ने इसे प्रभावित किया हो। |
| `argvus-ssh-agent.service` | सत्र के लिए SSH एजेंट. |
| `argvus-secret-service.service` | `argvus sessionctl start-secret-service` के माध्यम से गुप्त सेवा प्रारंभ करता है। || `argvus-wallpaper.service` | सक्रिय वॉलपेपर बैकएंड. |
| `argvus-dunst.service` | डंस्ट अधिसूचना डेमॉन. |
| `argvus-hypridle.service` | निष्क्रिय और लॉक नीति. |
| `argvus-clipboard-text.service` / `argvus-clipboard-image.service` | क्लिपबोर्ड इतिहास पर नजर रखने वाले। |
| `argvus-keyboard-layout.service` | कीबोर्ड-लेआउट सूचनाएं. |
| `argvus-polkit.service` | पॉलिसीकिट एजेंट एकीकरण। |
| `argvus-session-loading.service` | कंपोजिटर हैंडऑफ़ से डेस्कटॉप घटकों के प्रारंभ होने तक सत्र-लोडिंग ओवरले। |

`argvus-blueman-applet.service` को सत्र पैकेज द्वारा भी भेजा जाता है। कुछ इकाइयाँ वैकल्पिक हैं या सशर्त रूप से शुरू की गई हैं; वे हर सत्र में सक्रिय नहीं होते हैं.

किसी इकाई की केवल स्रोत जांच के बजाय उसकी प्रभावी परिभाषा का निरीक्षण करना:

```sh
systemctl --user cat argvus-session.target
systemctl --user status argvus-session.target
journalctl --user -u argvus-session.target -u 'argvus-*.service'
```

सिस्टम-स्तरीय `argvus-firewall.service` `argvus-firewall` से संबंधित है। कैलेंडर पैकेज अलग से `argvus-taskbar-calendar.service` को एक उपयोगकर्ता इकाई के रूप में शिप करता है। `argvus-clock.service`, `argvus` पैकेज, `WantedBy=graphical-session.target` की एक उपयोगकर्ता इकाई है, जो ARGVUS क्लॉक सेटिंग्स से सक्षम है।

## सत्र प्रबंधन आदेश

`argvus sessionctl` कमांड सत्र जीवनचक्र का प्रबंधन करता है और लॉग आउट किए बिना विशिष्ट घटकों को पुनरारंभ कर सकता है:

### सत्र जीवनचक्र

```sh
argvus sessionctl start               # Start a new session
argvus sessionctl stop                # Stop the current session
argvus sessionctl restart             # Restart the entire session
argvus sessionctl reload              # Reload configuration and active services
argvus sessionctl status              # Show session status
argvus sessionctl logs                # Display session logs
```

### घटक-विशिष्ट पुनरारंभ

संपूर्ण सत्र को प्रभावित किए बिना व्यक्तिगत सेवाएँ पुनः प्रारंभ करें:

```sh
# Desktop surfaces
argvus sessionctl restart waybar
argvus sessionctl restart wallpaper
argvus sessionctl restart shell

# Notifications and switcher
argvus sessionctl restart dunst

# Clipboard and input
argvus sessionctl restart clipboard keyboard-layout

# Bluetooth applet (optional)
argvus sessionctl restart blueman-applet
```

### पर्यावरण तुल्यकालन

सत्र में पर्यावरण चर आयात करें:

```sh
argvus sessionctl import-environment
```

यह शेल और सिस्टमड उपयोगकर्ता सेवाओं के बीच वेलैंड, हाइप्रलैंड, क्यूटी, कर्सर और डीबीस वेरिएबल्स को सिंक्रनाइज़ करता है।

## सेवा इकाइयों का संदर्भ

अतिरिक्त उपयोगकर्ता इकाइयों को सीधे `systemctl --user` के साथ प्रबंधित किया जा सकता है:

```sh
systemctl --user status argvus-session.target
systemctl --user start argvus-session.target
systemctl --user stop argvus-session.target
systemctl --user restart <unit>

# View logs for a specific service
journalctl --user -u argvus-wallpaper.service -b -f
```

प्रारंभिक सत्र बूटस्ट्रैप के लिए लॉग यहां संग्रहीत हैं:

```
${XDG_STATE_HOME:-$HOME/.local/state}/argvus/session.log
```
