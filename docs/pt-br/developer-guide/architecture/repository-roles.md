---
title: Papéis dos repositórios
description: Responsabilidades dos projetos ARGVUS.
slug: pt/0.4.0/docs/developer-guide/architecture/repository-roles
---

Os projetos são agrupados como infraestrutura (`argvus`, `argvus-config`, `argvus-session`, `argvus-tui`, `argvus-i18n`), UI, provedores, integrações e assets/packaging. Consulte a [matriz de pacotes](/pt/docs/reference/package-matrix/) para os limites instalados.

Configuração: o `argvus-config` é dono do `config.json`, do modelo de paths e locks, e é o único escritor de `data/generated/`. A única regra que atravessa os pacotes: só o `argvus-config` escreve em `data/generated/`. Um componente que precise alcançar um aplicativo que não lê o `config.json` adiciona um adapter no seu próprio helper e deixa o orquestrador chamá-lo depois do commit canônico; ele nunca escreve um arquivo generated.

A linguagem não define ownership: cada componente mantém seus helpers junto do
próprio pacote. Rust é responsável pela lógica persistente e stateful de
runtime, Shell fornece apenas compatibilidade mínima ou adapters de processo e
Python é usado em testes e auditorias. CLIs públicos ficam em `/usr/bin`;
helpers executáveis internos ficam em `/usr/lib/argvus/<component>` quando
necessários em runtime; dados independentes de arquitetura permanecem em
`/usr/share/argvus/<component>`.
