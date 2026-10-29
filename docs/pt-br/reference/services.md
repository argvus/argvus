---
title: Serviços
description: Serviços systemd do ARGVUS.
---

O target principal é `argvus-session.target`. Serviços comuns incluem `argvus-taskbar.service`, `argvus-control-panel.service`, `argvus-widget-telemetry.service`, `argvus-dunst.service`, `argvus-wallpaper.service`, `argvus-hypridle.service` e `argvus-session-loading.service`.

O firewall usa `argvus-firewall.service`; o calendário instala `argvus-taskbar-calendar.service` como unit de usuário, e `argvus-clock.service` (desativada por padrão) inicia o ARGVUS Clock oculto após o login quando **Iniciar com a sessão** está ligado.

```sh
systemctl --user status argvus-session.target
systemctl --user list-units 'argvus-*.service'
```

## Units pertencentes à sessão

O pacote de sessão instala estas units de usuário em `/usr/lib/systemd/user` (o payload de origem fica na árvore de configuração compartilhada do pacote de sessão):

| Unit | Função |
| --- | --- |
| `argvus-session.target` | Limite de vida da sessão gráfica do ARGVUS. |
| `argvus-session-prepare.service` | Preparação one-shot e configuração gerada no início. |
| `argvus-control-panel.service` | Control Panel Quickshell. |
| `argvus-taskbar.service` | Taskbar principal do ARGVUS. |
| `argvus-widget-telemetry.service` | Superfície opcional de telemetria/sysinfo. |
| `argvus-config.service` | Projeta a configuração canônica (`argvus config project`); o reload executa `argvus sessionctl reload-config-transaction`. |
| `argvus-session-reload.service` | Reload do login (`argvus sessionctl reload-desktop --login`): recarrega o Hyprland só quando a projeção o afetou. |
| `argvus-ssh-agent.service` | Agente SSH da sessão. |
| `argvus-secret-service.service` | Inicia o serviço de segredos por `argvus sessionctl start-secret-service`. |
| `argvus-wallpaper.service` | Backend do wallpaper ativo. |
| `argvus-dunst.service` | Daemon de notificações Dunst. |
| `argvus-hypridle.service` | Política de idle e lock. |
| `argvus-clipboard-text.service` / `argvus-clipboard-image.service` | Watchers do histórico de clipboard. |
| `argvus-keyboard-layout.service` | Notificações de layout do teclado. |
| `argvus-polkit.service` | Integração do agente PolicyKit. |
| `argvus-session-loading.service` | Overlay de carregamento da troca de compositor até os componentes do desktop iniciarem. |

`argvus-blueman-applet.service` também é fornecido pelo pacote de sessão. Algumas units são opcionais ou iniciadas condicionalmente; nem todas ficam ativas em toda sessão.

Para inspecionar a definição efetiva de uma unit, em vez de somente o checkout da fonte:

```sh
systemctl --user cat argvus-session.target
systemctl --user status argvus-session.target
journalctl --user -u argvus-session.target -u 'argvus-*.service'
```

O serviço de sistema `argvus-firewall.service` pertence ao `argvus-firewall`. O pacote do calendário fornece separadamente `argvus-taskbar-calendar.service` como uma unit de usuário. `argvus-clock.service` é uma unit de usuário do pacote `argvus`, `WantedBy=graphical-session.target`, ativada nas configurações do ARGVUS Clock.

## Comandos de gerenciamento de sessão

O comando `argvus sessionctl` gerencia o ciclo de vida da sessão e pode reiniciar componentes específicos sem fazer logout:

### Ciclo de vida da sessão

```sh
argvus sessionctl start               # Iniciar uma nova sessão
argvus sessionctl stop                # Parar a sessão atual
argvus sessionctl restart             # Reiniciar a sessão inteira
argvus sessionctl reload              # Recarregar configuração e serviços ativos
argvus sessionctl status              # Mostrar status da sessão
argvus sessionctl logs                # Exibir logs da sessão
```

### Reinicializações de componentes específicos

Reinicie serviços individuais sem afetar a sessão inteira:

```sh
# Superfícies de desktop
argvus sessionctl restart waybar
argvus sessionctl restart wallpaper
argvus sessionctl restart shell

# Notificações e seletor
argvus sessionctl restart dunst

# Clipboard e entrada
argvus sessionctl restart clipboard keyboard-layout

# Applet Bluetooth (opcional)
argvus sessionctl restart blueman-applet
```

### Sincronização de ambiente

Importe variáveis de ambiente para a sessão:

```sh
argvus sessionctl import-environment
```

Isso sincroniza variáveis de Wayland, Hyprland, Qt, cursor e DBus entre o shell e os serviços de usuário do systemd.

## Referência de units de serviço

Unidades adicionais de usuário podem ser gerenciadas diretamente com `systemctl --user`:

```sh
systemctl --user status argvus-session.target
systemctl --user start argvus-session.target
systemctl --user stop argvus-session.target
systemctl --user restart <unit>

# Visualizar logs para um serviço específico
journalctl --user -u argvus-wallpaper.service -b -f
```

Os logs do bootstrap inicial de sessão são armazenados em:

```
${XDG_STATE_HOME:-$HOME/.local/state}/argvus/session.log
```
