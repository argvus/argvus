---
title: कुंजियाँ
description: SSH और GPG कुंजियाँ argvus कुंजियाँ और argvus gpg के माध्यम से प्रबंधित की जाती हैं।
---

`argvus keys` SSH कुंजी जोड़े का प्रबंधन करता है और `argvus gpg` GPG कीरिंग का प्रबंधन करता है। दोनों हैं
`argvus` पैकेज के इन-प्रोसेस कमांड और कंट्रोल सेंटर के SSH और GPG पेज उनका उपयोग करते हैं।

## आदेश

```text
argvus keys                              show the help
argvus keys list                         list the key pairs in ~/.ssh and whether they are loaded
argvus keys generate NAME [--type ed25519|rsa] [--comment C] [--passphrase]
argvus keys load NAME                    add the key to the agent
argvus keys unload NAME                  remove the key from the agent
argvus keys unload-all                   remove every key from the agent
argvus keys delete NAME                  delete the key pair from ~/.ssh
```

कुंजियाँ कभी मुद्रित नहीं होतीं. एक पासफ़्रेज़ उस टर्मिनल में टाइप किया जाता है जो `ssh-keygen` चलाता है या
`ssh-add`; इसे कभी भी तर्क के रूप में पारित नहीं किया जाता है। किसी अन्य निर्देशिका का उपयोग करने के लिए `ARGVUS_SSH_DIR` सेट करें
`~/.ssh` के बजाय।

## जीपीजी

```text
argvus gpg list                          keys as TSV
argvus gpg status                        API version, gnupg and pinentry
argvus gpg import FILE                   public keys only
argvus gpg import-private FILE           a private key, kept protected
argvus gpg delete FINGERPRINT --yes
argvus gpg keyserver send|recv FINGERPRINT [--yes]
```

विनाशकारी और प्रकाशन आदेशों के लिए `--yes` की आवश्यकता है; नियंत्रण केंद्र इसे अपने पास भेजता है
पुष्टि. ARGVUS पिनेंट्री को ARGVUS के एक ब्लॉक के अंदर `gpg-agent.conf` में लिखा गया है
मालिक है; आपकी अपनी पंक्तियाँ रखी गई हैं।