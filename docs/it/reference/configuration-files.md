---
title: File di configurazione
description: Configurazione ARGVUS dell'utente, del sistema, predefinita del pacchetto e generata.
---

| Livello | Posizione | Significato |
| --- | --- | --- |
| Sistema | `/etc/argvus/` | Configurazione dell'amministratore per servizi e fornitori. |
| Predefiniti del pacchetto | `/usr/share/argvus/` | Predefiniti dei componenti in sola lettura e risorse di runtime. |
| Utente | `$XDG_CONFIG_HOME/argvus/config.json` | Preferenze portatili canoniche. |
| Dati gestiti | `$XDG_CONFIG_HOME/argvus/data/` | Dati dei componenti, file di compatibilità e metadati interni. |
| Generato | `$XDG_CONFIG_HOME/argvus/data/generated/` | Proiezioni di `config.json`, scritte solo da `argvus-config`. Non modificare mai, non copiare mai altrove come configurazione e non leggere mai come fonte. |
| Runtime | `$XDG_RUNTIME_DIR/` e `$XDG_CACHE_HOME/` | Socket, lock e stato transitorio. |

Il tema attivo, l'accento, lo sfondo personalizzato e lo stato degli effetti sono stati logici dell'utente. Il modello logico gestito è documentato in [Canonical configuration](/docs/reference/argvus-config/)]. I file generati di Hyprland, GTK, Qt6ct, Waybar, Quickshell, Dunst, Hyprlock, Yazi, Superfile, terminale, font e dispositivi rimovibili non devono essere trattati come fonti di verità indipendenti.

`argvus-config` riflette anche il livello del tema in `data/waybar/argvus-taskbar.{jsonc,css}` e `data/waybar/argvus-widget-telemetry.{jsonc,css}`. Queste copie vengono sostituite dai valori predefiniti pacchettizzati in caso di modifica dell'aspetto, quindi i loro blocchi gestiti delimitati vengono riscritti, quindi trattali come derivati piuttosto che come file mantenuti manualmente. La configurazione delle applicazioni native sotto `$XDG_CONFIG_HOME/<app>` è diversa: è un obiettivo adattatore scritto dagli orchestratori dell'aspetto, mai dal proiettore.

## Eccezione alla regola dello scrittore unico

`argvus config` è l'unico autore di `data/generated/`. Esiste un'eccezione per la migrazione: quando viene creata una copia utente di un file impacchettato, un file che una versione più vecchia ha lasciato sotto `data/generated/` viene spostato nella sua posizione utente (`data/`), così l'utente conserva le proprie modifiche. Lo spostamento avviene solo per i file che la versione corrente non genera. Non fare affidamento su questo percorso per i nuovi file: aggiungi un file generato nella proiezione, non qui.
