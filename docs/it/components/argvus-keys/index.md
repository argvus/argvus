---
title: Chiavi
description: Chiavi SSH e GPG gestite tramite chiavi argvus e gpg argvus.
---
`argvus keys` gestisce le coppie di chiavi SSH e `argvus gpg` gestisce il portachiavi GPG. Entrambi sono comandi in-process del pacchetto `argvus`, e le pagine SSH e GPG del Centro di Controllo li utilizzano.

## Comandi

```text
argvus keys                              show the help
argvus keys list                         list the key pairs in ~/.ssh and whether they are loaded
argvus keys generate NAME [--type ed25519|rsa] [--comment C] [--passphrase]
argvus keys load NAME                    add the key to the agent
argvus keys unload NAME                  remove the key from the agent
argvus keys unload-all                   remove every key from the agent
argvus keys delete NAME                  delete the key pair from ~/.ssh
```
Le chiavi non vengono mai stampate. Una passphrase viene digitata nel terminale che esegue `ssh-keygen` o `ssh-add`; non viene mai passata come argomento. Imposta `ARGVUS_SSH_DIR` per utilizzare un'altra directory invece di `~/.ssh`.

## GPG

```text
argvus gpg list                          keys as TSV
argvus gpg status                        API version, gnupg and pinentry
argvus gpg import FILE                   public keys only
argvus gpg import-private FILE           a private key, kept protected
argvus gpg delete FINGERPRINT --yes
argvus gpg keyserver send|recv FINGERPRINT [--yes]
```
I comandi distruttivi e di pubblicazione richiedono `--yes`; il Centro di Controllo lo passa dopo la propria conferma. Il pinentry ARGVUS è scritto in `gpg-agent.conf` all'interno di un blocco di proprietà di ARGVUS; le tue linee vengono conservate.
