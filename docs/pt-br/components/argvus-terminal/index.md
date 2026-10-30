---
title: Terminal
description: Abra o terminal e os perfis de aplicativos TUI.
---

```sh
argvus --terminal
```

`argvus terminal` (um subcomando do `argvus`) fornece a integração e a configuração Kitty. `argvus-app-profiles` fornece perfis para Superfile e Yazi; ele não fornece o dispatcher `argvus`.

O `argvus-config` projeta o tema em `data/generated/terminal/`, e `argvus --yazi` e `argvus --spf` leem sua configuração das árvores projetadas `data/generated/yazi/` e `data/generated/superfile/`, em vez de cópias materializadas na sua pasta pessoal. Um override nativo completo no diretório de configuração do próprio aplicativo ainda tem precedência, então você pode substituir a árvore projetada pela sua configuração.

As trocas de tema regeneram os arquivos derivados do Kitty em `$XDG_CACHE_HOME/argvus/argvus-terminal`. O gerador serializa as atualizações e substitui cada arquivo atomicamente. Se o terminal ainda não abrir, confira o tema ativo e regenere o cache com:

```sh
argvus config get /appearance/theme --raw
argvus terminal --apply
argvus terminal --print-config
```

O terminal lê primeiro o tema e o acento gerenciados em
`$XDG_CONFIG_HOME/argvus/config.json`. Os arquivos legados `.active-theme` e
`.accent-color` continuam como fallback para instalações antigas.

## Transparência e blur

Em **Control Center → Aparência → Terminal**, ajuste separadamente
**Transparência** e **Blur** de `0%` a `100%`. As alterações são aplicadas pelo
botão **Apply** e regeneram o perfil Kitty em
`$XDG_CACHE_HOME/argvus/argvus-terminal`; janelas Kitty existentes recebem a
atualização por reload da configuração.

`0%` de transparência deixa o fundo opaco e `100%` deixa o fundo totalmente
transparente. O blur usa o raio suportado pelo Kitty e só produz efeito quando
a janela tem transparência.

## Aviso de comando longo

Comandos que rodam por 30 segundos ou mais avisam quando terminam, desde que a
janela do terminal esteja sem foco. O aviso usa `notify_on_cmd_finish` do Kitty
com a integração de shell já ativa, então não é preciso alterar `~/.zshrc` ou
`~/.bashrc`. O limite de 30 segundos é fixo na configuração do Kitty.

## Modo App para TUI

O Control Center e os popups TUI da Taskbar usam `argvus tui-terminal` com um
perfil dedicado do Kitty. Ele herda o tema e a fonte ativos do ARGVUS, atribui
uma classe estável à janela e encerra com o processo filho, sem abrir um shell.
O Control Center oculta as abas e possui valores independentes de Transparência
e Blur; os popups de monitoramento e temperatura da taskbar usam uma barra
inferior com o título localizado `Pressione q para fechar`. O launcher aceita
`--title TÍTULO` para definir um título fixo para a janela e a aba do Kitty. O
terminal normal mantém suas abas e configurações.
