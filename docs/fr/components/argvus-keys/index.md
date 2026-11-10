---
title: Clés
description: Les clés SSH et GPG sont gérées via les clés argvus et gpg argvus.
---
`argvus keys` gère les paires de clés SSH et `argvus gpg` gère le trousseau de clés GPG. Les deux sont des commandes en processus du paquet `argvus`, et les pages SSH et GPG du Centre de Contrôle les utilisent.

## Commandes

```text
argvus keys                              show the help
argvus keys list                         list the key pairs in ~/.ssh and whether they are loaded
argvus keys generate NAME [--type ed25519|rsa] [--comment C] [--passphrase]
argvus keys load NAME                    add the key to the agent
argvus keys unload NAME                  remove the key from the agent
argvus keys unload-all                   remove every key from the agent
argvus keys delete NAME                  delete the key pair from ~/.ssh
```
Les clés ne sont jamais imprimées. Une phrase de passe est saisie dans le terminal qui exécute `ssh-keygen` ou `ssh-add` ; elle n'est jamais transmise en tant qu'argument. Définissez `ARGVUS_SSH_DIR` pour utiliser un autre répertoire au lieu de `~/.ssh`.

## GPG

```text
argvus gpg list                          keys as TSV
argvus gpg status                        API version, gnupg and pinentry
argvus gpg import FILE                   public keys only
argvus gpg import-private FILE           a private key, kept protected
argvus gpg delete FINGERPRINT --yes
argvus gpg keyserver send|recv FINGERPRINT [--yes]
```
Les commandes de destruction et de publication nécessitent `--yes` ; le Centre de Contrôle le transmet après sa propre confirmation. Le pinentry ARGVUS est écrit dans `gpg-agent.conf` à l'intérieur d'un bloc que ARGVUS possède ; vos propres lignes sont conservées.
