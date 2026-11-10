---
title: Prima configurazione
description: Un percorso pratico per la configurazione di ARGVUS al primo avvio.
---
Dopo la prima sessione grafica, non è necessario configurare ogni parte di ARGVUS. Inizia con le impostazioni che influenzano come si percepisce il desktop, quindi configura l'hardware e i servizi secondo necessità.

## Un ordine utile

1. Leggi la [panoramica del desktop](/docs/user-guide/desktop/) per identificare finestre, spazi di lavoro, la barra delle applicazioni, il Pannello di Controllo e il launcher.
2. Apri il [Centro di Controllo](/docs/components/argvus-control-center/) e scegli un tema, una modalità, un accento e uno sfondo in **Aspetto**.
3. Regola [finestre e layout](/docs/argvus-hyprland/windows-and-layout/) se gli spazi, i bordi o la spaziatura della barra delle applicazioni predefiniti non si adattano al tuo schermo.
4. Configura la [barra delle applicazioni](/docs/argvus-taskbar/taskbar/) e decidi quali schede del Pannello di Controllo vuoi rendere visibili.
5. Rivedi le [scorciatoie da tastiera](/docs/argvus-hyprland/keyboard-shortcuts/) e modifica solo le combinazioni che usi effettivamente.
6. Configura [mouse e touchpad](/docs/argvus-hyprland/input/), display, rete e alimentazione dalle relative pagine delle impostazioni.

I primi quattro passaggi sono personalizzazioni opzionali. I passaggi rimanenti sono normalmente necessari solo quando l'hardware, la lingua o il flusso di lavoro lo richiedono.

## Due posti per cambiare le impostazioni

Usa **Centro di Controllo** per le impostazioni che descrivono come ARGVUS dovrebbe essere configurato: aspetto, layout, caratteri, input, scorciatoie, lingua, regione e applicazioni predefinite. Usa **Pannello di Controllo** per lo stato e le azioni frequenti della sessione come volume, luminosità, stato della rete, notifiche, controllo dell'alimentazione e della sessione.

Vedi [Centro di Controllo vs Pannello di Controllo](/docs/components/argvus-control-center/) per la distinzione completa e [Dove configurare le cose](/docs/user-guide/where-to-configure/) per un indice rapido.

## Modifiche, persistenza e recupero

La maggior parte delle modifiche del Centro di Controllo vengono applicate mentre la sessione è in esecuzione e salvate come stato utente ARGVUS. Le modifiche all'aspetto vengono impegnate nella configurazione canonica, e `argvus-config` proietta i file consumati da Hyprland e dalla shell da essa. Un toggle del Pannello di Controllo può essere un'azione diretta o un cambiamento dello stato della sessione invece di una configurazione permanente.

Quando una pagina offre **Ripristina impostazioni predefinite**, utilizza quell'azione della pagina piuttosto che eliminare i file generati. Aspetto ha azioni di ripristino separate per accenti, sfondi e valori di layout; la pagina dei tasti di scelta rapida ha la propria azione di ripristino. Non esiste un singolo pulsante universale di ripristino per tutte le impostazioni del desktop.

## Prossimi passi

- [Aspetto](/docs/user-guide/appearance/) — temi, sfondi, effetti e layout.
- [Layout del desktop](/docs/argvus-hyprland/windows-and-layout/) — capire spazi, bordi e spazio del pannello.
- [Dispositivi di input](/docs/argvus-hyprland/input/) — controlli del mouse e del touchpad.
- [Risoluzione dei problemi di tema e sfondo](/docs/user-guide/troubleshooting/theme-and-wallpaper/) — recupero quando una modifica visiva non è quella che ti aspettavi.
