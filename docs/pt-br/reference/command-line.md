---
title: Linha de comando
description: Comandos públicos e pacotes responsáveis.
slug: pt/0.4.0/docs/reference/command-line
---

O `/usr/bin/argvus` fornece `--setup`, `--system-monitor`, `--control-center`, `--terminal`, `--spf`, `--yazi`, `--about`, `--calendar`, `--game-snake`, `--default-apps`, `--removable-devices`, `--displays`, `--network-interfaces` e `--version`. Os mesmos nomes funcionam sem `--`.

Outros pontos públicos incluem `argvus-session`, `argvus-start`, `argvus-sessionctl`, `argvus-tty`, `argvus-control-center`, `argvus-taskbar-calendar`, `argvus-accounts`, `argvus-displayctl`, `argvus-networkctl`, `argvus-bluetoothctl`, `argvus-notifications`, `argvus-firewall` e `argvus-widget-telemetry-toggle`.

Use `--help` em cada comando. Não existem binários separados `argvus-about`, `argvus-setup` ou `argvus-default-apps`.

## Exemplos do dispatcher

O dispatcher é o entrypoint voltado ao usuário para os aplicativos integrados:

| Comando | Ação |
| --- | --- |
| `argvus --control-center` | Abre o ARGVUS Control Center. |
| `argvus --default-apps` | Abre a página Apps do Control Center (rota de compatibilidade). |
| `argvus --calendar` | Alterna o calendário da taskbar. |
| `argvus --game-snake` | Abre o jogo Snake retro no terminal. |
| `argvus --removable-devices` | Abre o menu de dispositivos removíveis. |
| `argvus --system-monitor` | Abre o monitor do sistema configurado. |
| `argvus --terminal` | Abre o perfil de terminal do ARGVUS. |
| `argvus --spf` / `argvus --yazi` | Abre o perfil do gerenciador de arquivos correspondente, lendo `data/generated/superfile/` e `data/generated/yazi/`. |
| `argvus --displays` | Abre a página Displays do Control Center. |
| `argvus --network-interfaces` | Abre os controles de interfaces de rede no Control Center. |

O dispatcher também aceita os nomes sem o prefixo `--`, como `argvus control-center` ou `argvus calendar`. Execute `argvus --help` para consultar a lista completa da versão instalada.

## Comandos de sessão e domínios

```sh
argvus-sessionctl status
argvus-sessionctl reload
argvus-sessionctl logs
argvus-displayctl --apply
argvus-networkctl status
argvus-bluetoothctl available
argvus-widget-telemetry-toggle status
```

O `argvus-sessionctl` controla a sessão do usuário do ARGVUS e pode reiniciar componentes pertencentes a ela. Os comandos de display, rede e Bluetooth delegam aos seus provedores correspondentes. Eles não substituem os serviços de sistema subjacentes.

## Setup e personalização

`SUPER + SHIFT + R` executa `argvus-sessionctl reload`. Quando um comando anterior já confirmou uma generation, o reload a aplica a partir do manifesto de projeção e não projeta de novo. Se o perfil do ARGVUS
ou o `config.json` estiver ausente, o reload materializa um perfil canônico
novo, recria as projeções e aplica as mudanças ao runtime. Ele não copia
`/usr/share/argvus` para a pasta pessoal e não substitui silenciosamente um
JSON existente malformado.

`argvus --setup` é uma ferramenta separada e opcional para overrides avançados.
Ela não é um mecanismo de recovery. O manifesto atual oferece os overrides de
TUI `foot-tui` e `kitty-tui`:

```sh
argvus --setup --copy foot-tui
argvus --setup --copy kitty-tui
argvus --setup --copy-all
argvus --setup --copy foot-tui --force
argvus --setup --copy-all --dry-run
```

`--copy-all` copia somente essas entradas explícitas; não enumera todos os
diretórios de primeiro nível em `/usr/share/argvus` e nunca copia scripts,
serviços, assets, arquivos gerados ou `config.json`. Todas as fontes e destinos
são resolvidos antes de qualquer mutação. `--force` faz backup somente do
destino afetado, enquanto `--dry-run` não altera o filesystem. Não existe um
binário separado `argvus-setup`.
