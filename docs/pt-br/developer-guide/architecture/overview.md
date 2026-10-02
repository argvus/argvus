---
title: Visão geral da arquitetura
description: Como o ARGVUS forma um desktop integrado.
slug: pt/0.4.0/docs/developer-guide/architecture/overview
---

As camadas de runtime são: dispatcher `argvus`; ciclo de vida em `argvus-session`; compositor e shell em Hyprland, Waybar, taskbar e Control Panel; aplicativos; e provedores de display, rede, energia, aparência, contas e notificações.

O estado compartilhado conecta essas camadas. O `argvus-config` guarda o documento canônico e é o único escritor de `data/generated/`, então uma mudança de tema ou efeitos regenera em uma única transação os arquivos consumidos por vários pacotes.

## Fronteiras de responsabilidade

| Camada | Responsabilidade principal | Exemplos |
| --- | --- | --- |
| Entrypoints e dispatcher | Iniciar aplicativos do desktop e resolver perfis do ARGVUS. | `argvus`, `argvus-app-profiles` |
| Ciclo de vida da sessão | Preparar o ambiente, iniciar o Hyprland, sincronizar o ambiente do serviço do usuário e encerrar a sessão. | `argvus-session`, `argvus-sessionctl` |
| Compositor e shell | Definir o comportamento das janelas e as superfícies visíveis do desktop. | `argvus-hyprland`, `argvus-waybar`, `argvus-taskbar`, `argvus-control-panel` |
| Configuração | Ser dono do `config.json`, do modelo de paths e de todo arquivo consumidor gerado. | `argvus-config` |
| Aplicativos | Fornecer configurações e fluxos específicos do desktop. | Control Center, calendário, launcher, monitor do sistema e dispositivos removíveis |
| Provedores e assets | Implementar operações de domínio e recursos visuais compartilhados. | display, rede, energia, notificações, aparência, fontes e wallpapers |

O pacote `argvus` é o alvo de instalação e o coordenador, mas não é um runtime monolítico. Os pacotes especializados mantêm a responsabilidade por seus binários, configurações e serviços. O Session Manager inicia esses componentes no momento correto da sessão gráfica sem absorver sua implementação.

Essa distinção é importante para diagnóstico: uma falha da sessão pertence primeiro ao `argvus-session`; uma falha da taskbar pertence a `argvus-taskbar`/`argvus-waybar`; e uma falha de tema ou wallpaper gerado pertence ao caminho de configuração da aparência e da sessão.
