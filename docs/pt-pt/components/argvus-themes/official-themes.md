---
title: Temas oficiais
description: O catálogo de temas oficiais do ARGVUS, as cores de destaque padrão e como selecionar um tema e o seu modo Sticky ou Spaced.
---

Esta página lista os temas oficiais disponíveis pelos pacotes de tema oficiais e explica como selecionar um deles. Para instalá-los, veja [Instalando temas](/pt/docs/components/argvus-themes/installing-themes/).

## Selecionar um tema

1. Abra **Control Center → Aparência → Temas**.
2. Escolha **Temas oficiais**.
3. Escolha **Escuro** ou **Claro**.
4. Escolha a família do tema. O tema é aplicado assim que você o seleciona.

O tema selecionado é aplicado a todo o desktop de uma vez: a taskbar, os painéis, as notificações, as janelas e o wallpaper em execução mudam de cor no lugar, sem ecrã de carregamento e sem reiniciar. O ARGVUS mantém a seleção de wallpaper, a menos que o tema seja aplicado com a associação de wallpaper dele.

Você também pode pressionar `SUPER + SHIFT + T` para abrir o seletor de temas pelo teclado. Use as setas para percorrer as categorias, `Enter` para escolher e `Esc` para fechar.

## Sticky e Spaced

Cada tema funciona em dois modos. O modo é independente do tema: trocar de tema mantém o modo atual, e trocar de modo mantém o tema atual.

| Modo | Espaçamento entre janelas | Margem externa | Arredondamento das janelas | Margem da taskbar e do shell |
| --- | ---: | ---: | ---: | ---: |
| Sticky | `2` | `0` | `0` | próxima à borda |
| Espaçado | `10` | `18` | `4` | `18` |

Para trocar o modo, abra **Control Center → Aparência → Modo** e escolha **Fixo** (Sticky) ou **Flutuante** (Spaced). Os ajustes manuais desses valores de layout continuam valendo até a próxima seleção explícita de tema.

## Temas escuros

| Tema | Pacote | Destaque padrão |
| --- | --- | --- |
| Um Escuro | `argvus-theme-one-dark` | `#61AFEF` |
| Drácula | `argvus-theme-dracula` | `#BD93F9` |
| Prata Escuro | `argvus-theme-silver-dark` | `#595959` |
| Ardósia Escura | `argvus-theme-slate-dark` | `#7391A5` |
| Universo | `argvus-theme-universe` | `#EEEEEE` |
| Gruvbox alto escuro | `argvus-theme-gruvbox-high-dark` | `#D79921` |
| Gruvbox Escuro | `argvus-theme-gruvbox-dark` | `#D4BE98` |
| Pinho Rosé | `argvus-theme-rose-pine` | `#C4A7E7` |
| Noite de Tóquio | `argvus-theme-tokyo-night` | `#7AA2F7` |
| Solidão | `argvus-theme-solitude` | `#798186` |
| Pôr do sol | `argvus-theme-sunset` | `#E2BE8A` |
| Matriz | `argvus-theme-matrix` | `#82FB9C` |
| Monokai Sombrio | `argvus-theme-monokai-dark` | `#78DCE8` |
| Senhora Névoa | `argvus-theme-lady-mist` | `#354B78` |

## Temas claros

| Tema | Pacote | Destaque padrão |
| --- | --- | --- |
| Luz GitHub | `argvus-theme-github-light` | `#0969DA` |
| Luz Solarizada | `argvus-theme-solarized-light` | `#268BD2` |
| Luz Norte | `argvus-theme-nord-light` | `#5E81AC` |
| Uma Luz | `argvus-theme-one-light` | `#4078F2` |
| Luz da Floresta Ever | `argvus-theme-everforest-light` | `#3A94C5` |
| Catppuccin Latte | `argvus-theme-catppuccin-latte` | `#1E66F5` |
| Luz Gruvbox | `argvus-theme-gruvbox-light` | `#458588` |
| Aguarela | `argvus-theme-watercolor` | `#1A639C` |
| Luz Obsidiana | `argvus-theme-obsidian-light` | `#334A78` |

## Temas nativos

**ARGVUS Dark** e **ARGVUS Light** são fornecidos por `argvus-appearance` e estão sempre disponíveis em **Temas oficiais**. O ARGVUS Dark usa o destaque padrão `#3590BD`; o ARGVUS Light usa `#181818`.

## O que um tema controla

Um tema define a paleta e o tratamento das superfícies usados pelos componentes do desktop, a cor de destaque padrão, a associação padrão de wallpaper e os valores padrão de transparência e blur. Os componentes do desktop leem esse estado em vez de guardar uma cópia própria, então uma troca de tema chega ao compositor, à taskbar, ao Control Panel, à ecrã de bloqueio, às notificações e às aplicações ao mesmo tempo.

A cor de destaque padrão é apenas o ponto de partida. Você pode substituí-la por qualquer cor de seis dígitos, veja [Personalizando a cor de destaque](/pt/docs/components/argvus-themes/highlight-color/).
