---
title: Keys
description: SSH dan kunci GPG dikelola melalui kunci argvus dan argvus gpg.
---

`argvus keys` mengelola pasangan kunci SSH dan `argvus gpg` mengelola keyring GPG. Keduanya
perintah dalam proses dari paket `argvus`, dan halaman SSH dan GPG Pusat Kontrol menggunakannya.

## Perintah

```text
argvus keys                              show the help
argvus keys list                         list the key pairs in ~/.ssh and whether they are loaded
argvus keys generate NAME [--type ed25519|rsa] [--comment C] [--passphrase]
argvus keys load NAME                    add the key to the agent
argvus keys unload NAME                  remove the key from the agent
argvus keys unload-all                   remove every key from the agent
argvus keys delete NAME                  delete the key pair from ~/.ssh
```

Kunci tidak pernah dicetak. Frasa sandi diketikkan ke terminal yang menjalankan `ssh-keygen` atau
`ssh-add`; itu tidak pernah dianggap sebagai argumen. Setel `ARGVUS_SSH_DIR` untuk menggunakan direktori lain
bukannya `~/.ssh`.

## GPG

```text
argvus gpg list                          keys as TSV
argvus gpg status                        API version, gnupg and pinentry
argvus gpg import FILE                   public keys only
argvus gpg import-private FILE           a private key, kept protected
argvus gpg delete FINGERPRINT --yes
argvus gpg keyserver send|recv FINGERPRINT [--yes]
```

Perintah destruktif dan penerbitan memerlukan `--yes`; Pusat Kontrol meneruskannya setelah miliknya
konfirmasi. Pinentry ARGVUS ditulis ke dalam `gpg-agent.conf` di dalam blok yang ARGVUS
memiliki; dialog Anda sendiri disimpan.