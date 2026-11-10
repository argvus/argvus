---
title: Llaves
description: Claves SSH y GPG gestionadas a través de las claves argvus y gpg argvus.
---
`argvus keys` gestiona los pares de claves SSH y `argvus gpg` gestiona el anillo de claves GPG. Ambos son comandos en proceso del paquete `argvus`, y las páginas de SSH y GPG del Centro de Control los utilizan.

## Comandos

```text
argvus keys                              show the help
argvus keys list                         list the key pairs in ~/.ssh and whether they are loaded
argvus keys generate NAME [--type ed25519|rsa] [--comment C] [--passphrase]
argvus keys load NAME                    add the key to the agent
argvus keys unload NAME                  remove the key from the agent
argvus keys unload-all                   remove every key from the agent
argvus keys delete NAME                  delete the key pair from ~/.ssh
```
Las claves nunca se imprimen. Se escribe una frase de contraseña en el terminal que ejecuta `ssh-keygen` o `ssh-add`; nunca se pasa como argumento. Configure `ARGVUS_SSH_DIR` para usar otro directorio en lugar de `~/.ssh`.

## GPG

```text
argvus gpg list                          keys as TSV
argvus gpg status                        API version, gnupg and pinentry
argvus gpg import FILE                   public keys only
argvus gpg import-private FILE           a private key, kept protected
argvus gpg delete FINGERPRINT --yes
argvus gpg keyserver send|recv FINGERPRINT [--yes]
```
Los comandos de destrucción y publicación necesitan `--yes`; el Centro de Control lo pasa después de su propia confirmación. El pinentry ARGVUS se escribe en `gpg-agent.conf` dentro de un bloque que ARGVUS posee; tus propias líneas se mantienen.
