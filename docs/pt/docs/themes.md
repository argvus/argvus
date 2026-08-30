---
title: Temas
description: Famílias de tema do ARGVUS e cores de destaque.
---

Temas disponíveis: `argvus-dark-aether`, `argvus-dark-aether-float`, `argvus-dark-silver`, `argvus-dark-silver-float`, `argvus-light-veil`, `argvus-light-veil-float`, `argvus-dark-slate` e `argvus-dark-slate-float`. Mudanças de accent afetam bordas, títulos, seleções e outros destaques sem substituir o fundo do tema.

Abra o seletor com `SUPER + SHIFT + A`, use o controle **Accent** na sidebar, ou execute:

```sh
sh /usr/share/argvus/argvus/sh/accent-switch.sh
```

Você também pode aplicar uma das cores suportadas diretamente:

```sh
sh /usr/share/argvus/argvus/sh/accent-switch.sh '#17d174'
```

Paleta: `#996548`, `#3590bd`, `#7391a5`, `#17d174`, `#cb17d1`, `#d1174f`, `#d1ce17`, `#9617d1` e `#595959`. Os accents padrão são `#3590bd` para Dark, `#595959` para Dark Silver, `#181818` para Light e `#7391a5` para Slate. Trocar de tema substitui qualquer accent personalizado pelo padrão do tema selecionado.

Preferências de tema e accent são armazenadas como arquivos pequenos do usuário em `$XDG_CONFIG_HOME/argvus`. A configuração de runtime das aplicações gerada a partir dessas preferências é escrita em `$XDG_STATE_HOME/argvus/config`, usando `~/.local/state` quando `XDG_STATE_HOME` não estiver definido.

Temas dark usam `default.png`; Light usa `argvus-light-veil.png`; Slate usa `argvus-dark-slate.png`; e Dark Silver usa `argvus-dark-silver.png`. Variantes Normal e Float de cada família compartilham o mesmo wallpaper. Os wallpapers são entregues pelo pacote `argvus-appearance` em `/usr/share/backgrounds/argvus/`.
