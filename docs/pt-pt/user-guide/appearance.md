---
title: Aparência
description: Configure a linguagem visual compartilhada do ARGVUS.
---

O sistema de aparência aplica um estado comum ao Hyprland, GTK, Qt, Waybar, Quickshell, Rofi, Dunst, Hyprlock e aplicações ARGVUS.

## Famílias de tema

As vinte e três famílias abaixo são definidas pelo payload atual de temas. A coluna de cor mostra o valor restaurado pela ação **Restaurar padrão do tema**; o utilizador pode escolher outra cor RGB válida independentemente.

| Família | Cor de destaque restaurada | Fundo base | Modos |
| --- | --- | --- | --- |
| Um Escuro | `#61AFEF` | `#282C34` | pegajoso, espaçado |
| Drácula | `#BD93F9` | `#282A36` | pegajoso, espaçado |
| ARGVUS Escuro | `#3590bd` | `#111316` | pegajoso, espaçado |
| Prata Escuro | `#595959` | `#111316` | pegajoso, espaçado |
| Ardósia Escura | `#7391a5` | `#2f3541` | pegajoso, espaçado |
| Universo Sombrio | `#eeeeee` | `#000000` | pegajoso, espaçado |
| ARGVUS Dark Gruvbox Alto | `#D79921` | `#282828` | pegajoso, espaçado |
| ARGVUS Dark Gruvbox | `#D4BE98` | `#282828` | pegajoso, espaçado |
| Luz ARGVUS | `#181818` | `#f7f7f7` | pegajoso, espaçado |
| GitHub Light | `#0969DA` | `#FFFFFF` | pegajoso, espaçado |
| Catppuccin Latte | `#1E66F5` | `#EFF1F5` | pegajoso, espaçado |
| ARGVUS Light Gruvbox | `#458588` | `#FBF1C7` | pegajoso, espaçado |
| Pinho Rosé | `#C4A7E7` | `#191724` | pegajoso, espaçado |
| Noite de Tóquio | `#7AA2F7` | `#1A1B26` | pegajoso, espaçado |
| Solidão | `#798186` | `#101315` | pegajoso, espaçado |
| Pôr do sol | `#E2BE8A` | `#0F0F0F` | pegajoso, espaçado |
| Matriz | `#82FB9C` | `#0B0C16` | pegajoso, espaçado |
| Monokai Sombrio | `#78DCE8` | `#2D2A2E` | pegajoso, espaçado |
| Senhora Névoa | `#354B78` | `#131313` | pegajoso, espaçado |
| Uma Luz | `#4078F2` | `#FAFAFA` | pegajoso, espaçado |
| Luz da Floresta Ever | `#3A94C5` | `#FDF6E3` | pegajoso, espaçado |
| Luz Solarizada | `#268BD2` | `#FDF6E3` | pegajoso, espaçado |
| Luz Norte | `#5E81AC` | `#ECEFF4` | pegajoso, espaçado |
| Aguarela | `#1A639C` | `#E9E3D3` | pegajoso, espaçado |

**Sticky** é o layout compacto: janelas lado a lado usam gaps internos pequenos, os gaps externos são zero e a taskbar fica próxima da borda. **Spaced** adiciona gaps maiores, cantos arredondados, sombras e margens às superfícies do shell.

Valores de layout `float` e IDs de tema terminados em `-float` são reconhecidos como nomes legados de Spaced. Novos valores e IDs resolvidos usam `spaced` e `-spaced`.

O valor mostrado na tabela é o accent gravado no `config.json` quando a família é selecionada ou quando **Restaurar padrão do tema** é usado. Um accent RGB escolhido manualmente vale até a próxima troca de tema, que o substitui pelo padrão do tema selecionado.

O modo altera geometria e tratamento das superfícies; não cria uma nova família de cores. Veja [Temas](/pt/docs/user-guide/appearance/) e [Desktop](/pt/docs/user-guide/desktop/) para posicionamento.

* [Temas](/pt/docs/user-guide/appearance/)
* [Wallpapers](/pt/docs/argvus-appearance/wallpapers/)
* [Fontes e ícones](/pt/docs/argvus-appearance/fonts-and-icons/)
* [Efeitos](/pt/docs/argvus-appearance/effects/)

As interfaces estão no Control Center e no Control Panel. A implementação atravessa `argvus-appearance`, `argvus-session` e pacotes de assets.

## Wobble das janelas

Abra **Desktop e entrada → Hyprland → Animações → Janelas** e use os controles
**Wobble**. Busque por `wobble` ou abra a página diretamente:

```sh
argvus control-center hyprland animations windows
```

O Wobble começa desativado. Os padrões do ARGVUS são rigidez `100`, amortecimento
`10`, massa `1`, intensidade `1` e resolução da malha `8`. Rigidez e massa aceitam
`0.0001–1000`, amortecimento `0–1000`, intensidade `0–2` e malha um inteiro `2–32`.
Uma malha maior aumenta o custo de renderização. Use as setas para ajustar valores
ou Enter para digitá-los. **Restaurar padrões do Wobble** altera o rascunho;
**Aplicar** salva todas as edições de animações pendentes e recarrega o Hyprland
uma vez, sem reiniciar a sessão. Voltar dentro de Animações mantém o rascunho
compartilhado.

A chave global de Animações desativa o Wobble sem perder suas definições
individuais. Os valores canônicos ficam em `effects.wobble_*` no estado do ARGVUS;
`hypr/state.lua` é gerado e não deve ser editado manualmente.

O recurso exige Hyprland **0.57.0 ou superior**, garantido pela dependência do
pacote `argvus-hyprland`. A versão 0.56 não é compatível com esta configuração.
O máximo de intensidade segue o [código da versão 0.57.0](https://github.com/hyprwm/Hyprland/blob/v0.57.0/src/config/values/ConfigValues.cpp),
que limita o valor a `2`, embora a documentação mais recente possa indicar `3`.
