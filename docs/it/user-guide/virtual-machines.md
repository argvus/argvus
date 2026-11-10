---
title: Macchine virtuali
description: Esegui ARGVUS con hardware grafico virtuale.
---
`argvus start` rileva la virtualizzazione ed esporta le impostazioni di compatibilità per le sessioni interessate, inclusi il rendering software e le alternative per cursore/modificatori quando richiesto dall'ambiente rilevato.

Se una macchina virtuale non riesce ancora a eseguire il rendering, ispezionare l'ambiente della sessione e i log prima di aggiungere una sovrascrittura. Vedi [troubleshooting della grafica](/docs/user-guide/troubleshooting/graphics-and-virtual-machines/).

Quando viene rilevata la virtualizzazione, `argvus start` esporta variabili di compatibilità per la sessione, inclusi il rendering software, il cursore software e le alternative per i modificatori. Questo rende il percorso di accesso normale il primo da provare; non aggiungere una sovrascrittura permanente solo perché il desktop è in esecuzione in una VM.

Se il rilevamento non è sufficiente, utilizzare una sovrascrittura utente di Hyprland come fallback diagnostico:

```sh
mkdir -p ~/.config/argvus/data/hypr
nano ~/.config/argvus/data/hypr/user.lua
```
Aggiungi questo solo quando lo stack grafico della VM lo richiede:

```lua
hl.env("LIBGL_ALWAYS_SOFTWARE", "1")
```
Esci e accedi di nuovo dopo aver modificato l'ambiente della sessione. Conferma il valore attivo con `echo $LIBGL_ALWAYS_SOFTWARE`; il rendering software può rendere il desktop utilizzabile ma potrebbe ridurre le prestazioni.
