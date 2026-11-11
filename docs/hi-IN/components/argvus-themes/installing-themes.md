---
title: थीम इंस्टॉल करना
description: पैक्मैन के साथ आधिकारिक ARGVUS थीम पैकेज स्थापित करें।
---

आधिकारिक थीम आर्क लिनक्स पैकेज के रूप में प्रकाशित की जाती हैं। प्रत्येक थीम का अपना पैकेज है जिसका नाम `argvus-theme-<THEME_NAME>` है।

## थीम कहां से प्राप्त करें

प्रत्येक आधिकारिक विषय का स्रोत ARGVUS परियोजना संगठन में रहता है:

- प्रत्येक थीम `argvus-theme-<THEME_NAME>` (उदाहरण के लिए `argvus-theme-dracula`) नामक एक रिपॉजिटरी है, जिसे समान नाम के पैकेज के रूप में बनाया गया है।

थीम पैकेज `argvus-appearance` पर निर्भर करते हैं। पहले ARGVUS स्थापित करें, जैसा कि [इंस्टॉलेशन](/docs/getting-started/installation/) में बताया गया है, ताकि उपस्थिति घटक मौजूद रहे।

## एक ही थीम स्थापित करें

प्रत्येक आधिकारिक थीम को एक साथ इंस्टॉल करने के लिए, `argvus-themes` मेटापैकेज इंस्टॉल करें:

```bash
sudo pacman -S argvus-themes
```

यह सभी प्रकाशित `argvus-theme-*` पैकेजों पर निर्भर करता है। ARGVUS डार्क और ARGVUS
प्रकाश पहले ही `argvus-appearance` द्वारा भेज दिया गया है।

इसके पैकेज नाम के साथ एक थीम स्थापित करें:

```bash
sudo pacman -S argvus-theme-<THEME_NAME>
```

उदाहरण के लिए:

```bash
sudo pacman -S argvus-theme-dracula
sudo pacman -S argvus-theme-rose-pine
```

पैकेज थीम मेनिफ़ेस्ट, `/usr/share/argvus/themes/<theme-id>/theme.toml` (इसका नाम, श्रेणी, वॉलपेपर और पैलेट, वैकल्पिक `overrides/` के साथ) और थीम का वॉलपेपर `/usr/share/backgrounds/argvus/` के अंतर्गत स्थापित करता है। थीम लागू होने पर प्रत्येक घटक की फ़ाइलें पैलेट से प्रस्तुत की जाती हैं। यह आपकी होम निर्देशिका में नहीं लिखता है, और यह आपकी वर्तमान थीम को नहीं बदलता है। बाद में नियंत्रण केंद्र से इसे चुनें, [आधिकारिक थीम](/docs/components/argvus-themes/official-themes/) देखें।

## कई थीम इंस्टॉल करें

ऐसा कोई मेटा-पैकेज नहीं है जो हर थीम को इंस्टॉल करता हो। अपने इच्छित थीम को एक कमांड में इंस्टॉल करें:

```bash
sudo pacman -S argvus-theme-dracula argvus-theme-sunset
```
## उपलब्ध पैकेज

नीचे दिए गए पैकेज आज प्रकाशित आधिकारिक थीम पैकेज हैं।

**डार्क थीम**

| पैकेज | थीम |
| --- | --- |
| `argvus-theme-one-dark` | एक अंधेरा |
| `argvus-theme-dracula` | ड्रैकुला |
| `argvus-theme-silver-dark` | सिल्वर डार्क |
| `argvus-theme-slate-dark` | स्लेट डार्क |
| `argvus-theme-universe` | ब्रह्माण्ड |
| `argvus-theme-gruvbox-high-dark` | ग्रुवबॉक्स हाई डार्क |
| `argvus-theme-gruvbox-dark` | ग्रुवबॉक्स डार्क |
| `argvus-theme-rose-pine` | रोज़ पाइन |
| `argvus-theme-tokyo-night` | टोक्यो रात |
| `argvus-theme-solitude` | एकांत |
| `argvus-theme-sunset` | सूर्यास्त |
| `argvus-theme-matrix` | मैट्रिक्स |
| `argvus-theme-monokai-dark` | मोनोकाई डार्क |

**हल्की थीम**

| पैकेज | थीम |
| --- | --- |
| `argvus-theme-github-light` | गिटहब लाइट |
| `argvus-theme-solarized-light` | सोलराइज्ड लाइट |
| `argvus-theme-nord-light` | नॉर्ड लाइट |
| `argvus-theme-one-light` | एक रोशनी |
| `argvus-theme-everforest-light` | एवरफॉरेस्ट लाइट |
| `argvus-theme-catppuccin-latte` | कैटपुचिन लट्टे |
| `argvus-theme-gruvbox-light` | ग्रुवबॉक्स लाइट |
| `argvus-theme-watercolor` | जल रंग |

अंतर्निहित **ARGVUS डार्क** और **ARGVUS लाइट** थीम को अलग पैकेज की आवश्यकता नहीं है; वे `argvus-appearance` के साथ आते हैं।

## जांचें कि क्या स्थापित है

स्थापित थीम पैकेजों को इसके साथ सूचीबद्ध करें:

```bash
pacman -Qs argvus-theme
```

नियंत्रण केंद्र खोलें और वर्तमान में चयनकर्ता में उपलब्ध थीम देखने के लिए **प्रकटन → थीम्स → आधिकारिक थीम्स** पर जाएं। एक थीम जो स्थापित है लेकिन उस सूची से गायब है वह एक पैकेजिंग समस्या है; [थीम गाइड](/docs/components/argvus-themes/) का समस्या निवारण अनुभाग देखें।

## अपनी खुद की एक थीम जोड़ें

एक विषय एक प्रकट रूप है, इसलिए आपके अपने विषय को किसी पैकेज की आवश्यकता नहीं है। बनाएँ
`~/.local/share/argvus/themes/<id>/theme.toml`, जहां निर्देशिका का नाम है
थीम आईडी (छोटे अक्षर, अंक और एकल हाइफ़न):

```toml
schema = 2
id = "mist"
name = "Mist"
category = "dark"
wallpaper = "/usr/share/backgrounds/argvus/argvus-dark.jxl"

[palette]
bg = "#101418"
fg = "#D8DEE9"
accent = "#88C0D0"
```

केवल `bg`, `fg` और `accent` आवश्यक हैं: हर दूसरे रंग में एक डिफ़ॉल्ट होता है
उनसे व्युत्पन्न, और प्रकट इसे बदलने के लिए केवल एक को परिभाषित करता है। इसकी जाँच करें,
फिर इसे किसी अन्य विषय की तरह लागू करें:

```bash
argvus theme check ~/.local/share/argvus/themes/mist
argvus theme apply mist
```

`argvus theme list` आपके विषयों को आधिकारिक विषयों के बगल में दिखाता है (एक अधिकारी
समान आईडी वाली थीम जीतती है)। आपकी थीम थीम मेनू में दिखाई देती हैं
(`SUPER + Shift + T`); प्री-लॉगिन ग्रीटर आपकी होम निर्देशिका को नहीं पढ़ सकता है, इसलिए
यह आपके द्वारा लागू की गई अंतिम आधिकारिक थीम को तब तक रखता है जब तक आपकी कोई थीम सक्रिय है।

प्रति-थीम प्रतियां जो पुराने ARGVUS संस्करणों के अंतर्गत लिखी गई थीं`~/.config/argvus/data/<app>/themes/` अब नहीं पढ़ा जाता है। पहला थीम स्विच
अद्यतन के बाद उन्हें `~/.config/argvus/data/backups/theme-copies-<time>/` पर ले जाया जाता है;
आपके द्वारा संपादित फ़ाइल को रखने के लिए, इसे `overrides/` के अंतर्गत अपनी स्वयं की थीम में रखें और
इसे मेनिफेस्ट की `[overrides]` तालिका में नाम दें।

## एक थीम हटाएं

इसके साथ एकल थीम पैकेज निकालें:

```bash
sudo pacman -R argvus-theme-<THEME_NAME>
```

थीम पैकेज को हटाने से आपकी आयातित प्रोफ़ाइलें नहीं हटतीं। यदि हटाई गई थीम वर्तमान में लागू थीम है, तो उसे हटाने से पहले नियंत्रण केंद्र से किसी अन्य थीम का चयन करें।