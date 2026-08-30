---
title: Calendário
description: O popup de calendário nativo do ARGVUS para a Waybar.
---

O `argvus-calendar` é o popup de calendário nativo do ARGVUS para Wayland/Hyprland, escrito em Rust com Relm4 e GTK4 e posicionado com `gtk4-layer-shell`. Ele abre pelo módulo de data da Waybar e oferece uma visão de mês compacta, agenda, editor de eventos e lembretes.

## Abrir

Clique na data na barra superior da Waybar. O módulo é configurado como `clock#date` com `"on-click"` ligado à ação de calendário do taskbar do ARGVUS, que executa:

```sh
argvus-calendar toggle
```

O popup é uma superfície layer-shell de instância única: ele continua rodando depois de ocultado, então `toggle`, `show` e `hide` passam por um socket IPC Unix em `$XDG_CACHE_HOME/argvus-calendar/argvus-calendar.sock`. Ele fecha ao clicar fora dele, clicar na data novamente, pressionar Escape ou perder o foco.

## Comandos

```sh
argvus-calendar                 # alterna o popup
argvus-calendar show            # mostra o popup
argvus-calendar hide            # oculta o popup
argvus-calendar config          # abre o arquivo de configuração em um terminal com sudo
argvus-calendar config-path     # mostra o caminho da configuração
argvus-calendar reload          # recarrega o CSS/tema externo no popup em execução
argvus-calendar import file.ics # importa um arquivo ICS
argvus-calendar export --output calendar.ics
argvus-calendar sync            # roda uma sincronização com a conta CalDAV/WebDAV configurada
argvus-calendar status          # mostra status do banco/config/estilo/tema
argvus-calendar service         # roda o agendador de lembretes
```

## Configuração

Toda a configuração vive em `/etc/argvus-calendar/config.toml`. O aplicativo usa defaults embutidos quando o arquivo não existe. Abra-o para editar com `argvus-calendar config`, que abre um terminal com `sudo` (terminal de `$TERMINAL` ou da opção `[terminal]`, editor de `$VISUAL`/`$EDITOR` ou da opção `[editor]`, padrão `nano`).

```toml
[appearance]
font_family = "monospace"
font_size = 12 # intervalo válido: 8-32

[locale]
language = "en-US" # en-US | pt-BR

[calendar]
week_start = "monday" # monday | sunday
default_event_duration_minutes = 60
default_reminder_minutes = 10
sync_interval_minutes = 15

[popup]
enable = false # false abre abaixo do clique/cursor da Waybar
anchor = "top-right" # usado apenas quando enable = true
margin_top = 8
margin_right = 10
margin_bottom = 8
margin_left = 8

[editor]
command = "" # usa $VISUAL, depois $EDITOR, depois nano
args = []

[terminal]
command = "" # usa $TERMINAL, depois kitty
args = []
```

O botão de engrenagem no popup abre o mesmo arquivo de configuração. O popup recarrega a configuração a cada reinício; `argvus-calendar reload` atualiza o CSS/tema externo no popup em execução.

## Caminhos

```text
/etc/argvus-calendar/config.toml    configuração de sistema (o pacote entrega um default)
/etc/argvus-calendar/style.css      estilos de estrutura do popup
/etc/argvus-calendar/theme.css      tema padrão empacotado
/etc/argvus-calendar/themes/        arquivos de tema ARGVUS
$XDG_DATA_HOME/argvus-calendar/     banco de dados SQLite
$XDG_STATE_HOME/argvus-calendar/    estado de runtime
$XDG_CACHE_HOME/argvus-calendar/    socket IPC, toggle de eventos e tema ativo
```

## Eventos e lembretes

Os eventos ficam em um banco de dados SQLite. Os lembretes podem ser desativados ou configurados em horas e minutos antes do início; eventos de dia inteiro podem repetir a notificação em um intervalo escolhido durante o dia. Eventos encerrados antes do dia local atual são removidos permanentemente, e novos eventos não podem ser criados em datas passadas.

Os lembretes são confiáveis enquanto o `argvus-calendar service` estiver rodando, ou quando o serviço systemd de usuário fornecido estiver habilitado:

```sh
systemctl --user enable --now argvus-calendar.service
```

A seção de eventos é controlada pelo toggle dentro da janela e persistida em `$XDG_CACHE_HOME/argvus-calendar/events-enabled`; ela não é configurada em `config.toml`.

Importação e exportação ICS são suportadas por `argvus-calendar import` e `argvus-calendar export`. O suporte a CalDAV é implementado como uma base de cliente HTTP/XML interno mantida; o gerenciamento de contas e a reconciliação completa do banco são o próximo passo de integração.

## Estilo

O estilo é externo: `/etc/argvus-calendar/style.css` fornece a estrutura, `/etc/argvus-calendar/theme.css` fornece o tema padrão empacotado, `/etc/argvus-calendar/themes/` contém os temas ARGVUS, e `$XDG_CACHE_HOME/argvus-calendar/theme.css` é o tema ativo em nível de usuário escrito pelo alternador de temas do ARGVUS.

Temas ARGVUS suportados: Dark, Dark Float, Dark Silver, Dark Silver Float, Slate e Slate Float. O popup acompanha a família de tema do desktop e lê o tema ativo de `$XDG_CONFIG_HOME/argvus/.active-theme`.

## Pacote

O `argvus-calendar` é empacotado pelo [`argvus-pkgbuild`](https://github.com/argvus/argvus-pkgbuild) em `arch/argvus-calendar/PKGBUILD`. Ele instala `/usr/bin/argvus-calendar`, os defaults em `/etc/argvus-calendar/` e a unit systemd de usuário em `/usr/lib/systemd/user/argvus-calendar.service`.
