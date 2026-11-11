---
title: Keys
description: تتم إدارة مفاتيح SSH وGPG من خلال مفاتيح argvus وargvus gpg.
---

يدير `argvus keys` أزواج مفاتيح SSH ويدير `argvus gpg` حلقة مفاتيح GPG. كلاهما
تستخدمها الأوامر قيد التشغيل لحزمة `argvus` وصفحات SSH وGPG الخاصة بمركز التحكم.

##الأوامر

```text
argvus keys                              show the help
argvus keys list                         list the key pairs in ~/.ssh and whether they are loaded
argvus keys generate NAME [--type ed25519|rsa] [--comment C] [--passphrase]
argvus keys load NAME                    add the key to the agent
argvus keys unload NAME                  remove the key from the agent
argvus keys unload-all                   remove every key from the agent
argvus keys delete NAME                  delete the key pair from ~/.ssh
```

لا تتم طباعة المفاتيح أبدًا. تتم كتابة عبارة المرور في المحطة التي تقوم بتشغيل `ssh-keygen` أو
`ssh-add`; لا يتم تمريره كحجة أبدًا. قم بتعيين `ARGVUS_SSH_DIR` لاستخدام دليل آخر
بدلاً من `~/.ssh`.

## جي بي جي

```text
argvus gpg list                          keys as TSV
argvus gpg status                        API version, gnupg and pinentry
argvus gpg import FILE                   public keys only
argvus gpg import-private FILE           a private key, kept protected
argvus gpg delete FINGERPRINT --yes
argvus gpg keyserver send|recv FINGERPRINT [--yes]
```

تحتاج أوامر التدمير والنشر إلى `--yes`؛ يقوم مركز التحكم بتمريره بعد ذلك
تأكيد. تتم كتابة Pinentry ARGVUS في `gpg-agent.conf` داخل كتلة ARGVUS
يملك؛ يتم الاحتفاظ بالخطوط الخاصة بك.