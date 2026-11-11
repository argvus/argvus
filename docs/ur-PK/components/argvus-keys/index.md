---
title: کیز
description: SSH اور GPG کلیدوں کو argvus کیز اور argvus gpg کے ذریعے منظم کیا گیا۔
---

`argvus keys` SSH کلیدی جوڑوں کا انتظام کرتا ہے اور `argvus gpg` GPG کیرنگ کا انتظام کرتا ہے۔ دونوں ہیں۔
`argvus` پیکج کی ان پروسیس کمانڈز، اور کنٹرول سینٹر کے SSH اور GPG پیجز انہیں استعمال کرتے ہیں۔

## احکامات

```text
argvus keys                              show the help
argvus keys list                         list the key pairs in ~/.ssh and whether they are loaded
argvus keys generate NAME [--type ed25519|rsa] [--comment C] [--passphrase]
argvus keys load NAME                    add the key to the agent
argvus keys unload NAME                  remove the key from the agent
argvus keys unload-all                   remove every key from the agent
argvus keys delete NAME                  delete the key pair from ~/.ssh
```

چابیاں کبھی پرنٹ نہیں ہوتیں۔ ایک پاسفریز ٹرمینل میں ٹائپ کیا جاتا ہے جو `ssh-keygen` چلاتا ہے یا
`ssh-add`; یہ کبھی بھی دلیل کے طور پر نہیں گزرتا۔ دوسری ڈائریکٹری استعمال کرنے کے لیے `ARGVUS_SSH_DIR` سیٹ کریں۔
`~/.ssh` کے بجائے۔

## جی پی جی

```text
argvus gpg list                          keys as TSV
argvus gpg status                        API version, gnupg and pinentry
argvus gpg import FILE                   public keys only
argvus gpg import-private FILE           a private key, kept protected
argvus gpg delete FINGERPRINT --yes
argvus gpg keyserver send|recv FINGERPRINT [--yes]
```

تباہ کن اور پبلشنگ کمانڈز کو `--yes` کی ضرورت ہے۔ کنٹرول سینٹر اسے اپنے بعد منتقل کرتا ہے۔
تصدیق ARGVUS پنینٹری `gpg-agent.conf` میں ایک بلاک کے اندر لکھی گئی ہے جو ARGVUS
مالک ہے آپ کی اپنی لائنیں رکھی جاتی ہیں۔