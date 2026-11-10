---
title: Monitor di sistema
description: Apri il monitor di sistema ARGVUS.
---
Apri il monitor configurato con:

```sh
argvus --system-monitor
```
Il sottocomando `argvus system-monitor` fornisce il punto di ingresso nativo e l'integrazione con btop. La telemetria mostrata nella barra delle applicazioni è una funzione separata fornita da `argvus widget-telemetry`. Cliccando sui suoi moduli CPU, Memoria e GPU si apre btop all'interno di un pannello TUI di ARGVUS con un'intestazione brandizzata e un footer `Esc Close`. La modalità GPU mostra fino a sei riquadri GPU e nessun riquadro per CPU, memoria, rete o processi. Premere `q` o `Esc` per chiudere il pannello. La scheda inferiore Kitty mostra il suggerimento localizzato `Press q to close`. Chiamate standalone `argvus system-monitor --cpu`, `--mem` e `--gpu` avviano btop direttamente con i riquadri selezionati.

Il pannello GPU dipende dalle capacità della build di btop installata e dai fornitori di dispositivi del sistema. Il monitoraggio NVIDIA richiede il supporto GPU in btop, la NVIDIA Management Library dal driver NVIDIA installato e l'accesso al dispositivo. Il monitoraggio AMD utilizza ROCm SMI, che è opzionale e potrebbe non essere installato. I dati della GPU Intel vengono letti da sysfs e dipendono dall'accesso dell'utente a tali interfacce; btop 1.4.7 riporta l'utilizzo, il consumo energetico e il clock di Intel, ma non fornisce la temperatura di Intel. I driver NVIDIA nouveau e altri possono esporre una temperatura della GPU nella barra delle applicazioni tramite hwmon collegato al dispositivo, mentre il supporto del provider btop dipende dal driver e dalle librerie disponibili. ARGVUS non modifica i permessi del dispositivo né esegue btop con privilegi elevati. Se nessuna GPU è esposta, o btop non genera una casella GPU entro tre secondi perché il supporto o un provider non sono disponibili, il pannello mostra un messaggio di non disponibilità localizzato.

Il monitor legge prima il tema e l'accento selezionati da `$XDG_CONFIG_HOME/argvus/config.json`. I suoi file btop generati rimangono nello stato della cache derivata; `.active-theme` e `.accent-color` sono fallback di compatibilità. Poiché btop mantiene il proprio file del tema, il monitor viene riconciliato dalla transazione del tema (`argvus theme-switch`) come un adattatore esterno dopo che la modifica canonica è stata commessa; non fa parte dell'albero generato.

## Contratto di runtime

- Il tema e l'accento provengono dalla configurazione canonica (` and ` and `). The files `). The files ~/.config/argvus/data/.active-theme e `.accent-color` sono fallback.
- La configurazione generata di btop e i temi sono scritti in `$XDG_CACHE_HOME/argvus/argvus-system-monitor`. Sono stato derivato; non modificarli.
- I valori predefiniti sono installati sotto `/usr/share/argvus-system-monitor/btop`.
- Le scorciatoie dei moduli (`--mem`, `--cpu`, `--net`, `--proc`) sono letti solo come primo argomento. Passare `-c`/`--config` o `--themes-dir` di btop stesso esegue btop con quei percorsi e non scrive configurazioni.
- `--panel [--cpu|--mem|--gpu]` incorpora btop nella shell TUI di ARGVUS. Questa modalità è utilizzata dalla barra delle applicazioni; senza `--panel`, il monitor continua a sostituirsi con btop.
