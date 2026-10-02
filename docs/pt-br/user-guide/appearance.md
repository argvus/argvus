---
title: Aparência
description: Configure a linguagem visual compartilhada do ARGVUS.
slug: pt/0.4.0/docs/user-guide/appearance
---

O sistema de aparência aplica um estado comum ao Hyprland, GTK, Qt, Waybar, Quickshell, Rofi, Dunst, Hyprlock e aplicativos ARGVUS.

## Famílias de tema

As vinte e quatro famílias abaixo são definidas pelo payload atual de temas. A coluna de cor mostra o valor restaurado pela ação **Restaurar padrão do tema**; o usuário pode escolher outra cor RGB válida independentemente.

| Família | Cor de destaque restaurada | Fundo base | Modos |
| --- | --- | --- | --- |
| One Dark | `#61AFEF` | `#282C34` | Sticky, Float |
| Dracula | `#BD93F9` | `#282A36` | Sticky, Float |
| ARGVUS Dark | `#3590bd` | `#111316` | Sticky, Float |
| Dark Silver | `#595959` | `#111316` | Sticky, Float |
| Dark Slate | `#7391a5` | `#2f3541` | Sticky, Float |
| Dark Universe | `#eeeeee` | `#000000` | Sticky, Float |
| ARGVUS Dark Gruvbox High | `#D79921` | `#282828` | Sticky, Float |
| ARGVUS Dark Gruvbox | `#D4BE98` | `#282828` | Sticky, Float |
| ARGVUS Light | `#181818` | `#f7f7f7` | Sticky, Float |
| GitHub Light | `#0969DA` | `#FFFFFF` | Sticky, Float |
| Catppuccin Latte | `#1E66F5` | `#EFF1F5` | Sticky, Float |
| ARGVUS Light Gruvbox | `#458588` | `#FBF1C7` | Sticky, Float |
| Rosé Pine | `#C4A7E7` | `#191724` | Sticky, Float |
| Tokyo-Night | `#7AA2F7` | `#1A1B26` | Sticky, Float |
| Solitude | `#798186` | `#101315` | Sticky, Float |
| Sunset | `#E2BE8A` | `#0F0F0F` | Sticky, Float |
| Hackerman | `#82FB9C` | `#0B0C16` | Sticky, Float |
| Monokai Dark | `#78DCE8` | `#2D2A2E` | Sticky, Float |
| One Light | `#4078F2` | `#FAFAFA` | Sticky, Float |
| Flexoki Light | `#205EA6` | `#FFFCF0` | Sticky, Float |
| Everforest Light | `#3A94C5` | `#FDF6E3` | Sticky, Float |
| ARGVUS Kanagawa Lotus | `#4D699B` | `#F2ECBC` | Sticky, Float |
| Nord Light | `#5E81AC` | `#ECEFF4` | Sticky, Float |

**Sticky** é o layout compacto: janelas lado a lado usam gaps internos pequenos, os gaps externos são zero e a taskbar fica próxima da borda. **Float** adiciona gaps maiores, cantos arredondados, sombras e margens às superfícies do shell.

O valor mostrado na tabela é o accent gravado no `config.json` quando a família é selecionada ou quando **Restaurar padrão do tema** é usado. Um accent RGB escolhido manualmente vale até a próxima troca de tema, que o substitui pelo padrão do tema selecionado.

O modo altera geometria e tratamento das superfícies; não cria uma nova família de cores. Veja [Temas e acentos](./appearance/themes/) e [Desktop](./desktop/) para posicionamento.

* [Temas e acentos](./appearance/themes/)
* [Wallpapers](./appearance/wallpapers/)
* [Fontes e ícones](./appearance/fonts-and-icons/)
* [Efeitos](./appearance/effects/)

As interfaces estão no Control Center e no Control Panel. A implementação atravessa `argvus-appearance`, `argvus-session` e pacotes de assets.
