---
title: Contribuição
description: Limites de contribuição para projetos e documentação.
slug: pt/0.4.0/docs/developer-guide/development/contributing
---

Trace uma funcionalidade por todos os repositórios consumidores. Preserve contratos de estado, ownership de pacotes e serviços e os dois catálogos de idioma. Valide origem, payload instalado e sessão ativa separadamente.

O `argvus-config` é o único escritor de `data/generated/`. Adicione ou altere um arquivo consumidor lá, em `argvus-config/src/project.rs`, e não em um helper shell. Se o aplicativo consumidor não conseguir ler o `config.json`, adicione um adapter no pacote dono e chame-o pelo orquestrador depois do commit canônico. Apague `data/generated/`, execute `argvus-config apply` e confirme que a árvore é reconstruída byte a byte a partir do `config.json`.
