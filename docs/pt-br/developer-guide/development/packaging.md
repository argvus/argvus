---
title: Empacotamento
description: Construa e inspecione pacotes Arch do ARGVUS.
slug: pt/0.4.0/docs/developer-guide/development/packaging
---

Cada componente possui seu packaging Arch. Inspecione `packaging/arch/` e o payload antes de documentar arquivos. Para verificar o runtime, compare com `pacman -Ql <pacote>` e inspecione overrides ativos separadamente.

O Makefile do workspace trata cada repositório do desktop como um pacote Arch independente. `make build` compila projetos instaláveis que possuem target `build`; `make build-selected` compila o subconjunto selecionado; `make collect` copia os arquivos de pacote para `builds/`; e `make install` instala os arquivos `argvus-*.pkg.tar.zst` coletados com pacman. Esses comandos pertencem ao fluxo de desenvolvimento e não são pré-requisitos para usuários que instalam pelo repositório de pacotes do ARGVUS.

Não deduza o payload pelo nome do repositório. Um pacote pode ser responsável por binários, serviços de usuário, padrões em `/usr/share/argvus`, configuração administrativa em `/etc/argvus` ou assets. Confirme o payload real no `PKGBUILD` e com `pacman -Ql` depois da instalação.
