---
title: Problemas ao iniciar a sessão
description: Diagnostique greeter, Hyprland e serviços de usuário.
---

```sh
argvus sessionctl status
argvus sessionctl logs
systemctl --user status argvus-session.target
```

Verifique separadamente greeter, `argvus start`, Hyprland e a unit afetada. Um build bem-sucedido não prova que o payload instalado ou o override ativo está atualizado.

Faça as verificações nesta ordem:

1. Confirme que o greeter está usando a configuração instalada do greetd do ARGVUS.
2. Leia o log da sessão e procure uma configuração Lua do Hyprland ausente ou inválida.
3. Verifique a prontidão e o estado do target com `systemctl --user status argvus-session.target`.
4. Inspecione o componente que falhou em vez de reiniciar todos os serviços de uma vez.

Se `argvus-session.target` estiver inativo enquanto o overlay de carregamento
estiver ativo, verifique o erro de projeção antes de reiniciar os serviços:

```sh
argvus config project
journalctl --user -u argvus-session.target -u argvus-session-prepare.service -b --no-pager
```

O `argvus-config` é o único escritor abaixo de `data/generated/`, então `project` é
a forma autoritativa de reproduzir uma geração durante o diagnóstico. O início da
sessão o executa porque ainda não existe um plano confirmado; o plano gravado em
`${XDG_STATE_HOME:-$HOME/.local/state}/argvus/config-projection.json` é o que um
`argvus sessionctl reload` posterior aplica, sem projetar de novo.

A sessão encerra o overlay quando a preparação fatal ou o início do target
falha. Uma falha visual deve usar o fallback documentado ou ser reportada
explicitamente; ela não pode ser ocultada como se a sessão tivesse iniciado.

## Login lento

Cada etapa do início grava sua duração no log da sessão. Para ver as etapas do último login, da mais lenta para a mais rápida:

```sh
grep timing ~/.local/state/argvus/session.log | tail -30 | sort -k4 -rn
systemd-analyze --user critical-chain argvus-session.target
```

Para medir a troca do greeter para o desktop, leia os marcos do greeter e do handoff junto com o journal. Todos usam o `CLOCK_MONOTONIC`, então ficam alinhados:

```sh
cat /run/argvus-greeter/timing.log
journalctl -b -o short-monotonic --since -2min | grep -E 'greetd|Hyprland|coredump|argvus-greeter-handoff|session-loading'
grep monotonic_ns ~/.local/state/argvus/session.log | tail -5
```

O `timing.log` guarda só o último login: marcos do greeter (`greeter.*`) e da superfície DRM do handoff (`handoff.master-acquired`, `handoff.output-found`, `handoff.first-frame`, `handoff.release-received`, `handoff.replaced-by-compositor`). As linhas `handoff.crtc` registram cada mudança das CRTCs enquanto o handoff espera o DRM master; uma linha com `fb=0` ou `mode_valid=false`, ou `handoff.output-found ... framebuffer=0`, indica que o compositor do greeter desligou a saída antes de o handoff assumir, e o monitor precisou sincronizar de novo. Isso acontece quando o Hyprland instalado não suporta `HYPRLAND_RETAIN_FRAME_ON_EXIT`. `handoff.page-flip` indica que o handoff trocou o último quadro do greeter sem modeset; `handoff.page-flip-failed errno=<n>` seguido de `handoff.modeset` indica que o kernel recusou o page flip. O indicador da sessão (`argvus-greeter-handoff`) grava `first layer-shell frame callback` e `stop requested` no journal.

Uma entrada `project.theme-switch` ou `prepare.startup-theme` só é esperada após uma mudança de aparência, uma atualização de pacote ou um novo acento. Se ela aparecer em todo login, os arquivos de tema materializados estão ausentes; execute `argvus theme-switch <tema>` uma vez.

O log da sessão fica em `$XDG_STATE_HOME/argvus/session.log` (normalmente `~/.local/state/argvus/session.log`). Depois de alterar um override do Hyprland, faça logout/login para que a entrada da sessão valide e selecione o arquivo novamente. Veja [sessão gráfica](/pt/docs/argvus-session/graphical-session/) para o fluxo normal.
