---
title: Temas oficiais
description: O catálogo de temas oficiais do ARGVUS, as cores de destaque padrão e como selecionar um tema e o seu modo Sticky ou Float.
---

Esta página lista os temas oficiais disponíveis pelos pacotes de tema oficiais e explica como selecionar um deles. Para instalá-los, veja [Instalando temas](/pt/docs/components/argvus-themes/installing-themes/).

## Selecionar um tema

1. Abra **Control Center → Aparência → Temas**.
2. Escolha **Temas oficiais**.
3. Escolha **Escuro** ou **Claro**.
4. Escolha a família do tema. O tema é aplicado assim que você o seleciona.

O tema selecionado é aplicado a todo o desktop de uma vez: a taskbar, os painéis, as notificações, as janelas e o wallpaper em execução mudam de cor no lugar, sem tela de carregamento e sem reiniciar. O ARGVUS mantém a seleção de wallpaper, a menos que o tema seja aplicado com a associação de wallpaper dele.

Você também pode pressionar `SUPER + SHIFT + T` para abrir o seletor de temas pelo teclado. Use as setas para percorrer as categorias, `Enter` para escolher e `Esc` para fechar.

## Sticky e Float

Cada tema funciona em dois modos. O modo é independente do tema: trocar de tema mantém o modo atual, e trocar de modo mantém o tema atual.

| Modo | Espaçamento entre janelas | Margem externa | Arredondamento das janelas | Margem da taskbar e do shell |
| --- | ---: | ---: | ---: | ---: |
| Sticky | `2` | `0` | `0` | próxima à borda |
| Float | `10` | `18` | `4` | `18` |

Para trocar o modo, abra **Control Center → Aparência → Modo** e escolha **Fixo** (Sticky) ou **Flutuante** (Float). Os ajustes manuais desses valores de layout continuam valendo até a próxima seleção explícita de tema.

## Temas escuros

| Tema | Pacote | Destaque padrão |
| --- | --- | --- |
| One Dark | `argvus-theme-one-dark` | `#61AFEF` |
| Dracula | `argvus-theme-dracula` | `#BD93F9` |
| Silver Dark | `argvus-theme-silver-dark` | `#595959` |
| Slate Dark | `argvus-theme-slate-dark` | `#7391A5` |
| Universe | `argvus-theme-universe` | `#EEEEEE` |
| Gruvbox High Dark | `argvus-theme-gruvbox-high-dark` | `#D79921` |
| Gruvbox Dark | `argvus-theme-gruvbox-dark` | `#D4BE98` |
| Rosé Pine | `argvus-theme-rose-pine` | `#C4A7E7` |
| Tokyo Night | `argvus-theme-tokyo-night` | `#7AA2F7` |
| Solitude | `argvus-theme-solitude` | `#798186` |
| Sunset | `argvus-theme-sunset` | `#E2BE8A` |
| Matrix | `argvus-theme-matrix` | `#82FB9C` |
| Monokai Dark | `argvus-theme-monokai-dark` | `#78DCE8` |

## Temas claros

| Tema | Pacote | Destaque padrão |
| --- | --- | --- |
| GitHub Light | `argvus-theme-github-light` | `#0969DA` |
| Solarized Light | `argvus-theme-solarized-light` | `#268BD2` |
| Nord Light | `argvus-theme-nord-light` | `#5E81AC` |
| One Light | `argvus-theme-one-light` | `#4078F2` |
| Everforest Light | `argvus-theme-everforest-light` | `#3A94C5` |
| Catppuccin Latte | `argvus-theme-catppuccin-latte` | `#1E66F5` |
| Gruvbox Light | `argvus-theme-gruvbox-light` | `#458588` |
| Watercolor | `argvus-theme-watercolor` | `#1A639C` |

## Temas nativos

**ARGVUS Dark** e **ARGVUS Light** são fornecidos por `argvus-appearance` e estão sempre disponíveis em **Temas oficiais**. O ARGVUS Dark usa o destaque padrão `#3590BD`; o ARGVUS Light usa `#181818`.

## O que um tema controla

Um tema define a paleta e o tratamento das superfícies usados pelos componentes do desktop, a cor de destaque padrão, a associação padrão de wallpaper e os valores padrão de transparência e blur. Os componentes do desktop leem esse estado em vez de guardar uma cópia própria, então uma troca de tema chega ao compositor, à taskbar, ao Control Panel, à tela de bloqueio, às notificações e aos aplicativos ao mesmo tempo.

A cor de destaque padrão é apenas o ponto de partida. Você pode substituí-la por qualquer cor de seis dígitos, veja [Personalizando a cor de destaque](/pt/docs/components/argvus-themes/highlight-color/).
