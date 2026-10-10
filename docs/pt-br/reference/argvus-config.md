---
title: Configuração do ARGVUS
description: O contrato JSON lógico de configuração usado pelo ARGVUS.
slug: pt/0.4.0/docs/reference/argvus-config
---

O ARGVUS mantém as preferências de usuário gerenciadas em `$XDG_CONFIG_HOME/argvus/config.json` (normalmente `~/.config/argvus/config.json`). Esse documento é o modelo lógico canônico das preferências administradas pelo ARGVUS, e o `argvus-config` é o único componente que escreve arquivos em `data/generated/`. Arquivos nativos e gerados continuam como projeções de compatibilidade ou overrides específicos de cada aplicativo.

## Guia do usuário: Início rápido

Para a maioria dos usuários, a configuração acontece através do Control Center do ARGVUS ou desta ferramenta de linha de comando:

```sh
# Visualize o local da sua configuração e valide
argvus-config path
argvus-config validate

# Obtenha um valor específico
argvus-config get /appearance/theme
argvus-config get /appearance/theme --raw    # Sem formatação JSON

# Defina um valor (deve ser JSON válido)
argvus-config set /appearance/theme '"gruvbox-dark"'

# Aplique mudanças para recarregar a sessão
argvus-config apply
```

Para administradores e configuração avançada, continue lendo abaixo.

A raiz do usuário contém somente o documento canônico e os dados gerenciados:

```text
~/.config/argvus/
├── config.json
└── data/
```

O arquivo é escrito atomicamente, fica privado para o usuário e é protegido por `.config.lock`, fora da árvore `data/` que pode ser trocada durante uma projeção. Uma versão anterior fica em `data/backups/config.json.bak`.

## Linha de comando

O comando `argvus-config` é instalado pelo pacote `argvus-config`:

```sh
argvus-config path
argvus-config validate
argvus-config ensure
argvus-config migrate
argvus-config status
argvus-config explain /layout/gaps_out
argvus-config get /appearance/theme --effective --raw
argvus-config set /appearance/accent '\"#61AFEF\"'
argvus-config patch appearance-patch.json
argvus-config apply-theme universe --variant sticky --accent '#EEEEEE' --gtk-mode dark
argvus-config apply-theme universe-float --variant float --accent '#EEEEEE' --gtk-mode dark
argvus-config theme-manifest universe-float
argvus-config unset /appearance/accent
argvus-config apply
argvus-config rebuild
argvus-config project
argvus-config project --force
```

`ensure` materializa um novo `config.json` com os defaults do schema ou completa
campos ausentes sem substituir valores explícitos. Ele falha diante de JSON
malformado. `migrate` importa o estado legado de aparência sem substituir
valores já presentes no `config.json`. Antes de aplicar um reload, o
`argvus-sessionctl` executa `recover`, `ensure`, `migrate` e `project`:

```sh
argvus-sessionctl apply-config
```

`apply` e `rebuild` são aliases para reconstruir todas as projeções a partir do
`config.json`; `project` mantém a forma histórica e aceita `--force`. A geração
ocorre em staging e a árvore `data/` só é trocada depois que todos os
projectors terminam. Assim, apagar `data/generated/` e executar `argvus-config
apply` também delega o reload pós-commit ao `argvus-sessionctl`; use
`ARGVUS_NO_RUNTIME=1` durante preparação da sessão ou testes isolados. Esse
rebuild é reproduzível, sem publicar uma sequência parcial de Hyprland ou Taskbar. O comando registra hashes derivados por seção e a decisão
de runtime em `$XDG_STATE_HOME/argvus/config-projection.json` (normalmente
`~/.local/state/argvus/config-projection.json`). Nem o manifesto nem os arquivos
gerados são fontes de verdade. `apply` faz o reload pós-commit por meio do
`argvus-sessionctl`; `project` continua sendo apenas projeção por compatibilidade.

Toda mutation persistente segue a mesma regra: se o documento canônico mudou,
o CLI chama uma única vez o `argvus-sessionctl reload` depois do commit. O
resultado fica registrado em `data/internal/last-reload.json`.

Por isso `argvus-sessionctl reload` é uma etapa de *apply*, e não de projeção.
Quando o manifesto recebido indica trabalho pendente, ele restaura os
consumidores descritos pelo plano e não chama `argvus-config project` de novo;
reprojetar reconstruiria uma árvore idêntica e descartaria o plano que a mutação
acabou de produzir. A migração e a projeção só acontecem quando não existe um
plano commitado, por exemplo durante o startup da sessão.

Cada configuração efetiva possui uma generation operacional. Ela fica em
`data/internal/generation.json` e no manifesto em
`$XDG_STATE_HOME/argvus/config-projection.json`; não faz parte dos dotfiles.
Reaplicar uma configuração efetiva inalterada preserva a generation. `status`
compara hash, generation e todos os arquivos gerados; `apply` corrige uma
projeção ausente ou stale.

`explain /path` usa diretamente o Resolver e mostra valor efetivo, origem, tema
e variante. Ele não interpreta arquivos gerados nem cria outra regra de precedência.

As projeções de tema e efeitos são implementadas pelo próprio `argvus-config`.
`theme-switch.sh`, `accent-switch.sh` e `effects-toggle.sh` não projetam: eles
validam, travam, exibem o splash, confirmam através desta CLI e então reconciliam
consumidores externos que não conseguem ler o `config.json`.
`effects-toggle.sh` é um delegate fino, sem saída própria. Os helpers de
aparência nunca são chamados pelo projetor e jamais podem escrever em
`data/generated/`.

### Superfícies de projeção

`project_effective` renderiza todos os consumidores do ARGVUS em uma única
árvore de staging antes da publicação:

| Superfície | Saída |
| --- | --- |
| Hyprland | `generated/hypr/`, incluindo perfis de tema, `spaces-effective.conf`, `borders-effective.conf`, `hypridle.conf` e `monitors.lua` |
| Layout/efeitos | `generated/effects/`, `generated/spaces.conf`, `generated/borders.conf` |
| Waybar | `generated/waybar/`, incluindo `argvus-taskbar.css` e `argvus-widget-telemetry.{jsonc,css}` |
| Temas de widget-telemetry | `generated/widget-telemetry/` e `generated/waybar/widget-telemetry-themes/` |
| QuickShell | `generated/quickshell/` |
| Notificações | `generated/notifications/` e `dunstrc` |
| Lock screen | `generated/hypr/hyprlock.conf` e os perfis de tema do Hyprtoolkit |
| GTK | `generated/gtk/` |
| Yazi | `generated/yazi/` |
| Superfile | `generated/superfile/` |
| Perfis de terminal e Qt | `generated/terminal/`, `generated/qt6ct/` |
| Fontes | `generated/fonts.conf` e os blocos de fonte gerenciados nos stylesheets do Waybar |
| Dispositivos removíveis | `generated/removable-devices/theme.css` |
| Wallpaper | `generated/hypr/hyprpaper.conf` e `data/hypr/hyprpaper.conf` |
| Control Panel e defaults | `data/control-panel/`, `data/control-center/defaults.json` |
| Calendário, input, teclado, energia | TOML do calendário, fragmento de input, estado do teclado, marcador keep-awake |

A camada de tema também é espelhada fora da raiz generated:
`data/waybar/argvus-taskbar.{jsonc,css}`,
`data/waybar/argvus-widget-telemetry.{jsonc,css}`, `data/foot/`, `data/qt6ct/`, `data/hypr/hyprtoolkit.conf` e
`data/hypr/application-style.conf`. Essas cópias são substituídas pelos defaults
empacotados quando a seção de aparência muda, e o projetor então reescreve apenas
seus blocos gerenciados delimitados — o bloco de fonte nos stylesheets do Waybar
e os blocos `ARGVUS_TELEMETRY_*` no perfil de telemetria — então texto fora dos
marcadores não sobrevive a uma troca de tema. `fonts.targets` e
`control_panel.widget_telemetry_blocks` continuam canônicos: o Control Center os
persiste e nunca escreve esses arquivos por conta própria.

A seleção de blocos de widget-telemetry é canônica em
`/control_panel/widget_telemetry_blocks`, um array com os sete nomes de bloco
(`system`, `cpu_gpu`, `memory`, `storage`, `processes`, `network`, `keys`). O arquivo
legado `$XDG_STATE_HOME/argvus/widget-telemetry-blocks` é lido apenas como
fallback de migração e nunca é escrito. `argvus-widget-telemetry-toggle blocks
set` e `blocks all` aplicam o array canônico em uma única escrita atômica, e
`apply-state` pressupõe que a seleção já foi persistida por quem o chamou.

O estado de accent e do modo GTK usa a mesma fronteira Rust. Use `argvus-config
accent COLOR` ou `argvus-config accent --theme-default` para controlar o
accent, e `argvus-config mode toggle|set dark|light` para o modo GTK. Os
scripts históricos de accent e modo são apenas wrappers de compatibilidade;
`.accent-color`, `.accent-custom` e `.gtk-mode` servem somente para migração ou
projeção.

As preferências de espaçamento e bordas do Hyprland seguem a mesma fronteira.
Os comandos argvus-config spaces e argvus-config borders (e os wrappers
históricos de compatibilidade) atualizam /layout; o Rust gera
data/generated/spaces-effective.conf e data/generated/borders-effective.conf
e controla validação e aplicação live via hyprctl. .spaces e .borders são
apenas estado de migração e nunca são a fonte de verdade.

Caminhos lógicos do ARGVUS também são resolvidos por `argvus-config paths`. O
arquivo legado `paths.sh` apenas expõe funções de compatibilidade; overrides
nativos, dados do usuário, dados gerados e fallback de sistema são resolvidos
pelo modelo de paths Rust.

`patch` aplica um objeto JSON cujas chaves são JSON pointers em uma única
operação protegida por lock. `apply-theme` executa a transação atômica do tema
e preserva os valores `accent_custom` e `wallpaper_custom`. Os defaults de
tema ficam no manifesto empacotado
`appearance/config/theme-defaults.json`; CSS, Lua, Rasi e Qt são projeções
desse estado. Mutations constroem um `NextUserConfig`, resolvem um
`EffectiveConfig`, geram todos os projectors em staging e só então publicam o
JSON canônico. O marker operacional `.config-transaction.json` permite que
`argvus-config recover` reprojete a configuração anterior após uma queda entre
a publicação de `data/` e do JSON.

`apply-theme` recebe a variante explicitamente como `--variant sticky|float` e
resolve o nome pedido através do manifesto, então um pedido nunca pode resolver
para os assets da outra variante. `theme-manifest THEME` expõe essa mesma
resolução somente-leitura: imprime a entrada do manifesto junto com o
`asset_id` e o `layout_variant` resolvidos, que é o que um chamador precisa para
endereçar os assets empacotados. Sticky é o id sem sufixo e apenas Float carrega
o sufixo `-float`, então `dracula-sticky` e `argvus-dark-dracula-sticky` são
aceitos e normalizam para `dracula`.

Com `--raw`, valores string são impressos sem aspas JSON, enquanto objetos e arrays continuam sendo valores JSON. O projetor não grava mais dot-files de tema ou layout; `config.json` permanece a única fonte persistente dessas preferências. Se a projeção falhar, o reload da sessão é interrompido antes de reiniciar as superfícies do desktop, evitando apresentar um tema antigo parcialmente aplicado como se fosse o atual.

## Exportar e importar

O mesmo modelo lógico é usado na troca de perfis:

```sh
argvus-config export --scope appearance --output appearance.json
argvus-config import --scope appearance appearance.json
```

Use `--scope desktop` para o modelo completo gerenciado do desktop. Campos ausentes usam os defaults do ARGVUS; arquivos gerados não devem ser editados diretamente.

Os perfis de tema do Control Center usam o escopo `appearance` dentro do arquivo. Perfis novos incluem o JSON canônico junto com os arquivos nativos legados para compatibilidade; ao aplicar um perfil, esse JSON é importado e suas projeções são regeneradas. O escopo `desktop` na linha de comando continua sendo a opção para um backup completo da configuração.

O schema atual agrupa preferências em `appearance`, `layout`, `effects`, `fonts`, `control_panel`, `default_apps`, `keyboard_shortcuts`, `hyprland`, `displays`, `power`, `session`, `calendar` e `removable_devices`. `default_apps` é a única fonte persistente das preferências de aplicativos, e `keyboard_shortcuts` usa chaves estáveis, em inglês e independentes do locale, com strings ou `null` para atalhos desativados.

O Control Center lê os valores canônicos de aparência ao abrir uma página de Aparência e grava as alterações através do `argvus-config`. Os arquivos nativos de estado continuam como projeções usadas pelos scripts de aparência e por instalações antigas.

O Control Panel usa `control_panel.cards` para a ordem e a visibilidade dos cards, e `control_panel.enabled` para o interruptor geral do painel. Os arquivos `control-panel/cards.json` e `state/control-panel` continuam sendo mantidos como projeções de compatibilidade. Assim, as alterações feitas no Control Panel também ficam disponíveis nos perfis exportados/importados pelo Control Center.

As preferências de fontes usam `fonts.targets` para família, estilo e tamanho dos alvos taskbar, telemetry, Control Panel, sistema, aplicativos, terminal e navegador. `fonts.rendering` armazena antialiasing, hinting, ordem de subpixel e DPI. `fonts.conf`, a regra do fontconfig, as configurações do desktop e os blocos de fonte gerenciados dentro dos stylesheets do Waybar são projeções: o Control Center persiste `/fonts/targets/*` através do `argvus-config`, e o projetor reescreve apenas o bloco delimitado.

As preferências do Hyprland usam `keyboard_shortcuts` para atalhos, `hyprland.input` para mouse e touchpad e `hyprland.keyboard` para layout, variante e opções XKB. O ID do binding é mapeado para uma `config_key` estável (por exemplo `window.close` → `close_window` e `window.drag_mouse` → `drag_window__floating_window_only`). As preferências de monitores usam `displays.monitors`, `displays.workspaces` e `displays.primary_monitor`. Os arquivos TOML nativos e Lua gerados em `data/generated/` continuam sendo projeções consumidas pelo Hyprland.

As preferências de efeitos usam `effects` para o estado global das animações, o controle global de blur (`blur_global_enabled`), os parâmetros de blur do Hyprland (`blur_size`, `blur_passes`, `blur_brightness`, `blur_noise`, `blur_contrast`, `blur_vibrancy`, `blur_vibrancy_darkness`) e os valores de transparência por superfície. Effects pertencem ao tema, não à variante: Sticky e Float usam o mesmo arquivo `<theme>.conf`, enquanto as projeções de layout diferem. As superfícies atuais incluem `taskbar`, `control-panel` e `widget-telemetry`; os valores de transparência são percentuais de `0` a `100`, os parâmetros de blur são números dentro das faixas declaradas em `schema.json`, e os estados de habilitação são booleanos.

As preferências de energia usam `power.screen_off_minutes`, `power.lock_minutes`, `power.keep_awake` e as políticas da tampa e do botão de energia. As formas projetadas são `data/generated/hypr/hypridle.conf` e o marcador `data/power/keep-awake`; `.keep-awake` é apenas entrada de migração. `session.language` é o idioma canônico da interface, enquanto o arquivo legado `language` continua como projeção de compatibilidade. As preferências do calendário usam `calendar` e são projetadas para o TOML de usuário do taskbar-calendar. As preferências de dispositivos removíveis usam `removable_devices`; o aplicativo de dispositivos removíveis trata essa seção como override de usuário sobre os defaults empacotados.

`migrate` importa os arquivos antigos e as seções legadas uma vez, preservando o valor canônico existente. `defaults.json`, `/defaults`, `/hyprland/keybindings`, `keybindings.toml`, dot-files, `generated/` e diretórios de componentes antigos são movidos para `data/`; reloads repetidos não recriam o layout antigo. Selecionar novamente o tema atual é idempotente: quando o estado canônico e as projeções já coincidem, o plano informa nenhuma seção alterada e a sessão não abre splash nem reinicia serviços.

Power segue o mesmo contrato de projeção: o Control Center grava `/power` em `config.json`, `argvus-config project` renderiza atomicamente `data/hypr/hypridle.conf` e somente depois `argvus-sessionctl` pode reiniciar `argvus-hypridle.service`. Apagar esse arquivo é recuperável com `argvus-config project`; o valor `0` remove o timeout correspondente e significa Nunca.

A aparência da tela de bloqueio é projetada pelo Rust a partir do tema,
accent, efeitos, wallpaper e fontes canônicos para `data/hypr/hyprlock.conf`.
O antigo `hyprlock-theme.sh` permanece apenas por compatibilidade e delega a
`argvus-config lock apply`; `--invalidate` invalida somente o cache derivado do
wallpaper desfocado.

As alterações de timeout usam `/power/lock_minutes` e
`/power/screen_off_minutes` como fonte de verdade. `argvus-config project`
reutiliza o renderer Hypridle existente, e `idle-timeout.sh` é apenas um
adaptador de compatibilidade; o restart do serviço continua delegado ao
`argvus-sessionctl`.

A aparência das notificações é derivada do tema, accent e efeitos canônicos em
`data/notifications/dunstrc`. O antigo `theme.sh` de notifications expõe
somente funções de compatibilidade que chamam `argvus-config notifications`; ele
não faz parsing nem mutação da configuração do Dunst.

Adaptadores voltados a hardware, como descoberta de monitores e controle de
Bluetooth, não fazem parte dessas projeções: continuam sob o owner do
componente até receberem um boundary Rust próprio para estado de runtime e
ciclo de vida dos dispositivos.
O domínio Rust de display também importa a saída do `nwg-displays`; o programa
externo é apenas um adapter e `/displays` continua sendo a fonte canônica,
enquanto `data/generated/hypr/monitors.lua` é derivado. Seleção de wallpaper,
modo do grupo utilitário da taskbar e consulta de aplicativos padrão usam APIs
de `argvus-config`; os entrypoints Shell históricos são wrappers de UI ou
compatibilidade. Keep Awake é a preferência persistente `/power/keep_awake`, projetada no marcador `data/power/keep-awake`,
enquanto lock-DPMS é estado de sessão em `$XDG_CACHE_HOME/argvus/lock-dpms`.
O layout de teclado agora é de responsabilidade do
helper Rust `argvus-keyboard-layout`: a ordem configurada vem da projeção de
input do Hyprland ou do fallback de locale, enquanto o layout ativo continua
sendo estado de sessão descoberto pelo Hyprland. O entrypoint de ciclo e o
daemon de layout são apenas wrappers de compatibilidade; a unit executa o
daemon Rust diretamente e `argvus-sessionctl keyboard-layout cycle` é o comando
estável para callers.

O estado de runtime do Bluetooth segue a fronteira de adapter externo, mas não
faz parte do `argvus-config`: `/usr/bin/argvus-bluetooth` concentra o parsing e
a validação tipados sobre o `bluetoothctl`, retries limitados, verificação do
serviço do sistema e estado do RFKill. `bluetooth-control.sh` e
`argvus-bluetoothctl` permanecem como entrypoints de compatibilidade. Energia
do adapter, dispositivos pareados e conexões continuam sendo estado externo
do BlueZ, não estado canônico JSON; a validação live depende do hardware
disponível.

## Identidade do runtime e reconciliação

Use `argvus-config status --verbose` para consultar o binário, identidade do
build, raiz de configuração, raiz de arquivos gerados, raiz de assets, fonte do
manifesto e generation ativa. Ele informa os estados `synchronized`, `stale`,
`incomplete`, `legacy` e `pending-transaction`.

`argvus-config --version` informa versão, revisão do build quando fornecida e
perfil. Testes de desenvolvimento devem injetar explicitamente binário e
assets; não devem usar `/usr/bin/argvus-config` ou `~/.config/argvus`
silenciosamente. `apply` normaliza com segurança identificadores legados como
`argvus-dark-float` para theme e variant separados antes de reconstruir as
projeções. O manifesto de assets instalado fica em
`/usr/share/argvus/version.json`.
