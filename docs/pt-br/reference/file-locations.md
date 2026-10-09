---
title: Localizações de arquivos
description: Arquivos e diretórios instalados importantes.
slug: pt/0.4.0/docs/reference/file-locations
---

* `/usr/bin/argvus*` — comandos públicos.
* `/usr/share/argvus/hyprland/` — configuração e helpers Hyprland.
* `/usr/share/argvus/control-panel/` — QML e scripts do painel.
* `/usr/share/argvus/taskbar/` — configuração e scripts da taskbar.
* `/usr/share/argvus/lock/` — configuração Hyprlock.
* `/usr/share/argvus/portal/` — integração de portais.
* `/usr/share/backgrounds/argvus/` — wallpapers.
* `/etc/argvus/` — configuração de provedores.
* `/etc/argvus/greeter.toml` — configuração do greeter.
* `/etc/argvus/removable-devices/config.json` — padrões do sistema para armazenamento removível.
* `$XDG_CONFIG_HOME/argvus/config.json` — preferências portáveis canônicas.
* `$XDG_CONFIG_HOME/argvus/data/` — dados gerenciados, backups, locks e saídas geradas dos componentes.
* `$XDG_CONFIG_HOME/waybar/argvus-taskbar.{jsonc,css}` — overrides nativos completos opcionais da taskbar Waybar.
* `$XDG_CONFIG_HOME/argvus/data/taskbar/argvus-taskbar.{jsonc,css}` — cópias gerenciadas da taskbar pelo ARGVUS.
* `$XDG_CONFIG_HOME/argvus/data/generated/waybar/` — perfis gerados da Waybar, incluindo `argvus-taskbar.css` e `argvus-widget-telemetry.{jsonc,css}`; estes têm precedência sobre as cópias acima.
* `$XDG_CONFIG_HOME/argvus/data/generated/waybar/argvus-widget-telemetry.{jsonc,css}` — também espelhados em `data/waybar/`, que é substituído pelo default empacotado em uma mudança de aparência e depois tem seus blocos `ARGVUS_TELEMETRY_*` e de fonte reescritos.
* `$XDG_CONFIG_HOME/argvus/data/generated/yazi/` — árvore de configuração do Yazi projetada, com fallback consciente de variante para temas sem flavor Yazi.
* `$XDG_CONFIG_HOME/argvus/data/generated/superfile/` — conjunto de temas do Superfile projetado.
* `$XDG_CONFIG_HOME/argvus/data/generated/terminal/` — perfis de terminal projetados.
* `$XDG_CONFIG_HOME/argvus/data/generated/qt6ct/` — paleta do Qt6ct projetada.
* `$XDG_CONFIG_HOME/argvus/data/generated/gtk/` — arquivos de tema do GTK projetados.
* `$XDG_CONFIG_HOME/argvus/data/rofi/` — cópias do usuário da configuração, dos temas e do modo do Rofi. O ARGVUS cria cada cópia quando altera o arquivo pela primeira vez; arquivos não alterados são lidos de `/usr/share/argvus/launcher/config/`.
* `$XDG_CONFIG_HOME/argvus/data/generated/removable-devices/theme.css` — stylesheet projetado de dispositivos removíveis.
* `$XDG_CONFIG_HOME/argvus/data/hypr/` — overrides Lua e projeções nativas do Hyprland.
* `$XDG_CONFIG_HOME/hypr/hyprland.lua` — configuração nativa completa opcional do Hyprland, selecionada pelo `argvus-start`; não é um overlay incremental.
* `$XDG_STATE_HOME/argvus/config-projection.json` — manifesto de projeção: generation, hash da configuração efetiva, seções alteradas e trabalho de runtime pendente. Lido pelo `argvus-sessionctl reload`; nunca é fonte de verdade.
* `$XDG_STATE_HOME/argvus/widget-telemetry-blocks` — preferências legadas de blocos de telemetria. Apenas entrada de migração; os valores canônicos estão no `config.json` e esse arquivo não é mais escrito.
* `$XDG_STATE_HOME/argvus/session.log` — log da sessão gráfica (normalmente `~/.local/state/argvus/session.log`).
* `/usr/share/wayland-sessions/argvus.desktop` — entrada da sessão gráfica.
* `/usr/share/plymouth/themes/argvus/` — assets do splash de boot empacotado.
