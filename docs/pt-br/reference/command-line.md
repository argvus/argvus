---
title: Linha de comando
description: O comando argvus, seus subcomandos, as flags mantidas para atalhos e onde cada um roda.
---

`/usr/bin/argvus` é o ponto de entrada público do ARGVUS. É um único programa Rust, compilado a partir do crate `argvus-cli`. Cada subcomando roda dentro desse programa, roda um executável auxiliar do diretório de comandos, ou roda um script empacotado.

Execute `argvus --help` para ver a lista da sua versão instalada. Os subcomandos são escritos sem o `--` inicial. As formas com `--` existem só onde um atalho do Hyprland ou uma entrada de aplicativo ainda as usa.

## Onde cada comando roda

- **Em processo:** o comando é uma biblioteca do workspace e roda dentro do `argvus`. Não existe executável separado. São `config`, `i18n`, `keys`, `gpg`, `projects`, `terminal`, `system-monitor`, `network`, `bluetooth`, `widget-telemetry`, `tui-terminal` e `setup`.
- **Auxiliar:** um executável separado em `/usr/lib/argvus/commands/`, instalado pelo pacote `argvus-gtk`. São `control-center` (aplicativo de terminal) e os aplicativos GTK 4 `taskbar-calendar` e `removable-devices`.
- **Executável privado:** um executável que outro pacote instala em `/usr/lib/argvus/commands/`. O `argvus` apenas o inicia. Incluem `session`, `start`, `tty`, `sessionctl`, `launcher`, `snippets`, `ssh`, `git`, `app-profiles-yazi-keymap`, `notifications`, `displayctl`, `firewall`, `accounts`, `taskbar-actions` e os pontos de entrada `greeter-*`.
- **Script:** um script de shell empacotado, executado com `/bin/sh`. São os comandos de script do Hyprland listados abaixo.

`ARGVUS_COMMANDS_DIR` troca o diretório de comandos, para testes e verificações com `DESTDIR`. Um executável privado que não for encontrado ali é procurado no `PATH`.

## Subcomandos

| Comando | Roda | Observações |
| --- | --- | --- |
| `argvus config ...` | Em processo | Modelo de configuração e ferramentas. Também usado por `theme-switch` e `effects-toggle`. |
| `argvus i18n ...` | Em processo | Traduções: `get DOMAIN KEY [nome=valor...]`, `locale`, `list`, `validate [LOCALE]`. |
| `argvus keys ...` | Em processo | Gerenciador de chaves SSH. `gpg` é um atalho para os subcomandos GPG. |
| `argvus gpg ...` | Em processo | Chaves GPG: `list`, `status`, `generate`, `export-public`, `export-private`, `import`, `import-private`, `delete`, `edit`, `keyserver`. |
| `argvus projects ...` | Em processo | Seletor de projetos. |
| `argvus terminal ...` | Em processo | O perfil de terminal do ARGVUS. |
| `argvus system-monitor ...` | Em processo | O monitor do sistema. |
| `argvus network ...` | Em processo | Estado e controles do NetworkManager (`networkctl` continua como alias). |
| `argvus bluetooth ...` | Em processo | Estado e controles do Bluetooth (`bluetoothctl` continua como alias). |
| `argvus widget-telemetry block ...` | Em processo | O bloco de telemetria da Waybar. É o caminho quente da barra. |
| `argvus widget-telemetry toggle ...` | Em processo | Liga e desliga os blocos de telemetria, e a ordem deles. |
| `argvus game-snake` | Executável privado | O jogo Snake no terminal, do projeto independente `argvus-game-snake`, um pacote opcional. Fora de um terminal, abre um. |
| `argvus control-center [página]` | Auxiliar (aplicativo de terminal) | Abre o Centro de Controle, ou foca a janela que já está aberta. |
| `argvus taskbar-calendar [args]` | Auxiliar GTK 4 | O padrão é `toggle`. |
| `argvus removable-devices [args]` | Auxiliar GTK 4 | O padrão é `menu`. |
| `argvus loading-theme ...` | `splash` empacotado em `/usr/lib/argvus/loading-theme/splash` | O overlay de carregamento da sessão. |
| `argvus session`, `start`, `tty`, `sessionctl` | Executáveis privados | Pontos de entrada da sessão e o controlador da sessão. |
| `argvus greeter`, `greeter-session`, `greeter-tui`, `greeter-setup` | Executáveis privados | Pontos de entrada do greeter. |
| `argvus accounts`, `firewall` | Executáveis privados | Ferramentas de contas e de firewall. |
| `argvus launcher`, `snippets`, `ssh`, `git` | Executáveis privados | Ferramentas de lançador e seletor. |
| `argvus tui-terminal` | Em processo | Wrapper de terminal para aplicativos TUI. |
| `app-profiles-yazi-keymap` | Executável privado | Auxiliar de keymap. |
| `argvus notifications`, `displayctl`, `taskbar-actions` | Executáveis privados | Notificações, controles de display e ações da barra. |
| `argvus setup [opções]` | Em processo | Copia os padrões empacotados para a configuração do usuário. Veja abaixo. |

Os argumentos são repassados sem alteração, inclusive espaços e valores vazios. Entrada padrão, saída, erro, sinais e código de saída pertencem ao comando que roda. Um nome desconhecido falha com uma dica para executar `argvus --help`.

## Flags e aliases

Esses nomes existem para que atalhos do Hyprland, entradas de aplicativos e scripts antigos continuem funcionando. Prefira a forma de subcomando em código novo.

| Flag ou alias | Equivalente |
| --- | --- |
| `--control-center`, `--about`, `--default-apps` | `control-center` (com as páginas `about` e `apps`) |
| página `--displays`, `--monitors` | `control-center displays` |
| `--network-interfaces` | `control-center network interfaces` |
| `--system-monitor`, `--terminal`, `--game-snake` | `system-monitor`, `terminal`, `game-snake` |
| `--calendar`, `--removable-devices`, `--storage` | `taskbar-calendar`, `removable-devices` |
| `--spf`, `--superfile` | o gerenciador de arquivos (`spf`) |
| `--yazi`, `--yazy` | o navegador de arquivos (`yazi`) |
| `--setup` | `setup` |
| `--start-desktop` | inicia `tty` (nenhum argumento é repassado) |
| `--version`, `-V`, `version` | imprime `ARGVUS <versão>` |
| `help`, `--help`, `-h` | imprime o uso |
| `networkctl`, `bluetoothctl` | `network`, `bluetooth` |
| `widget-telemetry-toggle`, `widget-telemetry-block` | `widget-telemetry toggle`, `widget-telemetry block` |
| `theme-switch` | `config theme-apply` |
| `effects-toggle` | `config effects` |

O Centro de Controle aceita flags de página antigas como primeiro argumento e as converte em nomes de página. Exemplos: `--apps` vira `apps`, `--keyboard-layout` vira `locale` e `--hostname` vira `system`.

## Comandos de script do Hyprland

Estes executam scripts empacotados e são usados pela configuração do Hyprland:

`cheatsheets-hypr`, `cheatsheets-launcher`, `scratchpad`, `hypr-screenshot`, `spaces-switch`, `emoji-picker`, `toggle-sidebar`, `weather-location`, `layout-mode-menu`, `brightness-switch`, `toggle-mode`, `idle-timeout`, `keep-awake`, `hypr-power-menu`.

Um script na configuração do usuário substitui o empacotado. A ordem de busca é a mesma de `_config_path` em `hyprland.lua`: a cópia da configuração do usuário, depois a cópia de dados do ARGVUS, depois a cópia de estado, depois a cópia gerada, e só então o arquivo empacotado em `/usr/share/argvus`.

## Exemplos

```sh
argvus control-center displays
argvus taskbar-calendar toggle
argvus widget-telemetry block header system
argvus widget-telemetry toggle status
argvus i18n get widget-telemetry label.device
argvus config get /appearance/theme
argvus network status
argvus bluetooth status
argvus gpg list
```

## Setup

`argvus setup` copia os padrões empacotados de `/usr/share/argvus/<app>` para `~/.config/argvus/data/<app>`, para que você possa personalizá-los. Nunca edita os arquivos empacotados.

```sh
argvus setup --copy <app>          # copia um app empacotado
argvus setup --copy-all            # copia todos os diretórios de apps empacotados
argvus setup --copy <app> --force  # faz backup do destino existente antes
argvus setup --copy <app> --dry-run
```

Um destino que já existe é informado e mantido, a menos que `--force` seja usado. Com `--force`, o diretório existente é renomeado para `<nome>.bak-<timestamp>` antes da cópia.

O app `argvus` é especial: copia o conteúdo do seu diretório empacotado para a raiz da configuração. Como a raiz já existe quando o setup roda, `--copy argvus` precisa de `--force` no código atual.

O setup valida todas as opções antes de alterar qualquer coisa. Nomes de app inválidos (vazios, começando com `.` ou contendo `/`) são recusados.

## Sessão e recarga

`SUPER + SHIFT + R` executa `argvus sessionctl reload`. O controlador da sessão aplica a configuração gerada e reinicia só os componentes de que é responsável. Ele não copia `/usr/share/argvus` para o diretório pessoal, e não substitui um JSON malformado.
