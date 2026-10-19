---
title: Temas
description: Instale, selecione, personalize, importe e exporte temas do ARGVUS.
---

Um tema no ARGVUS define as cores, as superfícies, o wallpaper padrão e a geometria de layout usados em todo o desktop: Hyprland, Waybar, Control Panel, taskbar, tela de bloqueio, notificações e aplicativos do ARGVUS. Este guia reúne tudo o que você precisa saber para trabalhar com temas como usuário.

## O que um tema inclui

Cada tema oficial é uma identidade visual completa com dois modos:

- **Sticky** — layout compacto, com pequenos espaçamentos entre janelas, sem margem externa e com a taskbar próxima à borda da tela.
- **Float** — layout espaçoso, com espaçamentos maiores, cantos arredondados e margens ao redor da taskbar e das superfícies do shell.

Selecionar um tema não altera a identidade dele: a paleta, a cor de destaque padrão e a associação com o wallpaper permanecem as mesmas entre os modos. O modo é uma configuração separada, veja [Temas oficiais](/pt/docs/components/argvus-themes/official-themes/).

## Páginas deste guia

- [Instalando temas](/pt/docs/components/argvus-themes/installing-themes/) — de onde vêm os temas oficiais e como instalar um ou todos com `pacman`.
- [Temas oficiais](/pt/docs/components/argvus-themes/official-themes/) — o catálogo, a cor de destaque padrão de cada tema e como escolher um tema e o seu modo.
- [Personalizando a cor de destaque](/pt/docs/components/argvus-themes/highlight-color/) — altere a cor de destaque independentemente do tema.
- [Importando e exportando temas](/pt/docs/components/argvus-themes/import-and-export/) — salve a aparência atual como perfil `.tar.gz` e restaure-a depois ou em outra máquina.

## Referência rápida

| Tarefa | Onde |
| --- | --- |
| Escolher um tema | **Control Center → Aparência → Temas → Temas oficiais** |
| Alternar entre Sticky e Float | **Control Center → Aparência → Modo** |
| Alterar a cor de destaque | **Control Center → Aparência → Cor de destaque** |
| Exportar a aparência atual | **Control Center → Aparência → Temas → Exportar / Importar → Exportar perfil do tema** |
| Importar um perfil | **Control Center → Aparência → Temas → Exportar / Importar → Importar perfil do tema** |
| Abrir o seletor de temas pelo teclado | `SUPER + SHIFT + T` |

## De onde vêm os temas

O ARGVUS inclui dois temas nativos, **ARGVUS Dark** e **ARGVUS Light**, junto com o pacote `argvus-appearance`. Todos os outros temas oficiais são pacotes separados `argvus-theme-<nome>`. Ao instalar um pacote de tema, ele passa a aparecer no seletor de temas; não é preciso reiniciar a sessão. Perfis importados ficam separados, em **Temas personalizados**.

## Atalhos de teclado

As páginas de temas do Control Center mostram as teclas no rodapé:

- `↑` / `↓` — navegar.
- `Enter` — aplicar ou abrir.
- `e` — exportar a aparência atual, na página de Temas.
- `i` — importar um perfil, na página de Temas.
- `d` — excluir um tema personalizado importado.
- `Esc` — voltar.

## Solução de problemas

- **Um tema que instalei não aparece.** Confirme que o pacote está instalado com `pacman -Q argvus-theme-<nome>` e depois reabra o Control Center. O pacote precisa de `argvus-appearance` instalado.
- **A cor de destaque mudou depois que troquei de tema.** Uma cor de destaque personalizada sobrevive às trocas de tema. Use **Restaurar padrão do tema** para devolver ao tema a cor dele.
- **Um perfil importado aparece, mas o wallpaper é diferente.** O perfil usa o wallpaper oficial quando o wallpaper salvo não pode ser restaurado. Veja [Importando e exportando temas](/pt/docs/components/argvus-themes/import-and-export/).
