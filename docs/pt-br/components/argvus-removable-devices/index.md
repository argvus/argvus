---
title: Dispositivos removíveis
description: Monte e gerencie armazenamento removível.
---

`argvus-removable-devices` monitora UDisks2 e fornece o módulo de armazenamento da ARGVUS Taskbar e o menu dos dispositivos. O menu também está disponível como uma interface separada no estilo Rofi, permitindo usar as mesmas ações pela taskbar ou por um fluxo de launcher.

As ações disponíveis são `open`, `mount`, `unmount`, `eject`, `poweroff`, `lock`, `unlock` e `copy`. Os comandos `list`, `watch` e `menu` expõem respectivamente as interfaces de status, fluxo da taskbar e menu para integrações.

## Menu da taskbar

Com `"mode": "gui"` (o padrão do sistema), um clique no ícone de armazenamento da taskbar abre um menu GTK preso à ARGVUS Taskbar, como o popup do calendário: abaixo da barra quando a taskbar está no topo da tela, e acima dela quando a taskbar está embaixo. Cada linha de dispositivo tem um submenu de ações só com as ações que o estado dele permite.

- **Ponteiro:** passar o ponteiro ou clicar numa linha de dispositivo abre o submenu dela. O submenu continua aberto enquanto o ponteiro atravessa as bordas do menu ou sai dos menus, e só segue outra linha depois que o ponteiro para nela por um instante, então um caminho na diagonal até o submenu não o fecha. Um clique fora dos dois menus fecha o popup.
- **Teclado:** `Up`/`Down` selecionam uma linha, `Right`, `Enter` ou `Space` abrem o submenu (ou executam **Atualizar**/**Sair**), `Enter` ou `Space` executam a ação selecionada, `Left` ou `Escape` voltam para a lista de dispositivos, e `Escape` nela fecha o popup.

**Copiar caminho** coloca o ponto de montagem na área de transferência pelo `copy_command` (padrão `wl-copy`, do `wl-clipboard`), então o caminho continua lá depois que o menu fecha; uma notificação confirma a cópia quando `show_notification` está ativo. **Bloquear** aparece para todo volume criptografado desbloqueado e desmonta o volume antes, quando necessário.

A ação padrão de abrir arquivos segue o gerenciador de arquivos selecionado no ARGVUS Control Center. Assim, as ações de dispositivos removíveis permanecem consistentes com os aplicativos padrão do usuário.

A configuração do sistema é `/etc/argvus/removable-devices/config.json`. A projeção gerenciada por usuário fica em `~/.config/argvus/data/removable-devices/config.json`, com um `theme.css` correspondente no mesmo diretório. Esse stylesheet também é projetado em `~/.config/argvus/data/generated/removable-devices/theme.css` a partir do tema, accent e modo canônicos, com fallback de família/variante para temas sem um asset dedicado, então uma troca de tema mantém o módulo consistente sem nenhum passo manual. Os caminhos antigos do componente são apenas fontes de migração; a configuração do usuário sobrescreve os padrões do sistema.

Configurações úteis incluem ocultar o módulo quando não há dispositivos (`hide_when_empty`), mostrar nomes e capacidades, ordenar dispositivos, controlar notificações, escolher o backend do menu e definir ícones de estado. A folha de estilos dele é renderizada a partir do tema ativo do ARGVUS (`removable-devices/theme.css` do build do tema) e copiada para o `theme.css` do painel a cada troca de tema ou de destaque.
