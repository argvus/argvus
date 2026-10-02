---
title: Guia do desenvolvedor
description: Arquitetura e desenvolvimento dos componentes ARGVUS.
slug: pt/0.4.0/docs/developer-guide
---

ARGVUS é um conjunto de projetos empacotados separadamente, conectados por configuração compartilhada, serviços systemd de usuário e contratos Wayland/Hyprland.

## Comece pelo ARGVUS Workflow

Antes de trabalhar na infraestrutura do ARGVUS, leia o [`README.md` do repositório ARGVUS Workflow](https://github.com/argvus/workflow). O mesmo repositório possui um mirror no [GitLab](https://gitlab.com/argvus/workflow).

O repositório de workflow é o coordenador central, baseado em Make, do workspace ARGVUS. Ele cria e organiza checkouts independentes em `de/`, `web/` e `misc/`, compila pacotes Arch, coleta os artefatos em `builds/`, instala pacotes selecionados em um host Arch e fornece helpers para o workspace. Ele é a infraestrutura para desenvolver e montar o ARGVUS; não substitui o Makefile nem o sistema de build pertencente a cada projeto.

Os comandos usuais do workflow são:

```sh
make clone full
make build full
make collect
make install full
make status
make claude
```

Use `make clone de <project> [project...]`, `make build <project> [project...]` ou `make install <project> [project...]` quando estiver trabalhando com um conjunto selecionado de projetos do desktop. O target `claude` prepara o checkout do workflow para o Claude criando os links documentados para `AGENTS.md` e para o diretório de skills. A instalação chama o `pacman` e exige um host Arch Linux com as permissões necessárias.

Os projetos clonados continuam sendo repositórios Git independentes. O workflow os coordena, mas não possui o histórico, as versões de pacotes nem as dependências específicas de cada projeto. Suas variáveis configuráveis e regras de preparação do workspace estão documentadas no README; consulte-o antes de alterar a infraestrutura local ou adicionar um novo projeto.

* [Arquitetura](/pt/docs/developer-guide/architecture/overview/)
* [Subsistemas](/pt/docs/developer-guide/subsystems/session-and-systemd/)
* [Desenvolvimento](/pt/docs/developer-guide/development/environment/)
* [Referência](/pt/docs/reference/)

Os repositórios em `de/` são a fonte de verdade. Manifests, payloads e units definem os limites de runtime.
