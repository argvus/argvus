---
title: Monitor do sistema
description: Abra o monitor do sistema ARGVUS.
---

```sh
argvus --system-monitor
```

O subcomando `argvus system-monitor` fornece a integração com btop. A telemetria exibida na taskbar é fornecida separadamente por `argvus widget-telemetry`. Clicar nos módulos CPU e Memória abre o `btop` dentro de um painel TUI ARGVUS, com o módulo correspondente, cabeçalho da marca e rodapé `Esc Fechar`. A aba inferior do Kitty continua localizada e identifica o módulo. Pressione `Esc` para fechar o painel; `q` é encaminhado ao btop e encerra a visualização dele. As chamadas standalone `argvus system-monitor --cpu` e `--mem` continuam abrindo o btop diretamente.

O monitor lê primeiro o tema e o acento selecionados em
`$XDG_CONFIG_HOME/argvus/config.json`. Os arquivos btop gerados continuam
sendo cache derivado; `.active-theme` e `.accent-color` são fallbacks de
compatibilidade. Como o btop mantém seu próprio arquivo de tema, o monitor é reconciliado pela transação de tema (`argvus theme-switch`) como adapter externo depois que a mudança canônica é confirmada; ele não faz parte da árvore generated.

## Contrato de execução

- O tema e o accent vêm da configuração canônica (`argvus config get /appearance/theme` e
  `/appearance/accent`). Os arquivos `~/.config/argvus/data/.active-theme` e `.accent-color` são
  fallbacks.
- A configuração e os temas do btop gerados são gravados em
  `$XDG_CACHE_HOME/argvus/argvus-system-monitor`. São estado derivado; não os edite.
- Os padrões são instalados em `/usr/share/argvus-system-monitor/btop`.
- Os atalhos de módulo (`--mem`, `--cpu`, `--net`, `--proc`) são lidos apenas como primeiro
  argumento. Passar `-c`/`--config` ou `--themes-dir` do próprio btop executa o btop com esses
  caminhos e não grava configuração.
- `--panel [--cpu|--mem]` embute o btop na moldura TUI do ARGVUS. A taskbar usa esse modo; sem
  `--panel`, o monitor continua substituindo o processo diretamente pelo btop.
