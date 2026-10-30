---
title: Monitor do sistema
description: Abra o monitor do sistema ARGVUS.
---

```sh
argvus --system-monitor
```

O subcomando `argvus system-monitor` fornece a integração com btop. A telemetria exibida na taskbar é fornecida separadamente por `argvus widget-telemetry`. Clicar nos módulos CPU, Memória e GPU abre o `btop` dentro de um painel TUI ARGVUS, com cabeçalho da marca e rodapé `Esc Fechar`. O modo GPU mostra até seis painéis de GPU, sem caixas de CPU, memória, rede ou processos. Pressione `q` ou `Esc` para fechar. A aba inferior do Kitty mostra a dica localizada `Press q to close`. As chamadas standalone `argvus system-monitor --cpu`, `--mem` e `--gpu` abrem o btop diretamente com as caixas selecionadas.

O painel de GPU depende dos recursos da compilação instalada do btop e dos providers do sistema. O monitoramento NVIDIA exige suporte a GPU no btop, a NVIDIA Management Library do driver instalado e acesso ao dispositivo. AMD usa ROCm SMI, que é opcional e pode não estar instalado. Os dados da GPU Intel vêm de sysfs e dependem do acesso do usuário a essas interfaces; o btop 1.4.7 mostra uso, energia e frequência Intel, mas não oferece temperatura Intel. Nouveau e outros drivers podem expor a temperatura da GPU à taskbar por hwmon vinculado ao dispositivo, enquanto o suporte de painel do btop depende do driver e das bibliotecas disponíveis. O ARGVUS não altera permissões nem executa o btop com privilégios elevados. Sem GPU detectável ou se o btop não renderizar uma caixa de GPU em três segundos por falta de suporte/provider, o painel mostra uma mensagem localizada de indisponibilidade.

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
- `--panel [--cpu|--mem|--gpu]` embute o btop na moldura TUI do ARGVUS. A taskbar usa esse modo; sem
  `--panel`, o monitor continua substituindo o processo diretamente pelo btop.
