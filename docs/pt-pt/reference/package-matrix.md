---
title: Matriz de pacotes
description: Responsabilidades dos pacotes do ARGVUS.
---

| Pacote | Responsabilidade |
| --- | --- |
| `argvus` | Pacote mestre: dispatcher, comandos em processo (configuração, traduções, chaves, projetos, terminal, monitor do sistema, rede, Bluetooth, telemetria, terminal TUI, setup), os dados compartilhados e os catálogos de i18n. Todo pacote de componente depende dele. |
| `argvus` (auxiliares) | Auxiliares no mesmo pacote: o Centro de Controle e o Analisador de Uso de Disco (aplicações de terminal), além das aplicações GTK 4 do calendário da barra, dispositivos removíveis, overlay de carregamento, ARGVUS Focus e ARGVUS Clock. |
| `argvus` (greeter) | Greeter de login para o greetd, no mesmo pacote, gerado a partir de `crates/argvus-greeter`. |
| `argvus-desktop` | Perfil do ambiente gráfico e das aplicações de uso diário, exigido pelo pacote mestre `argvus`. Não depende de `argvus`, mantendo a dependência em uma única direção. |
| `argvus-devtools`, `argvus-pentest` | Metapacotes independentes e opcionais para ferramentas de desenvolvimento e auditoria de segurança autorizada. O pacote mestre não instala esses perfis. |
| `argvus-session` | Pontos de entrada da sessão e ciclo de vida do utilizador. |
| `argvus-hyprland` | Configuração do Hyprland. |
| `argvus-control-panel` | Painel de controle em Quickshell. |
| `argvus-taskbar`, `argvus-waybar` | Configuração da Waybar e binário com patch. |
| `argvus-branding` | Logo e recursos da marca. |
| `argvus-appearance`, `argvus-wallpapers`, `argvus-fonts`, `argvus-icons` | Estado de aparência e recursos visuais. |
| `argvus-display`, `argvus-power`, `argvus-notifications` | Provedores do desktop. |
| `argvus-lock`, `argvus-boot-splash` | Bloqueio de ecrã e splash de arranque. |
| `argvus-firewall` | Firewall, com privilégios próprios. |
| `argvus-launcher` | Configuração do lançador e pontos de entrada. |
| `argvus-app-profiles` | Perfis de dados para as aplicações de terminal incluídos. |
| `argvus-portal` | Portais XDG e padrões Wayland/DBus. |
| `argvus-theme-*` | Um pacote opcional por família de tema oficial. |
| `argvus-game-snake`, `argvus-games` | Opcional. O Snake é um projeto independente; `argvus-games` agrupa os jogos opcionais. |

## Pacotes substituídos

O `argvus` declara `replaces`, `conflicts` e `provides` dos pacotes que absorveu, para que uma atualização os remova e seus ficheiros não fiquem órfãos:

`argvus-config`, `argvus-i18n` (com `argvus-language-en-us` e `argvus-language-pt-br`), `argvus-keys`, `argvus-projects`, `argvus-terminal`, `argvus-system-monitor`, `argvus-network`, `argvus-widget-telemetry` e `argvus-du`. O `argvus` também substitui `argvus-control-center`, `argvus-taskbar-calendar`, `argvus-loading-theme`, `argvus-removable-devices` e os antigos pacotes separados `argvus-gtk` e `argvus-greeter`, que ele ainda fornece para que os pacotes que dependem deles continuem resolvidos. O gestor de contas (`argvus accounts`) é compilado a partir de `crates/argvus-accounts` e instalado em `/usr/lib/argvus/commands/accounts`, com a sua regra polkit. O Analisador de Uso de Disco é compilado a partir de `crates/argvus-du` e instalado em `/usr/lib/argvus/commands/argvus-du`.

`/usr/bin/argvus-terminal` continua instalado como launcher de compatibilidade para `argvus terminal`. O terminal padrão do Hyprland, o catálogo do Control Center e as aplicações padrão salvos ainda usam esse comando.

Os pacotes removidos `argvus-pwvucontrol` e `argvus-snappy-switcher` não são fornecidos por nenhum pacote.

Não há ordem de instalação entre os pacotes de componentes ARGVUS: `argvus` é o pacote mestre deles, e cada componente depende dele. Instalar `argvus` também instala o perfil `argvus-desktop`; `argvus-devtools` e `argvus-pentest` continuam opcionais. Estar nesta tabela não significa que um pacote é dono de toda a funcionalidade: por exemplo, a barra combina `argvus-waybar`, `argvus-taskbar`, `argvus-session` e os provedores de domínio.
