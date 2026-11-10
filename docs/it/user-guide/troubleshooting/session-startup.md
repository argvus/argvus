---
title: Risoluzione dei problemi di avvio della sessione
description: Diagnosticare i malfunzionamenti di greeter, Hyprland e delle sessioni utente di systemd.
---
Corri:

```sh
argvus sessionctl status
argvus sessionctl logs
systemctl --user status argvus-session.target
```
Controlla il greeter, `argvus start`, Hyprland e l'unità utente pertinente separatamente. Una build del pacchetto riuscita non dimostra che il payload installato o la sovrascrittura utente attiva siano aggiornati.

Usa i controlli in questo ordine:

1. Conferma che il greeter stia usando la configurazione ARGVUS greetd installata.
2. Leggi il registro della sessione e cerca una configurazione Lua di Hyprland mancante o non valida.
3. Controlla la prontezza e lo stato target con `systemctl --user status argvus-session.target`.
4. Ispeziona il componente che fallisce piuttosto che riavviare tutti i servizi contemporaneamente.

Se `argvus-session.target` è inattivo mentre il sovraccarico di caricamento è attivo, ispeziona l'errore di proiezione prima di riavviare i servizi:

```sh
argvus config project
journalctl --user -u argvus-session.target -u argvus-session-prepare.service -b --no-pager
```
`argvus-config` è l'unico scrittore sotto `data/generated/`, quindi `project` è il modo autorevole per riprodurre una generazione durante la diagnostica. L'avvio della sessione lo esegue perché non esiste ancora un piano confermato; il piano che scrive in `${XDG_STATE_HOME:-$HOME/.local/state}/argvus/config-projection.json` è ciò che un successivo `argvus sessionctl reload` applica, senza proiettare di nuovo.

La sessione interrompe il caricamento overlay quando la preparazione fatale o l'avvio del target falliscono. Un errore di un consumatore visivo dovrebbe utilizzare il suo fallback documentato o essere riportato esplicitamente; non deve essere nascosto come una sessione riuscita.

## Login lento

Se `greeter.handoff-takeover-failed` appare senza `handoff.master-acquired`, ispezionare `handoff.master-unavailable card=<name> errno=<n>` per l'errore di acquisizione. Un osservatore aperto mentre un altro compositore è master può restituire errno 13 (`EACCES`) anche dopo che quel compositore rilascia il master. L'helper riapre la stessa GPU per acquisire il master implicitamente; `handoff.master-reopened` registra questo recupero e un nuovo framebuffer viene preparato su quel descrittore. Il greeter annulla l'helper fallito (`greeter.handoff-cancelled`), quindi non può aggiungere un'altra attesa prima che il compositore dell'utente inizi. Una sessione che invia `release` mentre l'acquisizione è in sospeso riceve `handoff.release-before-master`: l'helper chiude i suoi descrittori DRM prima di confermare. Un crash del compositore greeter in Hyprland/Aquamarine è una constatazione separata; annullare l'helper non stabilisce che il problema del driver o del compositore sia stato risolto.

`greeter.vt-switch-returned active=<tty>` registra il VT attivo quando la chiamata switch di logind ritorna. Un timeout di disegno registra anche `active-vt=<tty>` nel suo errore. Questi sono snapshot, non prove di un passaggio DRM completato.

Per un confronto prima/dopo, salva il lavoro e utilizza cinque accessi per ciascuna versione sulla stessa macchina, con lo stesso tema e configurazione del monitor. Prima di ogni logout successivo, copia `/run/argvus-greeter/timing.log`, il registro della sessione e il diario di avvio in una directory di esempio separata sotto `/tmp/argvus/`; il successivo avvio del greeter sostituirà il suo registro dei tempi. Registra l'ID di avvio con ogni esempio e confronta i timestamp solo all'interno di quello stesso avvio. Misura il tempo dall'autenticazione al primo frame di handoff, l'ingresso al session-wrapper, il lancio finale di Hyprland e l'arresto del caricamento del spinner. Confronta il tempo mediano e peggiore dall'autenticazione allo stop dello spinner e verifica visivamente la presenza di frame neri, lampi o uno spinner bloccato. La sola durata delle superfici del desktop non rappresenta il tempo di login end-to-end. Ripeti i test di fallimento e recupero in una sessione usa e getta, senza terminare una sessione contenente lavoro non salvato.

Ogni fase di avvio registra la sua durata nel registro della sessione. Per vedere le fasi dell'ultimo login, dalla più lenta alla più veloce:

```sh
grep timing ~/.local/state/argvus/session.log | tail -30 | sort -k4 -rn
systemd-analyze --user critical-chain argvus-session.target
```
Per sincronizzare il passaggio dal greeter al desktop, leggere i segni del greeter e del passaggio insieme al journal. Tutti usano `CLOCK_MONOTONIC`, quindi si allineano:

```sh
cat /run/argvus-greeter/timing.log
journalctl -b -o short-monotonic --since -2min | grep -E 'greetd|Hyprland|coredump|argvus-greeter-handoff|session-loading'
grep monotonic_ns ~/.local/state/argvus/session.log | tail -5
```

`timing.log` conserva solo l'ultimo accesso: il greeter, la superficie di passaggio DRM (`handoff.master-acquired`, `handoff.output-found`, `handoff.first-frame`, `handoff.release-received`, `handoff.replaced-by-compositor`) e i segni `greeter.*`. Le righe `handoff.crtc` registrano ogni modifica CRTC mentre il passaggio attende il master DRM; una riga con `fb=0` o `mode_valid=false`, o `handoff.output-found ... framebuffer=0`, significa che il compositor del greeter ha spento l'uscita prima che il passaggio la prendesse, quindi il monitor ha dovuto sincronizzarsi di nuovo. `greeter.watchdog-stop-requested` significa che la sessione del greeter è terminata senza attendere il suo compositor, e `greeter.compositor-exit-deferred` che è stato chiesto al compositor di uscire una volta sparita la superficie di passaggio; `greeter.compositor-exit-requested` da solo significa che gli è stato chiesto di uscire immediatamente. Se le righe di greetd e logind nel journal mostrano un intervallo di circa mezzo secondo tra la chiusura della sessione del greeter e la sessione utente all'apertura, il teardown del compositor del greeter è stato eseguito durante il login. Ciò è previsto mentre il marchio mostra `compositor-in-foreground=true`: l'interruzione del terminale del greeter termina il compositor prima dell'uscita differita. Questo accade quando il greeter non è riuscito a lasciare il suo VT prima che il suo compositor uscisse: cerca `greeter.vt-switch-requested` seguito da `greeter.handoff-drawing`; quando il secondo manca, `greeter.handoff-takeover-failed error=<reason>` indica il motivo. `handoff.page-flip` significa che il trasferimento ha sostituito l'ultimo frame del greeter senza un modeset; `handoff.page-flip-failed errno=<n>` seguito da `handoff.modeset` significa che il kernel ha rifiutato il page flip. `handoff.overlay plane=<id>` e poi `handoff.retired` significano che l'overlay ha nascosto i primi frame di Hyprland fino a quando lo spinner della sessione non è apparso sullo schermo; `handoff.overlay-unavailable` significa che il CRTC non aveva piani overlay liberi, e `handoff.retire-timeout` significa che lo spinner della sessione non è mai stato inviato `retire`. Lo spinner della sessione (`argvus-greeter-handoff`) scrive `first layer-shell frame callback` e `stop requested` sul registro.

È previsto un `project.theme-switch` o un ingresso `prepare.startup-theme` diverso da zero dopo un cambio di aspetto, file di tema modificati, un nuovo accento o la migrazione di un vecchio timestamp di avvio. Reinstallare file identici preserva la cache di avvio basata sul contenuto. Se appare ad ogni accesso, i file del tema materializzati sono mancanti; eseguire `argvus theme-switch <theme>` una volta.

Il registro della sessione è conservato sotto `$XDG_STATE_HOME/argvus/session.log` (normalmente `~/.local/state/argvus/session.log`). Dopo aver cambiato un override utente di Hyprland, eseguire il logout e il login affinché il punto di accesso della sessione lo validi e lo selezioni di nuovo. Vedere [Graphical session](/docs/argvus-session/graphical-session/) per il flusso di avvio normale.
