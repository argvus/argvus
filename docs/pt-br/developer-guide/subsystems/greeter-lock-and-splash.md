---
title: Greeter, lock e splash
description: Visuais separados de login e inicialização.
---

`argvus-greeter` fornece a interface greetd; o overlay de carregamento da sessão é o auxiliar `argvus loading-theme` do pacote `argvus`; `argvus-boot-splash` fornece o splash de boot; `argvus-lock` fornece Hyprlock. Eles têm ciclos de vida e payloads distintos.

O greeter roda antes da sessão autenticada e usa a fronteira IPC/PAM do greetd. O overlay de carregamento inicia quando o novo socket Wayland fica disponível e termina quando o Hyprland informa que está pronto. O tema Plymouth pertence ao boot e não é a tela de login. O lock é usado depois do login e não substitui o greeter.

A transição de login tem uma superfície DRM própria. Depois que a autenticação dá certo, o greeter inicia `/usr/lib/argvus/greeter/handoff` (`argvus-greeter-handoff`) antes de pedir ao greetd para iniciar a sessão. O helper assume o DRM master assim que o compositor do greeter o libera e mantém um quadro na tela, para que o display não fique vazio entre o greeter e o compositor do usuário. Ele continua rodando depois que o greeter termina. O `argvus-start` envia `release` ao socket dele (`/run/argvus-greeter/handoff.sock`) logo antes de iniciar o Hyprland, e o helper só confirma depois de soltar o DRM master. Uma sessão sem greeter envia o mesmo comando, não encontra superfície e nada acontece. Uma falha ao iniciar a superfície é registrada e não interrompe o login. Enquanto ela roda, é possível enviar comandos com `/usr/lib/argvus/greeter/handoff --send theme=<nome>` ou `--send release`. O socket e o `session.log` pré-login do greeter ficam em `/run/argvus-greeter`, que o `systemd-tmpfiles` cria com dono `greeter` e modo 0755: só o greeter grava ali, e a sessão que faz login apenas lê o log e conecta ao socket.

Essa separação ajuda a diagnosticar problemas visuais: um splash de boot ausente é um problema de Plymouth/pacote, uma transição vazia é um problema do overlay ou da prontidão do compositor, e uma falha de autenticação pertence ao greetd/PAM, não à sessão do desktop.

A transição interativa de tema, o reload direto da sessão e o handoff após o login passam explicitamente o tema selecionado para `/usr/lib/argvus/loading-theme/splash`. Reloads diretos usam o modo instantâneo para que o primeiro frame mapeado seja opaco antes do início do trabalho de configuração; transições interativas mantêm o fade visual. O caminho interativo também passa os valores gerados de fundo, foreground e acento; a paleta do splash é apenas um fallback. Confira o binário instalado e o payload do pacote quando alterações da fonte não aparecerem na sessão em execução.
