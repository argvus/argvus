---
title: Ключи
description: SSH и GPG ключи управляются с помощью argvus keys и argvus gpg.
---
`argvus keys` управляет парами SSH-ключей, а `argvus gpg` управляет GPG-ключницей. Оба являются командами, выполняемыми в процессе, пакета `argvus`, и страницы SSH и GPG в Центре управления используют их.

## Команды

```text
argvus keys                              show the help
argvus keys list                         list the key pairs in ~/.ssh and whether they are loaded
argvus keys generate NAME [--type ed25519|rsa] [--comment C] [--passphrase]
argvus keys load NAME                    add the key to the agent
argvus keys unload NAME                  remove the key from the agent
argvus keys unload-all                   remove every key from the agent
argvus keys delete NAME                  delete the key pair from ~/.ssh
```
Ключи никогда не печатаются. Парольная фраза вводится в терминал, который запускает `ssh-keygen` или `ssh-add`; она никогда не передается в виде аргумента. Установите `ARGVUS_SSH_DIR`, чтобы использовать другой каталог вместо `~/.ssh`.

## GPG

```text
argvus gpg list                          keys as TSV
argvus gpg status                        API version, gnupg and pinentry
argvus gpg import FILE                   public keys only
argvus gpg import-private FILE           a private key, kept protected
argvus gpg delete FINGERPRINT --yes
argvus gpg keyserver send|recv FINGERPRINT [--yes]
```
Разрушительные и публикующие команды требуют `--yes`; Центр управления передает его после собственного подтверждения. Пинлентри ARGVUS записывается в `gpg-agent.conf` внутри блока, принадлежащего ARGVUS; ваши собственные строки сохраняются.
