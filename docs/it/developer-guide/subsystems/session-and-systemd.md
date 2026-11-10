---
title: Sessione e systemd
description: L'obiettivo della sessione ARGVUS e i suoi servizi utente.
---
Le definizioni delle unità sono pacchettizzate da `argvus-session` sotto `/usr/share/argvus/session/config/systemd/user/`. Il gestore delle sessioni possiede il ciclo di vita e l'importazione dell'ambiente. I repository dei componenti possiedono i comandi eseguiti dalle loro unità.

Usare `argvus sessionctl` invece di avviare manualmente un componente durante la diagnosi della sessione integrata.
