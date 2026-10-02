---
title: Greeter, lock e splash
description: Visuais separados de login e inicialização.
slug: pt/0.4.0/docs/developer-guide/subsystems/greeter-lock-and-splash
---

`argvus-greeter` fornece a interface greetd; `argvus-theme-splash` fornece o overlay GTK4 da sessão; `argvus-splash` fornece o tema Plymouth de boot; `argvus-lock` fornece Hyprlock. Eles têm ciclos de vida e payloads distintos.

O greeter roda antes da sessão autenticada e usa a fronteira IPC/PAM do greetd. O overlay de carregamento inicia quando o novo socket Wayland fica disponível e termina quando o Hyprland informa que está pronto. O tema Plymouth pertence ao boot e não é a tela de login. O lock é usado depois do login e não substitui o greeter.

Essa separação ajuda a diagnosticar problemas visuais: um splash de boot ausente é um problema de Plymouth/pacote, uma transição vazia é um problema do overlay ou da prontidão do compositor, e uma falha de autenticação pertence ao greetd/PAM, não à sessão do desktop.

A transição interativa de tema, o reload direto da sessão e o handoff após o login passam explicitamente o tema selecionado para `/usr/lib/argvus/theme-splash/splash`. Reloads diretos usam o modo instantâneo para que o primeiro frame mapeado seja opaco antes do início do trabalho de configuração; transições interativas mantêm o fade visual. O caminho interativo também passa os valores gerados de fundo, foreground e acento; a paleta do splash é apenas um fallback. Confira o binário instalado e o payload do pacote quando alterações da fonte não aparecerem na sessão em execução.
