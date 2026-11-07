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

Se aparecer `greeter.handoff-takeover-failed` sem `handoff.master-acquired`,
consulte `handoff.master-unavailable card=<nome> errno=<n>` para identificar o
erro da aquisição. Um observador aberto enquanto outro compositor era master
pode retornar errno 13 (`EACCES`) mesmo depois da liberação. O helper reabre a
mesma GPU para adquirir o master implicitamente; `handoff.master-reopened`
registra essa recuperação e um novo framebuffer é preparado nesse descritor. O greeter cancela o helper que falhou
(`greeter.handoff-cancelled`), evitando outra espera antes do compositor do usuário.
Um `release` durante a aquisição registra `handoff.release-before-master`: o helper
fecha seus descritores DRM antes de confirmar. Um crash do compositor do greeter
em Hyprland/Aquamarine é uma ocorrência separada; cancelar o helper não demonstra
que o problema do compositor ou driver foi resolvido.

`greeter.vt-switch-returned active=<tty>` registra a VT ativa ao retornar da
chamada de troca ao logind. O erro de timeout do primeiro quadro também registra
`active-vt=<tty>`. Esses registros são amostras, sem garantir a conclusão da
transferência do DRM.

Para comparar antes e depois, salve o trabalho e faça cinco logins por versão,
com a mesma máquina, tema e monitores. Antes do próximo logout, copie
`/run/argvus-greeter/timing.log`, o log da sessão e o journal do boot para um
diretório de amostra separado em `/tmp/argvus/`; a próxima execução do greeter
substitui seu log de tempos. Registre o identificador do boot e compare os marcos
apenas dentro do mesmo boot. Meça da autenticação ao primeiro quadro do handoff,
à entrada do wrapper, ao lançamento final do Hyprland e à parada do indicador de
carregamento. Compare mediana e pior tempo entre autenticação e parada do indicador,
verificando visualmente tela preta, piscadas e indicador preso. A duração das
superfícies do desktop sozinha não representa o tempo total do login. Faça testes
de falha e recuperação numa sessão descartável, sem encerrar uma sessão com
trabalho não salvo.

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

O `timing.log` guarda só o último login: marcos do greeter (`greeter.*`) e da superfície DRM do handoff (`handoff.master-acquired`, `handoff.output-found`, `handoff.first-frame`, `handoff.release-received`, `handoff.replaced-by-compositor`). As linhas `handoff.crtc` registram cada mudança das CRTCs enquanto o handoff espera o DRM master; uma linha com `fb=0` ou `mode_valid=false`, ou `handoff.output-found ... framebuffer=0`, indica que o compositor do greeter desligou a saída antes de o handoff assumir, e o monitor precisou sincronizar de novo. `greeter.watchdog-stop-requested` indica que a sessão do greeter terminou sem esperar o compositor, e `greeter.compositor-exit-deferred`, que o compositor recebe o pedido de saída depois que a superfície do handoff sai; `greeter.compositor-exit-requested` sozinho indica que o pedido foi feito na hora. Se as linhas do greetd e do logind no journal mostrarem um intervalo de cerca de meio segundo entre o fim da sessão do greeter e a abertura da sessão do usuário, o teardown do compositor do greeter rodou durante o login. Isso é esperado enquanto a marca mostrar `compositor-in-foreground=true`: o desligamento do terminal do greeter encerra o compositor antes da saída adiada. Isso acontece quando o greeter não conseguiu sair da VT antes de o compositor encerrar: procure `greeter.vt-switch-requested` seguido de `greeter.handoff-drawing`; quando o segundo faltar, `greeter.handoff-takeover-failed error=<motivo>` diz o porquê. `handoff.page-flip` indica que o handoff trocou o último quadro do greeter sem modeset; `handoff.page-flip-failed errno=<n>` seguido de `handoff.modeset` indica que o kernel recusou o page flip. `handoff.overlay plane=<id>` e depois `handoff.retired` indicam que o overlay escondeu os primeiros quadros do Hyprland até o indicador da sessão aparecer; `handoff.overlay-unavailable` indica que a CRTC não tinha plano overlay livre, e `handoff.retire-timeout` indica que o indicador da sessão não enviou `retire`. O indicador da sessão (`argvus-greeter-handoff`) grava `first layer-shell frame callback` e `stop requested` no journal.

Uma entrada `project.theme-switch` ou `prepare.startup-theme` com duração diferente de zero é esperada após uma mudança de aparência, arquivos de tema alterados, um novo acento ou a migração de um carimbo antigo. Reinstalar arquivos idênticos preserva o cache de startup baseado em conteúdo. Se ela aparecer em todo login, os arquivos de tema materializados estão ausentes; execute `argvus theme-switch <tema>` uma vez.

O log da sessão fica em `$XDG_STATE_HOME/argvus/session.log` (normalmente `~/.local/state/argvus/session.log`). Depois de alterar um override do Hyprland, faça logout/login para que a entrada da sessão valide e selecione o arquivo novamente. Veja [sessão gráfica](/pt/docs/argvus-session/graphical-session/) para o fluxo normal.
