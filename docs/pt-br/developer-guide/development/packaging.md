---
title: Empacotamento
description: Construa e inspecione pacotes Arch do ARGVUS.
---

Cada componente possui seu packaging Arch. Inspecione `packaging/archlinux/` e o payload antes de documentar arquivos. Para verificar o runtime, compare com `pacman -Ql <pacote>` e inspecione overrides ativos separadamente.

Builds locais mantêm os objetos do Cargo em `build/cargo-target`, fora da árvore de fonte do pacote, então um `make build` repetido recompila só o que mudou. `make clean` remove esse diretório, e o próximo build começa do zero.

Não deduza o payload pelo nome do repositório. Um pacote pode ser responsável por binários, serviços de usuário, padrões em `/usr/share/argvus`, configuração administrativa em `/etc/argvus` ou assets. Confirme o payload real no `PKGBUILD` e com `pacman -Ql` depois da instalação.
