---
title: Núcleo e dispatcher
description: O workspace argvus, seu dispatcher e os crates Rust compartilhados.
---

`argvus` é o workspace Cargo em `de/argvus` e o pacote que fornece `/usr/bin/argvus`. O crate `argvus-cli` é o dispatcher: ele lê o primeiro argumento e executa um comando de biblioteca em processo, um auxiliar GTK 4, um executável privado ou um script empacotado. A lista completa de subcomandos está na [referência de linha de comando](/pt/docs/reference/command-line/).

O workspace também reúne os crates que os comandos compartilham:

- `argvus-config`: o armazenamento canônico da configuração e o único escritor da árvore gerada (`data/generated/`). É também uma biblioteca usada em processo por `argvus config`, `theme-switch` e `effects-toggle`.
- `argvus-i18n` e `argvus-i18n-core`: os catálogos em `locales/`, a validação deles e o módulo QML `org.argvus.i18n`.
- `argvus-tui` e `argvus-theme`: a interface de terminal compartilhada e o carregamento de temas.
- `argvus-runtime`: as mensagens traduzidas e os auxiliares de processo compartilhados pelos comandos.

`argvus-session` continua sendo um repositório separado. Ele fornece os pontos de entrada da sessão (`session`, `start`, `tty`) e as units de ciclo de vida, e `argvus sessionctl` chega ao controlador dele.
