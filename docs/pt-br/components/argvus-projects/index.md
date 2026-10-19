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
atalhos `SUPER + ALT + N`. Abrir um projeto foca o seu workspace do Hyprland. Na primeira vez que
um projeto é aberto, o ARGVUS inicia dentro dele um terminal e o editor definido em
`/projects/editor`.

A configuração é lida e gravada por `argvus config`, então o Control Center e o shell veem a mesma
lista.
