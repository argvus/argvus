---
title: Matriz de pacotes
description: Responsabilidades dos pacotes ARGVUS.
slug: pt/0.4.0/docs/reference/package-matrix
---

| Pacote | Responsabilidade |
| --- | --- |
| `argvus` | Dispatcher e ponto de entrada. |
| `argvus-session` | Entrada e ciclo de vida da sessão. |
| `argvus-config` | Configuração lógica canônica do usuário, único escritor de `data/generated/`, e importação/exportação de perfis. |
| `argvus-hyprland` | Configuração Hyprland. |
| `argvus-control-center` | Aplicativo de configurações TUI. |
| `argvus-control-panel` | Painel Quickshell. |
| `argvus-taskbar`, `argvus-waybar` | Configuração e binário Waybar. |
| `argvus-branding` | Assets de logo e wordmark. |
| `argvus-appearance`, `argvus-wallpapers`, `argvus-fonts`, `argvus-icons` | Estado visual e assets. |
| `argvus-network`, `argvus-display`, `argvus-power`, `argvus-notifications` | Provedores do desktop. |
| `argvus-greeter`, `argvus-lock`, `argvus-splash`, `argvus-theme-splash` | Login, lock e startup. |
| `argvus-i18n`, `argvus-tui` | Bibliotecas e catálogos compartilhados. |

## Fronteiras adicionais de pacotes

| Pacote | Responsabilidade |
| --- | --- |
| `argvus-accounts` | Metadados de contas locais e avatares. |
| `argvus-taskbar-calendar` | Popup de calendário e integração de eventos. |
| `argvus-games` | Metapacote dos jogos oficiais do ARGVUS; depende de `argvus-game-snake`. |
| `argvus-game-snake` | Jogo Snake retro no terminal e ranking local. |
| `argvus-removable-devices` | Ações de armazenamento UDisks2 e integração com taskbar/menu. |
| `argvus-launcher` | Configuração e entrypoints do launcher. |
| `argvus-terminal`, `argvus-system-monitor` | Perfis de terminal e monitor do sistema. |
| `argvus-firewall` | Serviço de firewall do sistema e integração com o Control Center. |

O pacote `argvus` depende dos pacotes de componentes para formar o desktop completo. A presença de um pacote nesta tabela não significa que ele seja dono de toda a funcionalidade: por exemplo, a taskbar combina `argvus-waybar`, `argvus-taskbar`, `argvus-session` e provedores de domínio.
