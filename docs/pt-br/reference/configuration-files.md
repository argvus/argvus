---
title: Arquivos de configuração
description: Configuração de usuário, sistema, pacote e arquivos gerados.
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

## Exceção à regra de escritor único

`argvus config` é o único escritor de `data/generated/`. Existe uma exceção para migração: ao criar
a cópia do usuário de um arquivo empacotado, um arquivo que uma versão antiga deixou em
`data/generated/` é movido para o local do usuário (`data/`), para que ele mantenha as suas edições.
A movimentação ocorre apenas para arquivos que a versão atual não gera. Não conte com esse caminho
para arquivos novos: um arquivo gerado deve ser criado na projeção, e não aqui.
