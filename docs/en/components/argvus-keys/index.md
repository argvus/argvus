---
title: Keys
description: SSH and GPG keys managed through argvus keys and argvus gpg.
---

`argvus keys` manages the SSH key pairs and `argvus gpg` manages the GPG keyring. Both are
in-process commands of the `argvus` package, and the Control Center's SSH and GPG pages use them.

## Commands

```text
argvus keys                              show the help
argvus keys list                         list the key pairs in ~/.ssh and whether they are loaded
argvus keys generate NAME [--type ed25519|rsa] [--comment C] [--passphrase]
argvus keys load NAME                    add the key to the agent
argvus keys unload NAME                  remove the key from the agent
argvus keys unload-all                   remove every key from the agent
argvus keys delete NAME                  delete the key pair from ~/.ssh
```

Keys are never printed. A passphrase is typed into the terminal that runs `ssh-keygen` or
`ssh-add`; it is never passed as an argument. Set `ARGVUS_SSH_DIR` to use another directory
instead of `~/.ssh`.

## GPG

```text
argvus gpg list                          keys as TSV
argvus gpg status                        API version, gnupg and pinentry
argvus gpg import FILE                   public keys only
argvus gpg import-private FILE           a private key, kept protected
argvus gpg delete FINGERPRINT --yes
argvus gpg keyserver send|recv FINGERPRINT [--yes]
```

Destructive and publishing commands need `--yes`; the Control Center passes it after its own
confirmation. The ARGVUS pinentry is written into `gpg-agent.conf` inside a block that ARGVUS
owns; your own lines are kept.
