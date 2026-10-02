---
title: Configuração e estado
description: Estado compartilhado, overrides e arquivos gerados.
slug: pt/0.4.0/docs/developer-guide/architecture/configuration-and-state
---

A raiz normal é `$XDG_CONFIG_HOME/argvus` (geralmente `~/.config/argvus`). As preferências lógicas gerenciadas ficam em `config.json`. Tema, accent, wallpaper, espaçamento e bordas são canônicos nesse arquivo; os dot-files permanecem como projeções derivadas de compatibilidade para os consumidores atuais do Lua do Hyprland e dos scripts, enquanto esses consumidores migram para `data/`.

`argvus-config` é o único escritor de `generated/`. Todo arquivo consumidor
pertencente ao ARGVUS é uma projeção de `config.json`, incluindo Hyprland, GTK,
Qt6ct, Waybar, QuickShell, Rofi, Dunst, Hyprlock, os perfis de taskbar e
widget-telemetry do Waybar, as árvores de configuração do Yazi e do Superfile, as
árvores de perfis de terminal e de Qt, os blocos gerenciados de fonte e a folha de
estilo de dispositivos removíveis. Overrides explícitos do usuário têm
precedência, depois vêm as projeções geradas e então os defaults do pacote. A
projeção normal de tema/reload nunca promove arquivos gerados ou empacotados para
override do usuário.

Por isso `argvus-appearance` orquestra tema e accent, enquanto o
`argvus-config` mantém o estado canônico e as projeções estruturadas.
`theme-switch.sh` e `accent-switch.sh` validam, travam, confirmam a mudança pelo
`argvus-config` quando disponível e então reconciliam os consumidores de
compatibilidade que não conseguem ler o `config.json` diretamente. Esses adapters são o arquivo nativo do Qt6ct, o
`settings.ini` do GTK e as dicas de `gsettings`, os aplicativos de terminal e de
monitoramento do sistema, o snappy-switcher, o foot, o diretório de configuração
próprio do superfile e o greeter de pré-autenticação. Um helper que precise de um
novo adapter deve adicioná-lo ali, nunca dentro de `generated/`.

A ownership de runtime é separada por responsabilidade: Rust mantém o store
canônico, validação do schema, migrações, modelo de paths, locks, writes
atômicos, hashes de projeção e renderizadores estruturados. Shell fica apenas
com entrypoints de compatibilidade pequenos ou adapters para consumidores
externos como `hyprctl` e scripts de assets de temas. Python fica reservado a
testes, auditorias e ferramentas de desenvolvimento; não é dependência do
runtime crítico do desktop.

`argvus-config apply` é a fronteira de reconstrução (com `project` como alias
compatível). Ele regenera outputs ausentes mesmo quando o documento canônico
não mudou, escreve projectors em staging e troca atomicamente a árvore `data/`
apenas depois de projeções bem-sucedidas. O lifecycle da sessão continua pertencendo
ao `argvus-sessionctl`, que usa esse manifesto para planejar reloads focados.

Quando o chamador já confirmou uma generation — `apply-theme`, uma troca de
accent, um `set`/`patch` ou uma mudança de bloco de widget-telemetry — o
`reload` **aplica** essa generation em vez de projetá-la de novo. Ele lê o plano
do manifesto, restaura os consumidores alvo e só projeta quando o manifesto está
ausente ou não tem trabalho pendente. Reprojetar nesse momento reconstruiria uma
árvore idêntica e descartaria o plano que a mutação acabou de produzir.

O manifesto também registra a generation operacional, o hash da configuração
efetiva, os domains alterados e a lista de arquivos gerados.
`data/internal/generation.json` é apenas um marcador local descartável.
`argvus-config status` compara esses dados com a configuração resolvida e
detecta arquivos ausentes ou extras.

Mutations seguem a mesma fronteira: `UserConfig` é clonado para um
`NextUserConfig`, validado, resolvido em `EffectiveConfig`, projetado e só então
persistido. `.config-transaction.json` é estado operacional de recuperação;
`argvus-config recover` reprojeta deterministicamente o documento canônico
anterior após uma queda entre a publicação de `data/` e `config.json`. O reload
é solicitado somente depois do commit, e uma falha de reload não desfaz uma
configuração persistente coerente.

Mutations persistentes calculam `ChangedDomains` a partir da configuração efetiva
anterior e nova. Depois do commit, um único reload é enviado pelo
`argvus-sessionctl`; falhas são registradas sem rollback da configuração. O
startup faz recovery antes do bootstrap e da projeção, evitando que consumidores
iniciem contra uma transação abandonada.

## Modelo de resolução

```text
modelo lógico config.json
    ↓
estado nativo de compatibilidade e override por aplicativo do ARGVUS
    ↓
arquivo de runtime gerado pelo ARGVUS
    ↓
default empacotado em /usr/share/argvus
    ↓
default upstream
```

Os candidatos exatos variam por componente, mas o princípio é consistente: o override do usuário vence, a saída gerada é estado derivado e os arquivos empacotados permanecem como entradas somente leitura. A sessão exporta `ARGVUS_CONFIG_HOME` e `ARGVUS_SYSTEM_CONFIG` para que helpers e serviços do usuário usem as mesmas raízes.

A recuperação do reload segue `recover → ensure → migrate → apply → reload do
runtime`, e a etapa de projeção é pulada quando o manifesto recebido já descreve
uma generation commitada. `ensure` materializa `config.json` a partir dos defaults do schema somente
quando o perfil está ausente e completa campos ausentes sem substituir valores
explícitos como `false`, `0` ou customizações. Um documento existente e
malformado continua sendo um erro, em vez de ser substituído silenciosamente.

| Estado | Finalidade |
| --- | --- |
| `config.json` | Preferências lógicas gerenciadas canônicas. |
| `.active-theme`, `.accent-color`, `.accent-custom` | Entradas legadas de migração; não são fontes de runtime nem são recriadas. |
| `.wallpaper-custom` | Entrada legada de migração para `appearance.wallpaper`; não é fonte de runtime. |
| `.spaces`, `.borders` | Entradas legadas de layout; os valores canônicos ficam em `layout` no `config.json`. |
| `fonts.conf` | Projeção de compatibilidade das fontes canônicas. |
| `generated/effects` | Projeção descartável dos efeitos canônicos. |
| `$XDG_STATE_HOME/argvus/config-projection.json` | Manifesto de projeção: seções alteradas, consumidores afetados e se um reload é necessário. Lido pelo `reload`; nunca é fonte de verdade. |
| `$XDG_STATE_HOME/argvus/widget-telemetry-blocks` | Preferências legadas de blocos de telemetria. Lidas apenas como fallback de migração; os valores canônicos ficam em `/control_panel/widget_telemetry_blocks` e o arquivo não é mais escrito. |

Uma propriedade é canônica quando é lida de `config.json` e projetada a partir
dele. Uma propriedade é derivada quando é projetada, mas ainda pode ser consumida
como entrada de migração. Tudo o que um componente ainda escreve em runtime — um
cache, um arquivo nativo de configuração em `$XDG_CONFIG_HOME/<app>`, um arquivo
de estado do greeter — é alvo de adapter e não deve ser lido de volta como
configuração.

A projeção cobre os consumidores de Hyprland, GTK/Qt, Waybar, Quickshell, terminais, notificações, lock screen e aplicativos nativos. O Hyprland lê as cores das bordas ativa e inativa do tema selecionado; somente `appearance.accent_custom=true` substitui a borda ativa/do grupo pelo acento definido pelo usuário. A troca de tema reaplica esses valores no reload do compositor para que ele não retenha as cores do tema anterior. Quando a configuração do usuário é reconstruída, as animações ficam habilitadas por padrão. Apagar `generated/` e executar `argvus-config apply` o recria byte a byte a partir do `config.json`; editar arquivos gerados diretamente não é durável.
O diagnóstico de runtime deve identificar o conjunto completo de execução:
caminho e versão/revisão do binário, raiz de assets, config, data, arquivos
gerados, raiz interna e manifesto de projeção. `status --verbose` exibe esse
conjunto e compara generation/hash publicados com o config resolvido atual.
Testes do checkout e do pacote instalado são modos separados e não podem cair
silenciosamente em outro executável ou no home do desenvolvedor. `apply` é
também a fronteira de reconciliação para identificadores legados combinados:
`argvus-dark-float` torna-se `theme=argvus-dark` e `variant=float` antes da
resolução. `/usr/share/argvus/version.json` descreve a identidade dos assets
empacotados como metadado de diagnóstico, não como fonte de verdade.
