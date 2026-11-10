---
title: Ciclo di vita del runtime
description: Avvio della sessione e proprietà del servizio utente.
---
`argvus-session` importa l'ambiente, avvia Hyprland tramite `argvus start` e avvia `argvus-session.target` dopo che il compositor è pronto. Il target richiama i servizi di shell, notifica, sfondo, inattività e clipboard. `argvus sessionctl` coordina avvio, arresto, ricarica, riavvio, stato e log.

La sessione arresta il target quando Hyprland termina. I singoli progetti rimangono responsabili dei propri binari e della configurazione.

## Avvio

Il punto di accesso grafico è `/usr/share/wayland-sessions/argvus.desktop`, che invoca `argvus session`. La sessione importa l'ambiente di configurazione ARGVUS e XDG, lo sincronizza con `systemd --user` e l'attivazione D-Bus, e delega l'avvio del compositor a `argvus start`.

`argvus start` convalida la configurazione di Hyprland, applica il fallback pacchettizzato quando viene trovato un file generato di default o una configurazione utente non valida, rileva la compatibilità con la virtualizzazione e avvia Hyprland. Il bridge di prontezza del compositor chiama quindi `argvus sessionctl ready`, che avvia il target ARGVUS con l'ambiente corrente di Wayland e Hyprland.

Il target utilizza un'unità di preparazione one-shot prima di avviare componenti a lunga esecuzione. La preparazione applica la configurazione generata, lo stato del tema e dell'accento, lo stato dei monitor, l'integrazione delle applicazioni e l'ambiente della sessione. I servizi persistenti sono di proprietà di `systemd --user`, non dei processi background ad hoc.

`argvus start` mette in coda `argvus-config.service` (`--no-block`) subito prima di lanciare Hyprland: la proiezione non ha bisogno di un compositore, quindi si sovrappone all'avvio del compositore. `argvus sessionctl ready` poi avvia il target, che trova quella unità attiva o aspetta il suo job, quindi la configurazione canonica è ancora proiettata prima che la preparazione venga eseguita. Una volta che il target è avviato, `sessionctl ready` aspetta fino a 1,5 secondi per la carta da parati (un livello di sfondo) e la barra delle applicazioni per essere mappati prima di rimuovere il caricamento sovrapposto, perché un servizio semplice viene considerato avviato prima che venga disegnato; una superficie la cui unità non è in esecuzione non viene attesa, e l'attesa viene registrata come `session.desktop-surfaces`, con `session.wallpaper-surface` e `session.taskbar-surface` per il tempo impiegato da ciascuna superficie. Un errore fatale della proiezione o dell'avvio del target viene registrato e ferma il caricamento sovrapposto indipendente, prevenendo uno spinner apparentemente infinito. Opzionale i consumatori visivi devono risolvere un fallback o segnalare un errore degradato senza sostituire l'obiettivo della sessione.

### Costo del login

Un login senza modifiche alla configurazione non ripete lavori costosi:

- La proiezione esegue il cambio tema e l'adattatore degli effetti solo quando le sezioni di aspetto o effetti sono cambiate, o quando le loro uscite reali (le copie utente materializzate e `data/state/effects/<theme>.conf`) sono assenti.
- La preparazione applica il tema di avvio, e l'accento sopra di esso, solo quando il timbro in `$XDG_CACHE_HOME/argvus/session/theme-startup.stamp` non corrisponde più ai file del tema confezionati e al marcatore di accento. Il timbro fingerprinta i contenuti dei file, quindi reinstallare file identici non lo invalida. Gli input del tema modificati o un nuovo accento lo invalidano una volta. La migrazione di un timbro basato su timestamp più vecchio genera anche una ricostruzione una volta. Quando il tema viene riapplicato, la preparazione ricarica anche Hyprland, che era avviato sui file precedenti.
- Le impostazioni GTK (`gsettings`) vengono scritte solo quando il tema di avvio è stato riapplicato. Tra un accesso e l'altro, dconf le mantiene, quindi un accesso stabile non le esegue di nuovo.
- `argvus-dunst.service` è `Type=dbus`: viene conteggiato come avviato una volta che Dunst possiede `org.freedesktop.Notifications`, quindi il target della sessione non aspetta che lo stato di pausa sia ripristinato.
- La preparazione esegue `spaces-switch.sh` e `borders-switch.sh` (`--apply-static`) fianco a fianco: scrivono file disgiunti (i margini JSONC e la geometria effettiva delle barre, e i CSS delle barre e il tema Rofi).
- La proiezione riscrive un file generato solo quando il suo contenuto cambia. I file invariati vengono lasciati intatti, quindi un accesso stabile non li sincronizza di nuovo.
- La fase di visualizzazione della preparazione (`argvus config display session-prepare`) si applica solo a DPMS e uscite disabilitate: non proietta nuovamente né ricarica il compositor.
- `argvus-session-reload.service` esegue `argvus sessionctl reload-desktop --login`, che ricarica Hyprland solo quando il manifesto della proiezione elenca `hyprland` tra i consumatori interessati. I componenti del desktop non lo aspettano.

Il caricatore di sessione (l'assistente per il passaggio in modalità Wayland) è mappato dal passaggio del compositore fino a quando `argvus sessionctl ready` ha avviato il target, quindi un unico caricatore copre l'intero avvio. Non ha `PartOf=`, perché ricreare il target a un nuovo login non deve arrestarlo; `sessionctl` lo ferma esplicitamente, e `RuntimeMaxSec=30` lo termina se non arriva alcuno stop.

Ogni fase di preparazione, ricarica e adattatore di proiezione aggiunge la propria durata al registro della sessione come `<epoch-ms> timing <step> <N>ms`; vedere [risoluzione dei problemi all'avvio della sessione](/docs/user-guide/troubleshooting/session-startup/).

La risoluzione dei percorsi, la preparazione della sessione e il piano di ricarica di Hyprland sono gestiti da Rust.
`argvus config session-prepare` e `argvus config session-reload-config` eseguono i passaggi di avvio e ricarica; lo script legacy `hypr-init.sh` è stato rimosso.

## Servizi e spegnimento

Il target possiede comunemente `argvus-control-panel.service`, `argvus-taskbar.service`, `argvus-widget-telemetry.service` quando abilitato, `argvus-wallpaper.service`, `argvus-dunst.service`, `argvus-hypridle.service`, watcher degli appunti, notifiche del layout della tastiera e integrazione con PolicyKit. L'overlay di caricamento della sessione viene avviato da `argvus start`, non dal target. Un fallimento in un servizio opzionale non sostituisce l'intera sessione.

I servizi che necessitano di una connessione client Wayland, compreso il Pannello di Controllo e il gestore di inattività, attendono un runtime Hyprland utilizzabile prima di avviare il loro client. Ciò evita di avviare Quickshell o hypridle su un socket obsoleto durante il passaggio del compositore; se Hyprland viene sostituito, systemd può avviare nuovamente il componente dopo che il nuovo runtime è disponibile.

Un cambio di tema serializza il suo commit, la costruzione, la proiezione e i ricaricamenti dei consumer come un'unica operazione del ciclo di vita sotto il blocco del tema. Non ricarica `argvus-config.service` né riavvia i servizi: la proiezione che esegue si limita a scrivere file e lascia il tema al cambio mentre il blocco è attivo, e i consumer in esecuzione si ricaricano in loco (vedi [Aspetto e temi](/docs/developer-guide/subsystems/appearance-and-themes/)]).

Quando Hyprland esce, `argvus start` interrompe `argvus-session.target` e i servizi ad esso collegati. Ciò impedisce ai processi della barra delle applicazioni, della shell, delle notifiche, dell'inattività e degli appunti di sopravvivere al logout.

## Ricarica

```sh
argvus sessionctl reload
```

Il reload reimporta l'ambiente grafico, quindi **applica** una generazione confermata quando un comando di dominio precedente come `apply-theme`, `accent`, `set`/`patch` o una modifica a un blocco di telemetria widget ha già previsto la modifica. In tal caso ripristina i consumatori elencati nel piano del manifesto e non richiama di nuovo `argvus config project`, perché riproiettare ricostruirebbe un albero identico e scarterebbe il piano. Proiezione, migrazione e `ensure` vengono eseguite solo quando non esiste un piano pendente valido — all'avvio della sessione, o dopo che `argvus config recover` deve ripristinare il documento precedente. Un no-op pulito è permesso solo quando le proiezioni e il runtime gestito sono entrambi aggiornati. Se lo stato canonico non è cambiato ma un socket del compositore, un servizio o un consumatore è obsoleto, il reload lo riconcilia invece di ritornare anticipatamente. La normale distribuzione copre Hyprland, la barra delle applicazioni, la telemetria dei widget, il Pannello di Controllo, le notifiche, lo sfondo e lo stato inattivo. secondo il manifesto della proiezione, mentre i consumatori opzionali non modificati rimangono intatti. Ogni file sotto `data/generated/` è scritto da `argvus-config` a questo punto, quindi la sessione non ri-renderizza mai un tema di per sé. Il ricaricamento esegue quindi `theme-adapters.sh`, che riconcilia i consumatori che non possono leggere da soli il documento canonico o la proiezione pubblicata. Quegli adattatori leggono solo la generazione impegnata: scrivono i propri file e cache dei consumatori, e non scrivono mai il documento canonico o `data/generated/`, quindi un'unica azione dell'utente comporta ancora una mutazione, una proiezione, una pubblicazione e un ricaricamento. Usare `argvus sessionctl restart <component>` solo quando un singolo servizio di proprietà necessita di un riavvio esplicitamente isolato. Ispezionare il [riferimento servizi ](/docs/reference/services/) per i nomi esatti delle unità.

Le operazioni di proiezione e ricarica sono serializzate con un lock utente-runtime. Per una traccia temporanea dei tempi, imposta `ARGVUS_SESSION_PERF=1` prima di invocare la ricarica e ispeziona `~/.local/state/argvus/session.log`; la traccia registra i confini delle fasi senza modificare il normale comportamento della ricarica.
