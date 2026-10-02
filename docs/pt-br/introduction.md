---
title: Introdução
description: O que é o ARGVUS, como o desktop funciona e onde ficam seus componentes.
slug: pt/0.4.0/docs/introduction
---

ARGVUS é um desktop modular baseado em Hyprland e Wayland para Arch Linux. Ele é um conjunto integrado de pacotes, não um aplicativo monolítico: sessão, compositor, superfícies do shell, configurações, provedores e assets visuais trabalham juntos por meio de estado compartilhado e contratos de runtime.

Esta página é o mapa da documentação. Ela explica o desktop em alto nível; as páginas relacionadas contêm os detalhes de uso e implementação.

## Como o ARGVUS funciona

A sessão normal segue este fluxo:

```text
login gráfico ou TTY
        ↓
argvus-session → argvus-start
        ↓
Hyprland + argvus-hyprland
        ↓
argvus-session.target (systemd --user)
        ↓
taskbar · control panel · notificações · wallpaper · idle · clipboard
```

`argvus-session` possui o ciclo de vida e a configuração do ambiente. Ele não implementa todas as funcionalidades: cada componente possui seus comandos, configurações e payloads de serviço.

Para instalação e inicialização, veja [Instalação](/docs/getting-started/installation/) e [Sessões](/docs/user-guide/sessions/). Para ownership dos serviços, veja o [ciclo de vida](/docs/developer-guide/architecture/runtime-lifecycle/) do desenvolvedor.

## Camadas principais

### Entrypoints e infraestrutura

O pacote `argvus` fornece o dispatcher `/usr/bin/argvus` e o ponto de entrada principal. `argvus-session` fornece entradas gráfica e TTY, o target de usuário e `argvus-sessionctl`. TUI e localização compartilhadas vêm de `argvus-tui` e `argvus-i18n`.

### Compositor e shell do desktop

`argvus-hyprland` fornece configuração e keybindings Hyprland. `argvus-waybar` fornece o binário Waybar corrigido, enquanto `argvus-taskbar` possui configuração e ações da taskbar. `argvus-control-panel` fornece o painel Quickshell, e `argvus-widget-telemetry` fornece widgets opcionais.

Veja [Desktop](/docs/user-guide/desktop/) para uso e [superfícies do shell](/docs/developer-guide/subsystems/shell-surfaces/) para ownership de implementação.

### Configurações e aplicativos

`argvus-control-center` é o aplicativo de configurações orientado ao teclado. Calendário, monitor do sistema, terminal, launcher e ferramentas de dispositivos removíveis são aplicativos separados integrados ao shell e ao dispatcher.

Veja [Aplicativos](/docs/user-guide/applications/) e a [referência de comandos](/docs/reference/command-line/).

### Provedores e integrações

Display, rede, Bluetooth, energia, notificações, firewall e armazenamento removível são fornecidos por projetos específicos. Control Center e Control Panel consomem esses provedores; eles não os substituem.

Login, lock e visuais de inicialização também são separados: `argvus-greeter` trata greetd, `argvus-lock` trata Hyprlock, `argvus-theme-splash` trata o overlay de carregamento da sessão e `argvus-splash` trata o tema Plymouth de boot.

Veja [Hardware](/docs/user-guide/hardware/), [Privacidade e segurança](/docs/user-guide/privacy-and-security/) e a [referência de componentes](/docs/developer-guide/).

## Estado compartilhado de aparência

Preferências de tema, acento, wallpaper, fontes e efeitos ficam como estado lógico em `$XDG_CONFIG_HOME/argvus` (normalmente `~/.config/argvus`). O `argvus-config` é o único componente que escreve `data/generated/`, onde projeta arquivos consumidor para Hyprland, GTK, Qt6ct, Waybar, Quickshell, Rofi, Dunst, notificações, lock screen, Yazi, Superfile e terminais. Os helpers de aparência confirmam a mudança de estado e então reconciliam aplicativos externos; eles nunca escrevem a árvore generated.

O diretório gerado não é a fonte de verdade. Veja [Aparência](/docs/user-guide/appearance/) e [configuração e estado](/docs/developer-guide/architecture/configuration-and-state/).

O sistema visual também controla o layout. Sticky é compacto e quadrado; Float usa gaps maiores, cantos arredondados e margens maiores ao redor da taskbar e do shell. A taskbar usa a borda superior por padrão, mas permite posicionamento no topo ou embaixo pelo estado compartilhado. Os efeitos controlam transparência, blur, sombras e animações do compositor e das superfícies do desktop.

Veja a [tabela de temas](/docs/user-guide/appearance/) para famílias e cores, [efeitos](/docs/argvus-appearance/user-guide/effects/) para o contrato compartilhado e [taskbar](/docs/argvus-taskbar/user-guide/taskbar/) para o posicionamento da barra e das janelas.

## O que é — e o que não é — o ARGVUS

O ARGVUS é a camada de ambiente desktop sobre o Arch Linux. O Arch Linux fornece o sistema operacional e os pacotes; o Hyprland fornece a composição Wayland; o ARGVUS coordena a sessão gráfica, as superfícies do shell, as configurações, a aparência, os serviços do desktop e os aplicativos integrados.

Por isso, ele é mais do que uma configuração do Hyprland ou uma coleção de dotfiles pessoais. O ARGVUS define um entrypoint de sessão suportado, gerencia o ciclo de vida dos serviços do desktop, fornece um modelo comum de configurações e aparência e conecta componentes especializados de rede, displays, energia, notificações, contas e armazenamento removível.

O ARGVUS não precisa substituir todos os aplicativos. Ele integra aplicativos upstream selecionados quando essa é a opção mais confiável, mantendo coerentes o comportamento e a configuração do desktop.

Para conhecer as fronteiras de implementação e a responsabilidade dos pacotes, consulte a [visão geral da arquitetura](/docs/developer-guide/architecture/overview/) e os [papéis dos repositórios](/docs/developer-guide/architecture/repository-roles/) para desenvolvedores.

## Continue por objetivo

* [Ler as perguntas frequentes](/docs/user-guide/faq/).
* [Instalar o ARGVUS](/docs/getting-started/installation/).
* [Iniciar uma sessão gráfica ou TTY](/docs/user-guide/sessions/).
* [Configurar temas e wallpapers](/docs/user-guide/appearance/).
* [Usar taskbar e Control Panel](/docs/user-guide/desktop/).
* [Entender comandos, caminhos e serviços](/docs/reference/).
* [Estudar a arquitetura](/docs/developer-guide/architecture/overview/).
