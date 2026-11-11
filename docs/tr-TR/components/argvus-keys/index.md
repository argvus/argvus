---
title: Anahtarları
description: SSH ve GPG anahtarları, argvus anahtarları ve argvus gpg aracılığıyla yönetilir.
---

`argvus keys`, SSH anahtar çiftlerini yönetir ve `argvus gpg`, GPG anahtarlığını yönetir. Her ikisi de
`argvus` paketinin işlem içi komutları ve Kontrol Merkezi'nin SSH ve GPG sayfaları bunları kullanır.

## Komutlar

```text
argvus keys                              show the help
argvus keys list                         list the key pairs in ~/.ssh and whether they are loaded
argvus keys generate NAME [--type ed25519|rsa] [--comment C] [--passphrase]
argvus keys load NAME                    add the key to the agent
argvus keys unload NAME                  remove the key from the agent
argvus keys unload-all                   remove every key from the agent
argvus keys delete NAME                  delete the key pair from ~/.ssh
```

Anahtarlar asla yazdırılmaz. `ssh-keygen`'yu çalıştıran terminale bir parola yazılır veya
`ssh-add`; asla bir argüman olarak kabul edilmez. `ARGVUS_SSH_DIR`'yu başka bir dizini kullanacak şekilde ayarlayın
`~/.ssh` yerine.

##GPG

```text
argvus gpg list                          keys as TSV
argvus gpg status                        API version, gnupg and pinentry
argvus gpg import FILE                   public keys only
argvus gpg import-private FILE           a private key, kept protected
argvus gpg delete FINGERPRINT --yes
argvus gpg keyserver send|recv FINGERPRINT [--yes]
```

Yıkıcı ve yayınlama komutları `--yes`'ya ihtiyaç duyar; Kontrol Merkezi kendisinden sonra onu geçer
onay. ARGVUS pin girişi, ARGVUS'un belirlediği bir bloğun içindeki `gpg-agent.conf`'ya yazılır.
sahibi; kendi hatlarınız korunur.