---
title: खाते
description: स्थानीय उपयोगकर्ता प्रोफ़ाइल, प्रदर्शन नाम, अवतार और समूह सदस्यता, argvus खातों के माध्यम से।
---

`argvus accounts` ARGVUS डेस्कटॉप के स्थानीय उपयोगकर्ता खातों को सूचीबद्ध और संपादित करता है। यह एक निजी है
 `argvus` `/usr/lib/argvus/commands/accounts` पर स्थापित `argvus` पैकेज का निष्पादन योग्य। द
argvus कमांड इसे प्रारंभ करता है, और केवल वे ऑपरेशन जो अन्य खातों या सिस्टम डेटा को बदलते हैं
रूट के रूप में चलाएं.

## आदेश

```text
argvus accounts list [--all]                   list users (--all includes system accounts)
argvus accounts show USER                      show account details
argvus accounts name USER "DISPLAY NAME"       change another user's display name
argvus accounts avatar USER IMAGE              set another user's avatar
argvus accounts avatar USER --remove           remove another user's avatar
argvus accounts groups USER                    list groups
argvus accounts groups USER --add G --remove G change groups (authorization required)
argvus accounts passwd USER                    change a password (reads three lines from stdin)
argvus accounts self                           show your own account
argvus accounts self name "DISPLAY NAME"       change your own display name
argvus accounts self avatar IMAGE              set your avatar
argvus accounts self avatar --remove           remove your avatar
argvus accounts self groups                    list your groups
argvus accounts --verbose ...                  print diagnostic information to stderr
```

## नाम और अवतार प्रदर्शित करें

प्रदर्शन नाम मानक GECOS फ़ील्ड में संग्रहीत है। यूनिक्स उपयोक्तानाम कभी नहीं बदला जाता।

अवतार 20 MiB तक PNG, JPEG या WebP प्रारूप में एक छवि है। छवि का आकार 256×256 कर दिया गया है
पिक्सेल और ~/.face पर `0644` मोड के साथ PNG के रूप में सहेजा गया, फ्रीडेस्कटॉप स्थान द्वारा पढ़ा गया
स्वागतकर्ता और अधिकांश प्रदर्शन प्रबंधकों द्वारा। आपका अपना अवतार आपके होम डायरेक्टरी के अंदर लिखा हुआ है और
प्राधिकरण की आवश्यकता नहीं है.

## प्राधिकरण

केवल  `~/.face` `list`, `show`, `self`, `self groups` और आपका अपना अवतार प्राधिकरण के बिना काम करता है। हर
अन्य परिवर्तन डेस्कटॉप के पोलकिट एजेंट से प्राधिकरण के लिए पूछता है। आर्गवस नियम
(`/usr/share/polkit-1/rules.d/50-argvus-accounts.rules`) तय करता है कि किसे प्रमाणित करना होगा:

- व्यवस्थापक समूह में एक उपयोगकर्ता (`wheel`, आर्क डिफ़ॉल्ट, या `sudo`) अपने स्वयं के साथ पुष्टि करता है
  पासवर्ड;
- कोई अन्य उपयोगकर्ता रूट पासवर्ड से पुष्टि करता है।

| ऑपरेशन | बिना अनुमति के | प्राधिकरण के साथ |
| --- | --- | --- |
| उपयोगकर्ताओं की सूची बनाएं, खाता दिखाएं, समूहों की सूची बनाएं | ✓ | ✓ |
| अपना खुद का अवतार बदलें | ✓ | ✓ |
| अपना स्वयं का प्रदर्शन नाम बदलें | — | ✓ |
| अपना खुद का पासवर्ड बदलें | — | ✓ (वर्तमान पासवर्ड साबित करने के बाद) |
| किसी अन्य उपयोगकर्ता का नाम, अवतार, समूह या पासवर्ड बदलें | — | ✓ |

अपना स्वयं का पासवर्ड बदलने से पहले वर्तमान पासवर्ड की जाँच होती है। गलत पासवर्ड अस्वीकार कर दिया जाता है
प्राधिकरण संकेत प्रकट होने से पहले.

`sudo` के साथ कमांड चलाने की कोई आवश्यकता नहीं है। कमांड पोलकिट से प्राधिकरण के लिए पूछता है जब
इसकी जरूरत है.

## सुरक्षा सीमाएं

- `passwd` मानक इनपुट से वर्तमान, नए और पुष्टिकरण पासवर्ड पढ़ता है, प्रति पंक्ति एक,
  और उन्हें कभी भी तर्क के रूप में नहीं लेता। व्यवस्थापक के समय वर्तमान पासवर्ड लाइन खाली हो सकती है
  दूसरे उपयोगकर्ता का पासवर्ड रीसेट करता है। उदाहरण:

  ```sh
  printf '%s\n%s\n%s\n' "$OLD" "$NEW" "$NEW" | argvus accounts passwd ghost
  ```

  अंतःक्रियात्मक रूप से, तीन पंक्तियाँ टाइप करें और प्रत्येक के बाद Enter दबाएँ। इनपुट पर दिखाया गया है
  टर्मिनल जैसा टाइप किया गया है।
- UID 1000 (और `nobody`) से नीचे के खाते को संपादित करने से इनकार कर दिया गया है, और इसलिए इसे लॉक कर रहे हैं, हटा रहे हैं
  और इसका पासवर्ड समाप्त हो रहा है। आपका अपना खाता लॉक या हटाया नहीं जा सकता.
- GID 1000 से नीचे के समूह या व्यवस्थापक समूह (`wheel`, `sudo`) का नाम बदलना या हटाना है
  मना कर दिया; इसके सदस्य अभी भी बदले जा सकते हैं, सिवाय इसके कि आप स्वयं को इससे नहीं हटा सकते
  व्यवस्थापक समूह.
- प्रशासक सदस्यता `wheel` या `sudo` समूह है। नियंत्रण केंद्र नया जोड़ता है
  जब सिस्टम में यह मौजूद हो तो व्यवस्थापक `wheel` पर, और अन्यथा `sudo` पर। अपना खुद का हटाना
  संपादन के माध्यम से व्यवस्थापक सदस्यता अस्वीकार कर दी गई है।

## उदाहरण

```sh
# Set your display name and avatar after the first login
argvus accounts self name "William Canin"
argvus accounts self avatar ~/Pictures/profile.png

# Check your groups, then add another user to the wheel group (authorization required)
argvus accounts self groups
argvus accounts groups ghost --add wheel

# Administrator: show a user's details and reset that user's name
argvus accounts show ghost
argvus accounts name ghost "Ghost Account"
```

## समस्या निवारण

- `pkexec was not found`: `polkit` इंस्टॉल करें। `argvus` पैकेज इस पर निर्भर करता है।
- प्राधिकरण संकेत प्रकट नहीं होता है: जांचें कि पोलकिट प्रमाणीकरण एजेंट चल रहा है
  आपके सत्र में. `argvus` पैकेज `hyprpolkitagent` पर निर्भर करता है, जो एक प्रदान करता है।
- पासवर्ड बदलते समय `unix_chkpwd` त्रुटियाँ: `pam` पैकेज सहायक प्रदान करता है।
- stderr पर कमांड के चरणों को प्रिंट करने के लिए `argvus accounts --verbose ...` चलाएँ।

विशेषाधिकार सीमा के बारे में डेवलपर नोट्स के लिए, `docs/internal/argvus-accounts/` देखें।जब कोई व्यवस्थापक स्वचालित लॉगिन के लिए कॉन्फ़िगर किए गए खाते को लॉक या हटा देता है, तो खाता प्रदाता पहले उसकी स्वचालित-लॉगिन प्रविष्टि को अक्षम कर देता है। यदि वह सफ़ाई विफल हो जाती है, तो खाता संचालन अस्वीकार कर दिया जाता है। यदि बाद में लॉक करना या हटाना विफल हो जाता है, तो स्वचालित लॉगिन अक्षम रहता है। किसी अन्य खाते के लिए स्वचालित लॉगिन संरक्षित है; किसी खाते को अनलॉक करने से स्वचालित लॉगिन सक्षम नहीं होता है। मौजूदा सत्र बंद नहीं हुए हैं और greetd पुनः आरंभ नहीं हुआ है।
