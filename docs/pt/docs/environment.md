---
title: Ambiente
description: Como o ambiente desktop ARGVUS é organizado.
---

ARGVUS é um desktop completo, pronto para uso, focado em Hyprland. A configuração do desktop vive no repositório [`argvus`](https://github.com/argvus/argvus) e é empacotada como o pacote Arch Linux `argvus`.

O pacote instala padrões somente leitura em `/usr/share/argvus` e o helper opcional `argvus-setup` em `/usr/bin`. Ele nunca escreve em `$HOME` durante a instalação do pacote.

## Pacotes de componentes

O desktop é dividido em pacotes focados, todos construídos pelo `argvus-pkgbuild` e publicados no mesmo repositório:

| Pacote | É dono de |
| --- | --- |
| `argvus` | Configuração do desktop em `/usr/share/argvus` e do helper opcional `/usr/bin/argvus-setup`. |
| `argvus-session` | Launchers de sessão `/usr/bin/argvus-session`, `/usr/bin/argvus-start`, `/usr/bin/argvus-tty` e `/usr/share/wayland-sessions/argvus.desktop`. |
| `argvus-appearance` | Wallpapers compartilhados em `/usr/share/backgrounds/argvus` e fontes empacotadas em `/usr/share/fonts`. |
| `argvus-storage` | O módulo Waybar de armazenamento removível (binário, configuração e temas). |
| `argvus-calendar` | O popup de calendário nativo da Waybar (binário, configuração e temas). |
| `argvus-greeter` | A integração com o greeter greetd. |

Instalar `argvus` puxa os demais pacotes de componentes como dependências.

## Composição do desktop

| Área | Arquivos | Função |
| --- | --- | --- |
| Hyprland | `config/hypr/hyprland.lua` | Configuração principal do compositor, atalhos, workspaces, regras e inicialização. |
| Setup | `bin/argvus-setup` | Helper opcional que copia os padrões empacotados para o diretório de configuração do usuário. |
| Waybar | `config/waybar/config.jsonc`, `config/waybar/style.css` | Barra superior com workspaces, mídia, janela, tray, estado do sistema e módulos ARGVUS. |
| Sidebar | `config/quickshell/sidebar-right/` | Painel Quickshell para ajustes, sistema, calendário, rede, volume, brilho e energia. |
| Launchers | `config/rofi/`, `config/wofi/` | Lançadores, menus e superfícies de comando com tema. |
| Terminal e TUI | `config/kitty/`, `config/yazi/`, `config/superfile/`, `config/btop/`, `config/bottom/` | Terminal, gerenciadores de arquivos e monitores do sistema. |
| Notificações e logout | `config/dunst/`, `config/wlogout/` | Estilo de notificações e interface de energia/logout. |

## Fluxo de sessão

`argvus-session` é o ponto de entrada do display manager. Ele prepara o ambiente e executa `argvus-start`; não copia configuração para `$HOME`.

`argvus-start` inicia o Hyprland com a configuração ARGVUS. Ele usa `$XDG_CONFIG_HOME/hypr/hyprland.lua` quando esse override completo do usuário existe; caso contrário usa o empacotado `/usr/share/argvus/hypr/hyprland.lua`. A configuração Lua empacotada também carrega pequenos arquivos opcionais de override do usuário em `$XDG_CONFIG_HOME/argvus/hypr/`.

`argvus-tty` oferece um caminho manual para iniciar o ARGVUS em sistemas sem display manager. Ele registra uma sessão logind, inicia o session bus do D-Bus, define o ambiente Wayland/XDG e inicia a mesma sessão.

## Modelo de configuração

O ARGVUS não copia dotfiles para `$HOME` antes de o desktop poder iniciar. Os entrypoints de runtime usam esta prioridade:

```text
$XDG_CONFIG_HOME/<app>  ->  $XDG_STATE_HOME/argvus/config/<app>  ->  /usr/share/argvus/<app>  ->  defaults upstream
```

Se `XDG_STATE_HOME` não estiver definido, o ARGVUS usa o fallback padrão `~/.local/state`, então a configuração de runtime gerada fica em `~/.local/state/argvus/config`.

A responsabilidade de cada local é:

| Local | Responsabilidade |
| --- | --- |
| `/usr/share/argvus` | Defaults imutáveis empacotados e de propriedade do pacman. Upgrades de pacote podem substituir esses arquivos. |
| `$XDG_STATE_HOME/argvus/config` | Configuração de runtime gerada novamente a partir dos defaults atuais do pacote mais as preferências do ARGVUS. |
| `$XDG_CONFIG_HOME/argvus` | Preferências pequenas do usuário como `.active-theme`, `.accent-color`, `.spaces`, `.weather-location` e overrides Lua opcionais do Hyprland. |
| `$XDG_CONFIG_HOME/<app>` | Overrides explícitos e completos de aplicações, criados manualmente ou com `argvus-setup`. |
| `$XDG_CACHE_HOME/argvus` | Cache de runtime, como o wallpaper gerado do Hyprlock. |

`argvus-setup` é opcional. Use-o apenas quando quiser copiar os padrões empacotados para `$XDG_CONFIG_HOME` para personalizar:

| Modo | Comportamento |
| --- | --- |
| `--copy <app>` | Copia `/usr/share/argvus/<app>` para `$XDG_CONFIG_HOME/<app>`. |
| `--copy-all` | Copia todos os padrões empacotados. |
| `--force` | Faz backup do destino existente como `$XDG_CONFIG_HOME/<app>.bak-<timestamp>` antes de copiar. |
| `--dry-run` | Mostra as ações sem alterar arquivos. |
| `--repair` | Alias de compatibilidade para `--copy argvus`. |

Depois de copiados, os arquivos em `$XDG_CONFIG_HOME` são overrides do usuário e upgrades de pacote nunca os substituem.

## Hyprland

O arquivo principal do compositor é `config/hypr/hyprland.lua`. Ele carrega tema ativo, cor de destaque e espaçamentos antes de definir a sessão.

Padrões importantes:

- `SUPER` é o modificador principal.
- O layout é `dwindle`, com grupos habilitados.
- O teclado usa `br,us` com `grp:alt_shift_toggle`.
- Firefox roda nativamente em Wayland por `MOZ_ENABLE_WAYLAND=1`.
- Apps Qt usam o tema configurado e o estilo Qt do Hyprland quando disponível.
- XWayland continua habilitado para compatibilidade.

Overrides opcionais do usuário para Hyprland ficam em `$XDG_CONFIG_HOME/argvus/hypr/` e são carregados depois dos defaults empacotados nesta ordem:

```text
monitors.lua
rules.lua
bindings.lua
user.lua
```

Arquivos de override ausentes são ignorados.

Scripts auxiliares do Hyprland ficam em `config/hypr/scripts/` e cobrem screenshots, menu de energia, seleção de wallpaper, cheatsheets, inicialização da Waybar e âncora do menu do ARGVUS Storage.

## Waybar

ARGVUS usa Waybar como barra superior principal. O layout padrão agrupa workspaces e mídia à esquerda, contexto da janela no centro e módulos do sistema à direita.

Módulos importantes:

- `hyprland/workspaces` para navegação de workspaces.
- `mpris` para mídia.
- `network`, `memory`, `cpu`, scripts de temperatura CPU/GPU e perfil de energia.
- `custom/storage`, alimentado por `argvus-storage watch`.
- `clock#date`, que alterna o popup de calendário nativo (`argvus-calendar`) ao clicar.
- `pulseaudio#input` e `pulseaudio#output` com clique e scroll.
- `tray` para aplicativos com ícones de status.
- `custom/settings`, que alterna a sidebar Quickshell.

Scripts de detalhes do sistema ficam em `config/waybar/scripts/sysinfo/`; a visão auxiliar é configurada por `config/waybar/sysinfo.jsonc` e `config/waybar/sysinfo.css`.

## Sidebar Quickshell

A sidebar direita vive em `config/quickshell/sidebar-right`. Ela é uma interface QML para ações repetidas do desktop.

Ela inclui cards para:

- aparência e temas;
- brilho e volume;
- calendário;
- layout do teclado;
- rede;
- notificações;
- energia;
- espaçamento dos workspaces;
- informações do sistema;
- clima.

O botão de configurações da Waybar alterna essa sidebar pelo script de taskbar do ARGVUS.

## Temas

ARGVUS entrega oito famílias de tema:

- `argvus-dark`
- `argvus-dark-float`
- `argvus-dark-silver`
- `argvus-dark-silver-float`
- `argvus-light`
- `argvus-light-float`
- `argvus-slate`
- `argvus-slate-float`

Os temas são espelhados entre Hyprland, Waybar, Rofi, Dunst, Kitty, Btop, Bottom, Wlogout, Yazi, Superfile, Snappy Switcher e esquemas de cor Qt. O tema ativo é aplicado por `config/argvus/sh/theme-switch.sh`.

A cor de destaque é controlada separadamente por `config/argvus/sh/accent-switch.sh`, então o usuário troca o accent sem substituir a família de tema.

Módulos compartilhados de shell ficam em `config/argvus/sh/` e são carregados pelo `bootstrap.sh`, que expõe helpers de log, caminhos, notificações, JSON e Hyprland para os scripts.

Wallpapers e fontes empacotadas são de propriedade do pacote `argvus-appearance` e referenciados a partir de caminhos de sistema como `/usr/share/backgrounds/argvus/`.

## Caminhos empacotados

O pacote `argvus` instala:

```text
/usr/bin/argvus-setup
/usr/share/argvus/
```

O pacote `argvus-session` instala os entrypoints de sessão:

```text
/usr/bin/argvus-session
/usr/bin/argvus-start
/usr/bin/argvus-tty
/usr/share/wayland-sessions/argvus.desktop
```

`argvus-storage` e `argvus-calendar` são empacotados separadamente e instalam os próprios binários e defaults de sistema. Veja [Apps oficiais](./official-apps/), [Calendário](./calendar/) e [Armazenamento Removível](./storage/).
