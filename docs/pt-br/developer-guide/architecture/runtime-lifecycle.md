---
title: Ciclo de vida do runtime
description: Inicialização da sessão e ownership dos serviços.
slug: pt/0.4.0/docs/developer-guide/architecture/runtime-lifecycle
---

`argvus-session` importa o ambiente, inicia o Hyprland por `argvus-start` e inicia `argvus-session.target` quando o compositor está pronto. O target inclui shell, notificações, wallpaper, idle e clipboard. `argvus-sessionctl` coordena start, stop, reload, restart, status e logs.

## Inicialização

O entrypoint gráfico é `/usr/share/wayland-sessions/argvus.desktop`, que executa `/usr/bin/argvus-session`. A sessão importa o ambiente de configuração do ARGVUS e do XDG, sincroniza-o com `systemd --user` e com a ativação por D-Bus e delega o início do compositor ao `argvus-start`.

O `argvus-start` valida a configuração do Hyprland, aplica o fallback empacotado quando encontra um arquivo gerado padrão ou uma configuração do usuário inválida, detecta compatibilidade com virtualização e inicia o Hyprland. A ponte de prontidão do compositor chama `argvus-sessionctl ready`, que inicia o target do ARGVUS com o ambiente Wayland e Hyprland atual.

O target usa uma unidade de preparação one-shot antes de iniciar os componentes persistentes. A preparação aplica configuração gerada, estado de tema e acento, estado dos monitores, integração de aplicativos e ambiente da sessão. Os serviços persistentes pertencem ao `systemd --user`, não a processos iniciados de forma ad hoc em background.

`argvus-sessionctl ready` projeta a configuração canônica antes de iniciar o target. Uma falha fatal de projeção ou do início do target é registrada e encerra o overlay de carregamento independente, evitando um spinner aparentemente infinito. Consumidores visuais opcionais devem usar um fallback ou reportar uma falha degradada sem substituir o target da sessão.

A resolução de paths e o plano de preparação do Hyprland pertencem ao Rust. Os
arquivos legados `paths.sh` e `hypr-init.sh` são entrypoints de compatibilidade;
eles não contêm mais precedência de paths, loops de startup ou orquestração de
projeções.

## Serviços e encerramento

O target normalmente possui `argvus-control-panel.service`, `argvus-taskbar.service`, `argvus-widget-telemetry.service` quando habilitado, `argvus-wallpaper.service`, `argvus-dunst.service`, `argvus-hypridle.service`, watchers de clipboard, notificações de layout do teclado, integração do PolicyKit e o overlay de carregamento da sessão. A falha de um serviço opcional não substitui toda a sessão.

Os serviços que precisam de uma conexão cliente Wayland, incluindo o Control Panel e o gerenciador de ociosidade, aguardam um runtime Hyprland utilizável antes de iniciar o cliente. Isso evita iniciar o Quickshell ou o hypridle contra um socket antigo durante a troca do compositor; se o Hyprland for substituído, o systemd pode iniciar o componente novamente quando o novo runtime estiver disponível.

As transições de tema serializam a projeção e a distribuição para os serviços como uma única operação de ciclo de vida. O reload da transição reutiliza a projeção existente e não entra nela novamente enquanto o lock do tema está mantido, evitando que reloads concorrentes bloqueiem a preparação da sessão.

Quando o Hyprland encerra, o `argvus-start` para `argvus-session.target` e os serviços ligados a ele. Isso impede que taskbar, shell, notificações, idle e clipboard sobrevivam ao logout.

## Reload

```sh
argvus-sessionctl reload
```

O reload importa novamente o ambiente gráfico e então **aplica** uma generation commitada quando um comando de domínio anterior, como `apply-theme`, `accent`, `set`/`patch` ou uma mudança de bloco de widget-telemetry, já projetou a mudança. Nesse caso ele restaura os consumidores listados no plano do manifesto e não chama `argvus-config project` de novo, porque reprojetar reconstruiria uma árvore idêntica e descartaria o plano. Projeção, migração e `ensure` só acontecem quando não existe um plano pendente válido — no startup da sessão, ou depois que `argvus-config recover` precisou restaurar o documento anterior. Um no-op limpo só é permitido quando as projeções e o runtime gerenciado estão atuais. Se o estado canônico não mudou, mas um socket do compositor, serviço ou consumidor está obsoleto, o reload o reconcilia em vez de retornar cedo. O fan-out normal cobre Hyprland, taskbar, widget telemetry, Control Panel, notificações, wallpaper e idle conforme o manifesto de projeção; consumidores opcionais atuais permanecem sem churn. Todo arquivo em `data/generated/` é escrito pelo `argvus-config` nesse momento, então a sessão nunca re-renderiza um tema por conta própria. Depois disso o reload executa o `theme-adapters.sh`, que reconcilia os consumidores que não conseguem ler o documento canônico ou a projeção publicada por conta própria. Esses adapters só leem a generation commitada: escrevem os próprios arquivos e caches de consumidor e nunca escrevem o documento canônico nem `data/generated/`, então uma única ação do usuário ainda custa uma mutation, uma projeção, uma publicação e um reload. Use `argvus-sessionctl restart <componente>` apenas quando um serviço isolado precisar de um restart explícito. Consulte a [referência de serviços](../../reference/services/) para os nomes exatos das units.

As operações de projeção e reload são serializadas por um lock no runtime do usuário. Para uma medição temporária, defina `ARGVUS_SESSION_PERF=1` antes do reload e consulte `~/.local/state/argvus/session.log`; o trace registra as fronteiras das etapas sem alterar o comportamento normal.
