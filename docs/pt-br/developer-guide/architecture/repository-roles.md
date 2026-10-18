---
title: Papéis dos repositórios
description: Responsabilidades dos repositórios de código do ARGVUS depois da consolidação no workspace.
---

Os repositórios são agrupados por responsabilidade, e não pela página de navegação:

- workspace: `argvus` (dispatcher, comandos em processo, auxiliares GTK 4, crates compartilhados, o greeter em `greeter/` e os dados dos comandos);
- sessão e integração: `argvus-session`, `argvus-hyprland`, `argvus-portal`, `argvus-lock`, `argvus-boot-splash`, e o greeter (`argvus-greeter`, gerado a partir de `argvus/greeter`);
- interface fora do workspace: Control Panel, barra (`argvus-taskbar`), lançador e Waybar;
- provedores: display, energia, firewall, notificações;
- recursos e empacotamento: aparência, papéis de parede, fontes, ícones, perfis.

## Consolidados no workspace

Estes repositórios foram unidos em `de/argvus`. A documentação deles agora fica em `docs/` do workspace, e os repositórios são mantidos só como histórico:

| Repositório anterior | Agora | Documentação |
| --- | --- | --- |
| `argvus-config` | `argvus config` | [linha de comando](/pt/docs/reference/command-line/), [arquivos de configuração](/pt/docs/reference/configuration-files/) |
| `argvus-i18n` | `argvus i18n` | [componente de internacionalização](/pt/docs/components/argvus-i18n/) |
| `argvus-keys`, `argvus-projects` | `argvus keys`, `argvus projects` | linha de comando |
| `argvus-terminal` | `argvus terminal` | [componente terminal](/pt/docs/components/argvus-terminal/) |
| `argvus-system-monitor` | `argvus system-monitor` | [componente monitor do sistema](/pt/docs/components/argvus-system-monitor/) |
| `argvus-network` | `argvus network`, `argvus bluetooth` | [componente rede](/pt/docs/components/argvus-network/) |
| `argvus-widget-telemetry` | `argvus widget-telemetry` | linha de comando |
| `argvus-control-center` | `argvus control-center` (aplicativo de terminal) | [componente Centro de Controle](/pt/docs/components/argvus-control-center/) |
| `argvus-taskbar-calendar` | `argvus taskbar-calendar` (auxiliar GTK 4) | [componente calendário da barra](/pt/docs/components/argvus-taskbar-calendar/) |
| `argvus-removable-devices` | `argvus removable-devices` (auxiliar GTK 4) | [componente dispositivos removíveis](/pt/docs/components/argvus-removable-devices/) |
| `argvus-loading-theme` | `argvus loading-theme` (auxiliar `splash`) | linha de comando |
| `argvus-tui-terminal` | `argvus tui-terminal` (crate no workspace) | comandos de linha |
| `argvus-tui`, `argvus-theme` | Crates compartilhados `crates/argvus-tui` e `crates/argvus-theme` | notas do desenvolvedor |
| `argvus-greeter` | Fonte em `greeter/`, pacote `argvus-greeter` | [componente greeter](/docs/components/argvus-greeter/) |
| `argvus-themes` | Aposentado. Cada tema oficial é um pacote `argvus-theme-*`. | [instalando temas](/docs/user-guide/appearance/) |

As notas de desenvolvimento dos componentes consolidados ficam em `docs/internal/<repositório>/`.

Idioma não é a fronteira de responsabilidade: cada componente mantém seus auxiliares de execução junto ao pacote. A regra fixa que atravessa os pacotes: só `argvus-config` escreve abaixo de `data/generated/`. Um componente que precisa chegar a uma aplicação que não lê `config.json` adiciona um adaptador no próprio auxiliar e deixa o orquestrador chamá-lo depois do commit canônico; ele nunca escreve um arquivo gerado. CLIs públicas ficam em `/usr/bin`; auxiliares internos ficam em `/usr/lib/argvus/<componente>` ou no diretório de comandos quando são necessários em execução; dados independentes de arquitetura continuam em `/usr/share/argvus/<componente>`.

Veja a [matriz de pacotes](/pt/docs/reference/package-matrix/) para a fronteira de pacotes instalados.
