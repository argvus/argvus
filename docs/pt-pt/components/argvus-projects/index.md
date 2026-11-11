---
title: Projetos
description: O seletor de projetos e a configuração de projetos, pelo argvus projects.
---

`argvus projects` lista e abre os seus projetos. É um comando em processo do pacote `argvus`, e
`SUPER + O` abre o seletor.

## Comandos

```text
argvus projects                    escolhe um projeto (SUPER + O)
argvus projects open SELETOR       abre um projeto pela posição ou pelo nome
argvus projects add [--root] DIR   adiciona um projeto, ou uma raiz que contém projetos
argvus projects remove DIR         remove um projeto ou uma raiz
argvus projects list               mostra as raízes e os caminhos configurados
argvus projects pick [PROMPT]      mostra o caminho escolhido na lista, para outras ferramentas
```

## O que é um projeto

Um projeto é um diretório listado na configuração (`/projects/paths`), ou um subdiretório de
uma raiz listada em `/projects/roots`. No máximo nove projetos aparecem na lista: os números dos
atalhos `SUPER + CTRL + N`. Abrir um projeto foca o seu workspace do Hyprland. Na primeira vez que
um projeto é aberto, o ARGVUS inicia dentro dele um terminal e o editor definido em
`/projects/editor`.

A configuração é lida e gravada por `argvus config`, então o Control Center e o shell veem a mesma
lista.

## Workspaces de projetos e taskbar

Projetos usam workspaces nomeados comuns do Hyprland, não workspaces especiais. `SUPER + O` mostra o número, diretório e atalho direto `SUPER + CTRL + N`. As duas entradas focam o mesmo workspace. Apenas workspaces abertos aparecem na taskbar, com rótulo `Projeto: nome`, destaque do ativo e tooltip com caminho completo e atalho. Clique no botão do workspace para retornar a ele. A taskbar acompanha o monitor do workspace.

O caminho canônico do diretório determina uma identidade estável: alterar a ordem da lista altera o atalho, mas não o workspace. Projetos com nomes de diretório iguais ficam separados; links simbólicos para o mesmo diretório são deduplicados. Terminal e editor recebem explicitamente o workspace de destino. Editores que reutilizam um processo/janela existente podem manter o comportamento de workspace do próprio aplicação.

Workspaces abertos antes da atualização mantêm suas janelas e nomes nativos. Abrir um projeto após a atualização usa sua nova identidade derivada do caminho; feche as janelas antigas quando for conveniente. Não há associação nem movimentação automática porque o nome antigo não identifica o diretório com segurança. Com nomes de diretório duplicados, use o seletor, um número ou o caminho canônico completo em vez de um nome ambíguo.

Abrir um projeto regenera atomicamente `$XDG_RUNTIME_DIR/argvus/project-workspaces.json`, usando a lista configurada e as traduções. Esse ficheiro é metadado da sessão para rótulos e tooltips; não é configuração e não deve ser editado. Reabra um projeto após alterar a lista ou o idioma para atualizar os rótulos. Atualize `argvus`, `argvus-waybar` e a configuração do `argvus-taskbar` para usar essa integração.
