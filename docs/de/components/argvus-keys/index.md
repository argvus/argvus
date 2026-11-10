---
title: Schlüssel
description: SSH- und GPG-Schlüssel werden über argvus keys und argvus gpg verwaltet.
---
`argvus keys` verwaltet die SSH-Schlüsselpaare und `argvus gpg` verwaltet den GPG-Schlüsselspeicher. Beide sind Befehle im Prozess des `argvus`-Pakets, und die SSH- und GPG-Seiten des Kontrollzentrums verwenden sie.

## Befehle

```text
argvus keys                              show the help
argvus keys list                         list the key pairs in ~/.ssh and whether they are loaded
argvus keys generate NAME [--type ed25519|rsa] [--comment C] [--passphrase]
argvus keys load NAME                    add the key to the agent
argvus keys unload NAME                  remove the key from the agent
argvus keys unload-all                   remove every key from the agent
argvus keys delete NAME                  delete the key pair from ~/.ssh
```
Schlüssel werden niemals ausgedruckt. Eine Passphrase wird in das Terminal eingegeben, das `ssh-keygen` oder `ssh-add` ausführt; sie wird niemals als Argument übergeben. Legen Sie `ARGVUS_SSH_DIR` fest, um ein anderes Verzeichnis anstelle von `~/.ssh` zu verwenden.

## GPG

```text
argvus gpg list                          keys as TSV
argvus gpg status                        API version, gnupg and pinentry
argvus gpg import FILE                   public keys only
argvus gpg import-private FILE           a private key, kept protected
argvus gpg delete FINGERPRINT --yes
argvus gpg keyserver send|recv FINGERPRINT [--yes]
```
Destruktive und Veröffentlichungsbefehle benötigen `--yes`; das Kontrollzentrum gibt es nach eigener Bestätigung weiter. Das ARGVUS-Pinentry wird in `gpg-agent.conf` innerhalb eines Blocks geschrieben, den ARGVUS besitzt; Ihre eigenen Zeilen bleiben erhalten.
