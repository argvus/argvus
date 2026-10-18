---
title: Monitor do sistema
description: Abra o monitor do sistema ARGVUS.
slug: pt/0.4.0/docs/user-guide/applications/system-monitor
---

```sh
argvus --system-monitor
```

O subcomando `argvus system-monitor` fornece a integração com btop. A telemetria exibida na taskbar é fornecida separadamente por `argvus widget-telemetry`.

O monitor lê primeiro o tema e o acento selecionados em
`$XDG_CONFIG_HOME/argvus/config.json`. Os arquivos btop gerados continuam
sendo cache derivado; `.active-theme` e `.accent-color` são fallbacks de
compatibilidade. Como o btop mantém seu próprio arquivo de tema, o monitor é reconciliado pela transação de tema (`argvus theme-switch`) como adapter externo depois que a mudança canônica é confirmada; ele não faz parte da árvore generated.
