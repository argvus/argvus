---
title: Prima sessione
description: Avvia ARGVUS dopo l'installazione.
---
Per un login grafico, configura l'integrazione greetd inclusa con `argvus greeter-setup --help` e utilizza il flusso del display-manager descritto in [Sessions](/docs/user-guide/sessions/)].

Per una sessione TTY, accedi a un terminale virtuale ed esegui:

```sh
argvus --start-desktop
```
Dopo il login, usa `argvus sessionctl status` per ispezionare l'obiettivo utente ARGVUS. Se il compositor o la shell non si avviano, segui [session troubleshooting](/docs/user-guide/troubleshooting/session-startup/).
