---
title: Dispositivos removíveis
description: Monte e gerencie armazenamento removível.
slug: pt/0.4.0/docs/user-guide/applications/removable-devices
---

`argvus-removable-devices` monitora UDisks2 e fornece o módulo de armazenamento da ARGVUS Taskbar e o menu dos dispositivos. O menu também está disponível como uma interface separada no estilo Rofi, permitindo usar as mesmas ações pela taskbar ou por um fluxo de launcher.

As ações disponíveis são `open`, `mount`, `unmount`, `eject`, `poweroff`, `lock`, `unlock` e `copy`. Os comandos `list`, `watch` e `menu` expõem respectivamente as interfaces de status, fluxo da taskbar e menu para integrações.

A ação padrão de abrir arquivos segue o gerenciador de arquivos selecionado no ARGVUS Control Center. Assim, as ações de dispositivos removíveis permanecem consistentes com os aplicativos padrão do usuário.

A configuração do sistema é `/etc/argvus/removable-devices/config.json`. A projeção gerenciada por usuário fica em `~/.config/argvus/data/removable-devices/config.json`, com um `theme.css` correspondente no mesmo diretório. Esse stylesheet também é projetado em `~/.config/argvus/data/generated/removable-devices/theme.css` a partir do tema, accent e modo canônicos, com fallback de família/variante para temas sem um asset dedicado, então uma troca de tema mantém o módulo consistente sem nenhum passo manual. Os caminhos antigos do componente são apenas fontes de migração; a configuração do usuário sobrescreve os padrões do sistema.

Configurações úteis incluem ocultar o módulo quando não há dispositivos (`hide_when_empty`), mostrar nomes e capacidades, ordenar dispositivos, controlar notificações, escolher o backend do menu e definir ícones de estado. Os temas empacotados ficam em `/usr/share/argvus/removable-devices/config/themes/`; eles são assets de implementação e não são as sete famílias de temas de aparência do ARGVUS.
