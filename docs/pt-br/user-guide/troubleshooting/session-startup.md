---
title: Problemas ao iniciar a sessão
description: Diagnostique greeter, Hyprland e serviços de usuário.
slug: pt/0.4.0/docs/user-guide/troubleshooting/session-startup
---

```sh
argvus-sessionctl status
argvus-sessionctl logs
systemctl --user status argvus-session.target
```

Verifique separadamente greeter, `argvus-start`, Hyprland e a unit afetada. Um build bem-sucedido não prova que o payload instalado ou o override ativo está atualizado.

Faça as verificações nesta ordem:

1. Confirme que o greeter está usando a configuração instalada do greetd do ARGVUS.
2. Leia o log da sessão e procure uma configuração Lua do Hyprland ausente ou inválida.
3. Verifique a prontidão e o estado do target com `systemctl --user status argvus-session.target`.
4. Inspecione o componente que falhou em vez de reiniciar todos os serviços de uma vez.

Se `argvus-session.target` estiver inativo enquanto o overlay de carregamento
estiver ativo, verifique o erro de projeção antes de reiniciar os serviços:

```sh
argvus-config project
journalctl --user -u argvus-session.target -u argvus-session-prepare.service -b --no-pager
```

O `argvus-config` é o único escritor abaixo de `data/generated/`, então `project` é
a forma autoritativa de reproduzir uma geração durante o diagnóstico. O início da
sessão o executa porque ainda não existe um plano confirmado; o plano gravado em
`${XDG_STATE_HOME:-$HOME/.local/state}/argvus/config-projection.json` é o que um
`argvus-sessionctl reload` posterior aplica, sem projetar de novo.

A sessão encerra o overlay quando a preparação fatal ou o início do target
falha. Uma falha visual deve usar o fallback documentado ou ser reportada
explicitamente; ela não pode ser ocultada como se a sessão tivesse iniciado.

O log da sessão fica em `$XDG_STATE_HOME/argvus/session.log` (normalmente `~/.local/state/argvus/session.log`). Depois de alterar um override do Hyprland, faça logout/login para que a entrada da sessão valide e selecione o arquivo novamente. Veja [sessão gráfica](../sessions/graphical-session/) para o fluxo normal.
