---
title: Arquivos de configuração
description: Configuração de usuário, sistema, pacote e arquivos gerados.
slug: pt/0.4.0/docs/reference/configuration-files
---

| Camada | Localização | Significado |
| --- | --- | --- |
| Sistema | `/etc/argvus/` | Configuração administrativa. |
| Defaults | `/usr/share/argvus/` | Defaults e assets somente leitura. |
| Usuário | `$XDG_CONFIG_HOME/argvus/config.json` | Preferências portáveis canônicas. |
| Dados gerenciados | `$XDG_CONFIG_HOME/argvus/data/` | Dados de componentes, compatibilidade e metadados internos. |
| Gerados | `$XDG_CONFIG_HOME/argvus/data/generated/` | Projeções de `config.json`, escritas apenas pelo `argvus-config`. Nunca edite, nunca copie para outro lugar como configuração e nunca leia de volta como fonte. |
| Runtime | `$XDG_RUNTIME_DIR/` e `$XDG_CACHE_HOME/` | Sockets, locks e estado transitório. |

Tema, acento, wallpaper personalizado e efeitos são estado lógico. O modelo lógico gerenciado está documentado em [Configuração canônica](/pt/docs/reference/argvus-config/). Arquivos gerados de Hyprland, GTK, Qt6ct, Waybar, Quickshell, Dunst, Hyprlock, Yazi, Superfile, terminal, fontes e dispositivos removíveis não devem ser tratados como fonte independente.

O `argvus-config` também espelha a camada de tema em `data/waybar/argvus-taskbar.{jsonc,css}` e `data/waybar/argvus-widget-telemetry.{jsonc,css}`. Essas cópias são substituídas a partir dos defaults empacotados em uma mudança de aparência e depois têm seus blocos gerenciados delimitados reescritos, então trate-as como derivadas, não como arquivos mantidos à mão. A configuração nativa de aplicativos em `$XDG_CONFIG_HOME/<app>` é diferente: é alvo de adapter, escrita pelos orquestradores de aparência e nunca pelo projetor.
