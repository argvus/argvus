---
title: Introduzione
description: Cos'è ARGVUS, come funziona il suo desktop e dove si collocano i suoi componenti.
---
ARGVUS è un ambiente desktop modulare per Hyprland e Wayland su Arch Linux. È una raccolta integrata di pacchetti piuttosto che un'unica applicazione monolitica: la sessione, il compositore, le superfici della shell, le applicazioni di impostazioni, i fornitori e le risorse visive lavorano insieme attraverso uno stato condiviso e contratti di runtime espliciti.

Questa pagina è la mappa della documentazione. Spiega l'intero desktop a un livello generale; le pagine collegate contengono i dettagli operativi e di implementazione.

## Come funziona ARGVUS

La sessione normale segue questo percorso:

```text
graphical login or TTY
        ↓
argvus-session → argvus start
        ↓
Hyprland + argvus-hyprland
        ↓
argvus-session.target (systemd --user)
        ↓
taskbar · control panel · notifications · wallpaper · idle · clipboard
```
`argvus-session` gestisce il ciclo di vita della sessione e la configurazione dell'ambiente. Non implementa tutte le funzionalità del desktop: ogni componente mantiene la proprietà dei propri comandi, configurazioni e payload dei servizi.

Per l'installazione e l'avvio, vedere [Installation](/docs/getting-started/installation/) e [Sessions](/docs/user-guide/sessions/). Per la gestione dei servizi, vedere il [runtime lifecycle](/docs/developer-guide/architecture/runtime-lifecycle/) per sviluppatori.

## Livelli principali

### Punti di accesso e infrastruttura

Il pacchetto `argvus` fornisce il dispatcher `/usr/bin/argvus` e il punto di ingresso principale del pacchetto. `argvus-session` fornisce punti di accesso grafici e TTY, il target utente e `argvus sessionctl`. Le funzionalità TUI condivise e di localizzazione provengono dallo spazio di lavoro `argvus` (i crate `argvus-tui` e `argvus-i18n`) e dal comando `argvus i18n`.

### Compositore e shell del desktop

`argvus-hyprland` fornisce la configurazione di Hyprland e le combinazioni di tasti. `argvus-waybar` fornisce il binario Waybar patchato, mentre `argvus-taskbar` possiede la configurazione della barra delle applicazioni e le azioni. `argvus-control-panel` fornisce il pannello Quickshell, e `argvus widget-telemetry` fornisce widget di sistema opzionali, in-process in `argvus`.

Vedi [Desktop](/docs/user-guide/desktop/) per l'uso e [shell surfaces](/docs/developer-guide/subsystems/shell-surfaces/) per la proprietà dell'implementazione.

### Impostazioni e applicazioni

`argvus control-center` è l'applicazione per le impostazioni orientata alla tastiera, inclusa come aiuto in `argvus`. Gli strumenti calendario e dispositivi rimovibili sono helper GTK 4 in `argvus`; il monitor di sistema e il terminale sono sottocomandi di `argvus`; il launcher è `argvus-launcher`. Tutti sono integrati con la shell e il dispatcher.

Vedi [Applications](/docs/user-guide/applications/) e il [command reference](/docs/reference/command-line/).

### Provider e integrazioni

Display, rete, Bluetooth, alimentazione, notifiche, firewall e archiviazione rimovibile sono forniti da progetti specifici. Il Centro di Controllo e il Pannello di Controllo consumano questi provider; non li sostituiscono.

Login, blocco e elementi visivi di avvio sono anch'essi separati: `argvus-greeter` gestisce greetd, `argvus-lock` gestisce Hyprlock, il helper di handoff incluso in `argvus` disegna l'animazione di login, il sovrapposto di caricamento gestisce le transizioni di tema, e `argvus-boot-splash` gestisce la schermata di avvio.

Vedi [Hardware](/docs/user-guide/hardware/), [Privacy e sicurezza](/docs/user-guide/privacy-and-security/) e il [component reference per sviluppatori](/docs/developer-guide/).

## Stato condiviso dell'aspetto

Preferenze di tema, accento, sfondo, font ed effetti sono memorizzate come stato logico sotto `$XDG_CONFIG_HOME/argvus` (normalmente `~/.config/argvus`). `argvus-config` è l'unico componente che scrive su `data/generated/`, dove proietta i file dei consumatori per Hyprland, GTK, Qt6ct, Waybar, Quickshell, Dunst, notifiche, la schermata di blocco, Yazi, Superfile e terminali. Rofi è l'eccezione: `argvus launcher` distribuisce la sua configurazione e i temi come file semplici, e le copie modificate dall'utente risiedono sotto `data/rofi/`. Gli strumenti di supporto all'aspetto confermano il cambiamento di stato e poi riconciliano le applicazioni esterne; non scrivono mai l'albero generato.

La directory generata non è la fonte della verità. Vedi [Appearance](/docs/user-guide/appearance/) e [configuration and state](/docs/developer-guide/architecture/configuration-and-state/).

Il sistema visivo controlla anche il layout. La modalità Sticky è compatta e quadrata; la modalità Spaced utilizza spazi più ampi tra le finestre, angoli arrotondati e margini più larghi attorno alla barra delle applicazioni e alla shell. La barra delle applicazioni predefinita è sul bordo superiore, ma supporta il posizionamento in alto/ in basso tramite lo stato di layout condiviso. Gli effetti controllano trasparenza, sfocatura, ombre e animazioni attraverso il compositor e le superfici del desktop.

Consulta la [tabella dei temi](/docs/user-guide/appearance/) per le famiglie e i colori attuali, [effetti](/docs/argvus-appearance/effects/) per il contratto degli effetti condivisi, e [taskbar](/docs/argvus-taskbar/taskbar/) per il posizionamento della barra e delle finestre.

## Cosa è e cosa non è ARGVUS

ARGVUS è il livello dell'ambiente desktop sopra Arch Linux. Arch Linux fornisce il sistema operativo e i pacchetti; Hyprland fornisce la composizione Wayland; ARGVUS coordina la sessione grafica, le superfici della shell, le impostazioni, l'aspetto, i servizi desktop e le applicazioni integrate.

È quindi più di una configurazione di Hyprland o di una raccolta di dotfile personali. ARGVUS definisce un punto di ingresso supportato per la sessione, gestisce il ciclo di vita dei suoi servizi desktop, fornisce un modello comune per impostazioni e aspetto, e collega componenti focalizzati come networking, display, alimentazione, notifiche, account e archiviazione rimovibile.

ARGVUS non deve sostituire ogni applicazione. Integra applicazioni selezionate upstream quando questa è la scelta più affidabile, mantenendo al contempo il comportamento e la configurazione coerenti su tutto il desktop.

Il nome combina due idee. *ARGV*, da *argument vector*, è l'elenco degli argomenti che un programma riceve, e riflette l'obiettivo di far interagire il desktop principalmente tramite tastiera, comandi e scorciatoie. *US*, da *User System*, riflette l'obiettivo di un desktop centrato sull'utente. Per la spiegazione completa e l'audio della pronuncia, vedere [About ARGVUS](/about/).

Per i confini dell'implementazione e la proprietà dei pacchetti, vedere la panoramica dell'architettura dello sviluppatore [architecture overview](/docs/developer-guide/architecture/overview/) e i [ruoli del repository](/docs/developer-guide/architecture/repository-roles/).

## Continuare per obiettivo

- [Leggere le FAQ](/docs/user-guide/faq/).
- [Installare ARGVUS](/docs/getting-started/installation/).
- [Avviare una sessione grafica o TTY](/docs/user-guide/sessions/).
- [Configurare temi e sfondi](/docs/user-guide/appearance/).
- [Usare la barra delle applicazioni e il Pannello di Controllo](/docs/user-guide/desktop/).
- [Comprendere comandi, percorsi e servizi](/docs/reference/).
- [Studiare l'architettura](/docs/developer-guide/architecture/overview/).
