---
title: Ciclo de vida do runtime
description: Inicialização da sessão e ownership dos serviços.
---

`argvus-session` importa o ambiente, inicia o Hyprland por `argvus start` e inicia `argvus-session.target` quando o compositor está pronto. O target inclui shell, notificações, wallpaper, idle e clipboard. `argvus sessionctl` coordena start, stop, reload, restart, status e logs.

## Inicialização

O entrypoint gráfico é `/usr/share/wayland-sessions/argvus.desktop`, que executa `argvus session`. A sessão importa o ambiente de configuração do ARGVUS e do XDG, sincroniza-o com `systemd --user` e com a ativação por D-Bus e delega o início do compositor ao `argvus start`.

O `argvus start` valida a configuração do Hyprland, aplica o fallback empacotado quando encontra um arquivo gerado padrão ou uma configuração do usuário inválida, detecta compatibilidade com virtualização e inicia o Hyprland. A ponte de prontidão do compositor chama `argvus sessionctl ready`, que inicia o target do ARGVUS com o ambiente Wayland e Hyprland atual.

O target usa uma unidade de preparação one-shot antes de iniciar os componentes persistentes. A preparação aplica configuração gerada, estado de tema e acento, estado dos monitores, integração de aplicativos e ambiente da sessão. Os serviços persistentes pertencem ao `systemd --user`, não a processos iniciados de forma ad hoc em background.

`argvus sessionctl ready` inicia o target, cujo `argvus-config.service` projeta a configuração canônica antes da preparação. Uma falha fatal de projeção ou do início do target é registrada e encerra o overlay de carregamento independente, evitando um spinner aparentemente infinito. Consumidores visuais opcionais devem usar um fallback ou reportar uma falha degradada sem substituir o target da sessão.

### Custo do login

Um login sem mudança de configuração não repete trabalho caro:

- A projeção só executa a troca de tema e o adaptador de efeitos quando as seções de aparência ou de efeitos mudaram, ou quando as saídas reais (as cópias de usuário materializadas e `data/state/effects/<tema>.conf`) estão ausentes.
- A preparação só aplica o tema de startup, e o acento por cima dele, quando o carimbo em `$XDG_CACHE_HOME/argvus/session/theme-startup.stamp` deixa de corresponder aos arquivos de tema empacotados e ao marcador de acento. Uma atualização de pacote ou um novo acento o invalida uma vez. Quando o tema é reaplicado, a preparação também recarrega o Hyprland, que iniciou com os arquivos anteriores.
- As configurações GTK (`gsettings`) são gravadas somente quando o tema de startup foi reaplicado. Entre logins o dconf as mantém, então um login estável não as executa de novo.
- A unidade `argvus-dunst.service` é `Type=dbus`: ela conta como iniciada quando o Dunst assume `org.freedesktop.Notifications`, então o target da sessão não espera a restauração do estado de pausa.
- A projeção regrava um arquivo gerado somente quando o conteúdo mudou. Arquivos inalterados ficam como estão, então um login estável não faz fsync deles de novo.
- A etapa de monitores da preparação (`argvus config display session-prepare`) aplica apenas DPMS e saídas desativadas: ela não projeta de novo nem recarrega o compositor.
- `argvus-session-reload.service` executa `argvus sessionctl reload-desktop --login`, que só recarrega o Hyprland quando o manifesto da projeção lista `hyprland` entre os consumidores afetados. Os componentes do desktop não esperam por ele.

O indicador da sessão (o helper do handoff em modo Wayland) fica visível desde a troca de compositor até `argvus sessionctl ready` ter iniciado o target, então um único spinner cobre todo o início. Ele não tem `PartOf=`, porque recriar o target num novo login não pode derrubá-lo; o `sessionctl` o encerra explicitamente, e `RuntimeMaxSec=30` o encerra se nenhum stop chegar.

Cada etapa da preparação, do reload e dos adaptadores da projeção grava sua duração no log da sessão como `<epoch-ms> timing <etapa> <N>ms`; veja [problemas no início da sessão](/pt/docs/user-guide/troubleshooting/session-startup/).

A resolução de paths, a preparação da sessão e o plano de reload do Hyprland
pertencem ao Rust. `argvus config session-prepare` e `argvus config
session-reload-config` executam as etapas de startup e de reload; o script legado
`hypr-init.sh` foi removido.

## Serviços e encerramento

O target normalmente possui `argvus-control-panel.service`, `argvus-taskbar.service`, `argvus-widget-telemetry.service` quando habilitado, `argvus-wallpaper.service`, `argvus-dunst.service`, `argvus-hypridle.service`, watchers de clipboard, notificações de layout do teclado e integração do PolicyKit. O overlay de carregamento da sessão é iniciado pelo `argvus start`, não pelo target. A falha de um serviço opcional não substitui toda a sessão.

Os serviços que precisam de uma conexão cliente Wayland, incluindo o Control Panel e o gerenciador de ociosidade, aguardam um runtime Hyprland utilizável antes de iniciar o cliente. Isso evita iniciar o Quickshell ou o hypridle contra um socket antigo durante a troca do compositor; se o Hyprland for substituído, o systemd pode iniciar o componente novamente quando o novo runtime estiver disponível.

Uma troca de tema serializa o commit, o build, a projeção e as recargas dos consumidores como uma única operação de ciclo de vida, sob o lock do tema. Ela não recarrega o `argvus-config.service` nem reinicia serviços: a projeção que ela roda só grava arquivos e deixa o tema para a troca enquanto o lock está mantido, e os consumidores em execução recarregam no lugar (veja [Aparência e temas](/pt/docs/developer-guide/subsystems/appearance-and-themes/)).

Quando o Hyprland encerra, o `argvus start` para `argvus-session.target` e os serviços ligados a ele. Isso impede que taskbar, shell, notificações, idle e clipboard sobrevivam ao logout.

## Reload

```sh
argvus sessionctl reload
```

O reload importa novamente o ambiente gráfico e então **aplica** uma generation commitada quando um comando de domínio anterior, como `apply-theme`, `accent`, `set`/`patch` ou uma mudança de bloco de widget-telemetry, já projetou a mudança. Nesse caso ele restaura os consumidores listados no plano do manifesto e não chama `argvus config project` de novo, porque reprojetar reconstruiria uma árvore idêntica e descartaria o plano. Projeção, migração e `ensure` só acontecem quando não existe um plano pendente válido — no startup da sessão, ou depois que `argvus config recover` precisou restaurar o documento anterior. Um no-op limpo só é permitido quando as projeções e o runtime gerenciado estão atuais. Se o estado canônico não mudou, mas um socket do compositor, serviço ou consumidor está obsoleto, o reload o reconcilia em vez de retornar cedo. O fan-out normal cobre Hyprland, taskbar, widget telemetry, Control Panel, notificações, wallpaper e idle conforme o manifesto de projeção; consumidores opcionais atuais permanecem sem churn. Todo arquivo em `data/generated/` é escrito pelo `argvus-config` nesse momento, então a sessão nunca re-renderiza um tema por conta própria. Depois disso o reload executa o `theme-adapters.sh`, que reconcilia os consumidores que não conseguem ler o documento canônico ou a projeção publicada por conta própria. Esses adapters só leem a generation commitada: escrevem os próprios arquivos e caches de consumidor e nunca escrevem o documento canônico nem `data/generated/`, então uma única ação do usuário ainda custa uma mutation, uma projeção, uma publicação e um reload. Use `argvus sessionctl restart <componente>` apenas quando um serviço isolado precisar de um restart explícito. Consulte a [referência de serviços](/pt/docs/reference/services/) para os nomes exatos das units.

As operações de projeção e reload são serializadas por um lock no runtime do usuário. Para uma medição temporária, defina `ARGVUS_SESSION_PERF=1` antes do reload e consulte `~/.local/state/argvus/session.log`; o trace registra as fronteiras das etapas sem alterar o comportamento normal.
