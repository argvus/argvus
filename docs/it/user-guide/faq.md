---
title: Domande frequenti
description: Domande comuni su ARGVUS, la sua architettura e l'uso quotidiano.
---
Questa pagina risponde alle domande comuni su cos'è ARGVUS, come viene distribuito e dove configurarlo. Per procedure dettagliate, segui i link in ogni risposta.

## Cos'è ARGVUS?

ARGVUS è un ambiente desktop modulare per Wayland e Arch Linux, costruito intorno a Hyprland. Non è un ambiente desktop indipendente da Hyprland; è un'esperienza desktop assemblata sopra Hyprland e adattata a Arch Linux. ARGVUS coordina la sessione grafica, le superfici della shell, le applicazioni delle impostazioni, l'aspetto, i servizi desktop e gli strumenti integrati attraverso una raccolta di pacchetti.

È più di una configurazione di Hyprland o di una raccolta di dotfile personali: fornisce un punto di ingresso alla sessione supportato, configurazione e stato dell'aspetto condivisi, e proprietà definita per componenti come la barra delle applicazioni, il Centro di Controllo, le notifiche, la rete e i display. Vedi l'[introduzione](/docs/introduction/) per una panoramica di come i pezzi si incastrano tra loro.

## ARGVUS è gratuito?

Sì. ARGVUS è gratuito da scaricare e usare, e i pacchetti principali del progetto sono distribuiti come software libero sotto la [Licenza Pubblica Generale GNU versione 3](https://www.gnu.org/licenses/gpl-3.0.html). Questo dà agli utenti la libertà di eseguire, studiare, modificare e ridistribuire il software coperto secondo i termini della licenza.

ARGVUS viene distribuito come pacchetti Arch Linux firmati. Il software stesso non ha un prezzo di acquisto, ma il download dei pacchetti può comunque comportare normali costi di internet o di hosting. Componenti e applicazioni forniti da terzi possono avere le proprie licenze.

## ARGVUS è una distribuzione Linux?

No. ARGVUS è il livello dell'ambiente desktop installato sopra Arch Linux. Arch Linux fornisce il sistema operativo, il kernel, il gestore di pacchetti e i pacchetti di base; Hyprland fornisce la composizione Wayland; ARGVUS coordina la sessione desktop e i suoi componenti integrati.

Vedi [installation](/docs/getting-started/installation/) per il modello di installazione supportato e [What ARGVUS is—and is not](/docs/introduction/#what-argvus-isand-is-not) per la distinzione architetturale.

## Perché un ambiente desktop invece di una distribuzione?

Mantenere ARGVUS come ambiente desktop offre agli utenti maggiore controllo sul resto del loro sistema. Una distribuzione normalmente sceglie e mantiene una baseline completa, inclusi il kernel, la selezione dei pacchetti, i repository, le impostazioni predefinite e le politiche di sistema. ARGVUS si concentra sull'esperienza desktop lasciando però all'utente e all'installazione sottostante di Arch Linux la gestione di queste decisioni più ampie.

Questo modello mantiene anche il progetto modulare: la gestione delle sessioni, la configurazione di Hyprland, l'aspetto grafico, la barra delle applicazioni, le impostazioni e i provider possono essere sviluppati e pacchettizzati separatamente pur funzionando insieme come un unico desktop.

## Quali sistemi sono supportati?

La base attualmente supportata è Arch Linux. ARGVUS è pacchettizzato per Arch Linux e dipende dai componenti di Wayland, Hyprland e del sistema descritti nei [requisiti di installazione](/docs/getting-started/installation/requirements/).

Altre distribuzioni possono fornire alcuni degli stessi componenti upstream, ma ciò non significa che l'insieme completo di pacchetti ARGVUS o l'integrazione della sessione siano supportati lì.

## Come installo ARGVUS?

Segui la [guida all'installazione](/docs/getting-started/installation/). Spiega i requisiti, come configurare il repository dei pacchetti ARGVUS firmati, come installare l'insieme di pacchetti e come avviare la prima sessione.

ARGVUS viene distribuito come un insieme coordinato di pacchetti Arch. Il pacchetto `argvus` è l'obiettivo principale dell'installazione e include i componenti desktop che coordina.

## Come posso modificare le impostazioni di ARGVUS?

Usa **Centro di Controllo** per la configurazione persistente del desktop, inclusi temi, sfondi, effetti, layout, font, tastiera, input, lingua, regione e applicazioni predefinite. I modi più efficienti per aprirlo sono la scorciatoia `SUPER + Alt + C`, o il launcher con `SUPER + D`: digita **Centro di Controllo ARGVUS**. Puoi anche digitare `argvus` per elencare le applicazioni ARGVUS.

Come opzione aggiuntiva, puoi aprirlo da un terminale con:

```sh
argvus --control-center
```
Puoi anche aprire un'area focalizzata con `argvus control-center`, ad esempio `argvus control-center appearance themes`. Esegui `argvus control-center --help` per vedere le rotte disponibili nella versione installata.

Utilizza **Pannello di Controllo** per lo stato corrente e le azioni frequenti come volume, luminosità, rete, notifiche, alimentazione e controllo sessione. Non è un secondo database di configurazione. Vedi [Dove configurare le cose](/docs/user-guide/where-to-configure/) e [Centro di Controllo](/docs/components/argvus-control-center/) per la distinzione.

## Dove sono memorizzate le impostazioni ARGVUS?

La configurazione di proprietà dell'utente è memorizzata sotto `$XDG_CONFIG_HOME/argvus` (normalmente `~/.config/argvus`). La configurazione logica canonica è mantenuta lì, mentre i file derivati per i consumatori come Hyprland, Waybar e il widget di telemetria, GTK, Qt6ct, Quickshell, Dunst, Hyprlock, Yazi, Superfile, terminali, font, effetti, calendari, input, tastiera, alimentazione e dispositivi rimovibili risiedono sotto `~/.config/argvus/data/generated/`. Quell'albero è scritto solo da `argvus-config` e ricostruito dalla configurazione canonica; `argvus-appearance` conferma le modifiche dell'aspetto e poi riconcilia solo i consumatori esterni che non possono leggere la configurazione canonica da soli. Rofi è l'eccezione: `argvus launcher` confeziona la sua configurazione e i temi, e le copie modificate dall'utente risiedono sotto `~/.config/argvus/data/rofi/`.

Non modificare i file generati come metodo di configurazione permanente. Utilizza il Control Center o il percorso di override nativo documentato per il componente che desideri personalizzare. Vedi [configuration files](/docs/reference/configuration-files/) e [configuration and state](/docs/developer-guide/architecture/configuration-and-state/).

## Cosa dovrei fare se una modifica non viene applicata?

Prima conferma che la modifica sia stata salvata o applicata nella pagina di appartenenza. Poi verifica se il componente interessato necessita di un riavvio della sessione o del servizio. Una modifica confermata viene proiettata nei file del consumatore tramite `argvus-config`, ma proiettarla non garantisce che ogni consumatore in esecuzione l'abbia ricaricata. `argvus sessionctl reload` applica una generazione già confermata dal manifesto di proiezione invece di proiettarla nuovamente, e salta il riavvio grafico quando lo stato canonico non è cambiato.

Utilizza la [guida alla risoluzione dei problemi](/docs/user-guide/troubleshooting/) pertinente per l'installazione, l'avvio della sessione, la grafica, i display, Hyprland, i temi e gli sfondi. Quando segnali un problema, includi il componente, l'impostazione modificata, il comando o la pagina utilizzata e l'output del comando di stato o diagnostico pertinente.

## Dove posso saperne di più?

- [Prima configurazione](/docs/user-guide/getting-started/) per il flusso consigliato della prima sessione.
- [Guida utente](/docs/user-guide/) per istruzioni specifiche sulle funzionalità.
- [Riferimento](/docs/reference/) per comandi, percorsi, servizi e contratti di configurazione.
- [Guida per sviluppatori](/docs/developer-guide/) per l'architettura, la proprietà dei pacchetti e i flussi di lavoro per i contributi.
