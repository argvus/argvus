---
title: Chaves
description: Chaves SSH e GPG gerenciadas por argvus keys e argvus gpg.
---

`argvus keys` gerencia os pares de chaves SSH e `argvus gpg` gerencia o chaveiro GPG. Os dois são
comandos em processo do pacote `argvus`, e as páginas de SSH e GPG do Control Center os usam.

## Comandos

```text
argvus keys                              mostra a ajuda
argvus keys list                         lista os pares de chaves em ~/.ssh e se estão carregados
argvus keys generate NOME [--type ed25519|rsa] [--comment C] [--passphrase]
argvus keys load NOME                    adiciona a chave ao agente
argvus keys unload NOME                  remove a chave do agente
argvus keys unload-all                   remove todas as chaves do agente
argvus keys delete NOME                  apaga o par de chaves de ~/.ssh
```

As chaves nunca são impressas. A senha é digitada no terminal que executa `ssh-keygen` ou
`ssh-add`; nunca é passada como argumento. Defina `ARGVUS_SSH_DIR` para usar outro diretório
no lugar de `~/.ssh`.

## GPG

```text
argvus gpg list                          chaves em TSV
argvus gpg status                        versão da API, gnupg e pinentry
argvus gpg import ARQUIVO                somente chaves públicas
argvus gpg import-private ARQUIVO        uma chave privada, guardada protegida
argvus gpg delete IMPRESSAO_DIGITAL --yes
argvus gpg keyserver send|recv IMPRESSAO_DIGITAL [--yes]
```

Comandos destrutivos e de publicação exigem `--yes`; o Control Center o passa depois da própria
confirmação. O pinentry do ARGVUS é gravado em `gpg-agent.conf` dentro de um bloco que o ARGVUS
controla; as suas próprias linhas são mantidas.
