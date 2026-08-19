---
title: Apps oficiais
description: Comandos e aplicativos mantidos pelo ARGVUS.
---

Aplicativos e comandos oficiais do ARGVUS usam o prefixo `argvus-`. Eles são mantidos como parte do desktop ARGVUS, não como utilitários genéricos de terceiros.

## Comandos do desktop

O pacote `argvus` entrega o `argvus-setup`:

| Comando | Função |
| --- | --- |
| `argvus-setup` | Helper opcional que copia os padrões empacotados de `/usr/share/argvus` para `$XDG_CONFIG_HOME/<app>` apenas quando uma personalização completa do usuário for desejada. |

Os launchers de sessão são de propriedade do pacote separado [`argvus-session`](https://github.com/argvus/argvus-session):

| Comando | Função |
| --- | --- |
| `argvus-session` | Wrapper de display manager que prepara o ambiente e inicia a sessão. |
| `argvus-start` | Inicia o runtime do ARGVUS e a sessão Hyprland. |
| `argvus-tty` | Launcher manual por TTY para sistemas sem display manager. |

## ARGVUS Calendar

`argvus-calendar` é um aplicativo oficial em Rust mantido em [`argvus/argvus-calendar`](https://github.com/argvus/argvus-calendar). Ele renderiza um popup de calendário GTK4 nativo como uma superfície layer-shell, aberto pelo módulo de data da Waybar, com armazenamento SQLite, importação/exportação ICS e lembretes configuráveis.

Layout instalado:

```text
/usr/bin/argvus-calendar
/etc/argvus-calendar/config.toml
/etc/argvus-calendar/style.css
/etc/argvus-calendar/theme.css
/etc/argvus-calendar/themes/
/usr/lib/systemd/user/argvus-calendar.service
```

O pacote `argvus` depende de `argvus-calendar`, e o pacote de calendário é dono do binário, da configuração e dos temas.

Veja [Calendário](./calendar/) para uso, configuração e comandos.

## ARGVUS Storage

`argvus-storage` é um aplicativo oficial em Rust mantido em [`argvus/argvus-storage`](https://github.com/argvus/argvus-storage). Ele observa armazenamento removível via UDisks2 e alimenta o módulo `custom/storage` da Waybar.

Layout instalado:

```text
/usr/bin/argvus-storage
/etc/argvus-storage/config.json
/etc/argvus-storage/theme.css
/etc/argvus-storage/themes/
/usr/share/licenses/argvus-storage/LICENSE
```

Overrides do usuário ficam em:

```text
$XDG_CONFIG_HOME/argvus-storage/config.json
$XDG_CONFIG_HOME/argvus-storage/theme.css
$XDG_CONFIG_HOME/argvus-storage/themes/
```

O ambiente ARGVUS não carrega mais esses arquivos dentro dele. O pacote `argvus` depende de `argvus-storage`, e o pacote de storage é dono do binário, da configuração padrão e dos temas.

Veja [Armazenamento Removível](./storage/) para uso dos comandos e chaves de configuração.

## ARGVUS Appearance

[`argvus-appearance`](https://github.com/argvus/argvus-appearance) entrega os ativos visuais compartilhados usados pelo desktop:

```text
/usr/share/backgrounds/argvus/
/usr/share/fonts/
```

Manter esses ativos em um pacote separado evita duplicar wallpapers e fontes dentro do pacote de configuração `argvus`.

## Helpers internos de shell

O ambiente também inclui helpers em `config/argvus/sh/`. Eles fazem parte da implementação do desktop e são chamados por atalhos, módulos da Waybar e pela sidebar.

Helpers comuns:

- `theme-switch.sh` para aplicar uma família de tema em todos os componentes.
- `accent-switch.sh` para mudar a cor de destaque.
- `spaces-switch.sh` para gaps e modos de espaçamento.
- `brightness-switch.sh` para ações de brilho.
- `weather-location.sh` para o módulo de clima.
- `toggle-mode.sh` para alternância de modo.

Esses helpers não são empacotados como comandos públicos `/usr/bin/argvus-*`. Eles são instalados como arquivos de suporte em `/usr/share/argvus` e rodam a partir dos defaults empacotados durante uma sessão normal. Use `argvus-setup --copy argvus` apenas quando quiser explicitamente uma cópia completa de propriedade do usuário para personalização.
