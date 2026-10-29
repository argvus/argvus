---
title: Localizações de arquivos
description: Arquivos e diretórios instalados importantes.
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
* `$XDG_CONFIG_HOME/argvus/data/focus/settings.json`, `$XDG_DATA_HOME/argvus/focus/history.json`, `$XDG_STATE_HOME/argvus/focus/session.json` — configurações, histórico de sessões e checkpoint da sessão atual do ARGVUS Focus.
* `$XDG_RUNTIME_DIR/argvus/focus/status.json` — status temporário do ARGVUS Focus consumido pela taskbar; removido quando o Focus encerra.
* `$XDG_CONFIG_HOME/argvus/data/clock/settings.json`, `$XDG_DATA_HOME/argvus/clock/alarms.json`, `$XDG_STATE_HOME/argvus/clock/timers.json` — configurações e relógios mundiais, alarmes e checkpoint do temporizador e do cronômetro do ARGVUS Clock.
* `$XDG_CONFIG_HOME/argvus/data/generated/theme/current/time/theme.css` — cores do ARGVUS Focus e do ARGVUS Clock (consumidor de tema `time`).
* `/usr/share/argvus/sounds/` — sons do ARGVUS Focus e do ARGVUS Clock (`argvus-chime.oga`, `argvus-beacon.oga`, `argvus-bell.oga`).
* `$XDG_CONFIG_HOME/argvus/data/generated/theme/` — builds de tema (`builds/<id>-<variante>-<hash>/`), gravados pelo `argvus theme render` e por toda troca de tema e de destaque, e o symlink `current` para o build do tema ativo.
* `$XDG_CONFIG_HOME/argvus/data/hypr/` — overrides Lua e projeções nativas do Hyprland.
* `$XDG_CONFIG_HOME/hypr/hyprland.lua` — configuração nativa completa opcional do Hyprland, selecionada pelo `argvus start`; não é um overlay incremental.
* `$XDG_STATE_HOME/argvus/config-projection.json` — manifesto de projeção: generation, hash da configuração efetiva, seções alteradas e trabalho de runtime pendente. Lido pelo `argvus sessionctl reload`; nunca é fonte de verdade.
* `$XDG_STATE_HOME/argvus/widget-telemetry-blocks` — preferências legadas de blocos de telemetria. Apenas entrada de migração; os valores canônicos estão no `config.json` e esse arquivo não é mais escrito.
* `$XDG_STATE_HOME/argvus/session.log` — log da sessão gráfica (normalmente `~/.local/state/argvus/session.log`).
* `/usr/share/wayland-sessions/argvus.desktop` — entrada da sessão gráfica.
* `/usr/share/plymouth/themes/argvus/` — assets do splash de boot empacotado.
