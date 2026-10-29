---
title: Matriz de pacotes
description: Responsabilidades dos pacotes do ARGVUS.
---

| Pacote | Responsabilidade |
| --- | --- |
| `argvus` | Pacote mestre: dispatcher, comandos em processo (configuração, traduções, chaves, projetos, terminal, monitor do sistema, rede, Bluetooth, telemetria, terminal TUI, setup), os dados compartilhados e os catálogos de i18n. Todo pacote de componente depende dele. |
| `argvus` (auxiliares) | Auxiliares no mesmo pacote: o Centro de Controle (aplicativo de terminal) e os aplicativos GTK 4 do calendário da barra, dispositivos removíveis, overlay de carregamento, ARGVUS Focus e ARGVUS Clock. |
| `argvus` (greeter) | Greeter de login para o greetd, no mesmo pacote, gerado a partir de `crates/argvus-greeter`. |
| `argvus-session` | Pontos de entrada da sessão e ciclo de vida do usuário. |
| `argvus-hyprland` | Configuração do Hyprland. |
| `argvus-control-panel` | Painel de controle em Quickshell. |
| `argvus-taskbar`, `argvus-waybar` | Configuração da Waybar e binário com patch. |
| `argvus-branding` | Logo e recursos da marca. |
| `argvus-appearance`, `argvus-wallpapers`, `argvus-fonts`, `argvus-icons` | Estado de aparência e recursos visuais. |
| `argvus-display`, `argvus-power`, `argvus-notifications` | Provedores do desktop. |
| `argvus-lock`, `argvus-boot-splash` | Bloqueio de tela e splash de inicialização. |
| `argvus-firewall` | Firewall, com privilégios próprios. |
| `argvus-launcher` | Configuração do lançador e pontos de entrada. |
| `argvus-app-profiles` | Perfis de dados para os aplicativos de terminal incluídos. |
| `argvus-portal` | Portais XDG e padrões Wayland/DBus. |
| `argvus-theme-*` | Um pacote opcional por família de tema oficial. |
| `argvus-game-snake`, `argvus-games` | Opcional. O Snake é um projeto independente; `argvus-games` agrupa os jogos opcionais. |

## Pacotes substituídos

O `argvus` declara `replaces`, `conflicts` e `provides` dos pacotes que absorveu, para que uma atualização os remova e seus arquivos não fiquem órfãos:

`argvus-config`, `argvus-i18n` (com `argvus-language-en-us` e `argvus-language-pt-br`), `argvus-keys`, `argvus-projects`, `argvus-terminal`, `argvus-system-monitor`, `argvus-network`, `argvus-widget-telemetry`. O `argvus` também substitui `argvus-control-center`, `argvus-taskbar-calendar`, `argvus-loading-theme`, `argvus-removable-devices` e os antigos pacotes separados `argvus-gtk` e `argvus-greeter`, que ele ainda fornece para que os pacotes que dependem deles continuem resolvidos. O gerenciador de contas (`argvus accounts`) é compilado a partir de `crates/argvus-accounts` e instalado em `/usr/lib/argvus/commands/accounts`, com a sua regra polkit.

`/usr/bin/argvus-terminal` continua instalado como launcher de compatibilidade para `argvus terminal`. O terminal padrão do Hyprland, o catálogo do Control Center e os aplicativos padrão salvos ainda usam esse comando.

Os pacotes removidos `argvus-pwvucontrol` e `argvus-snappy-switcher` não são fornecidos por nenhum pacote.

Não há ordem de instalação: `argvus` é o pacote mestre, e todos os outros dependem dele. Estar nesta tabela não significa que um pacote é dono de toda a funcionalidade: por exemplo, a barra combina `argvus-waybar`, `argvus-taskbar`, `argvus-session` e os provedores de domínio.
