---
title: Contribuendo
description: Confini di contribuzione per progetti e documentazione ARGVUS.
---
Traccia una funzionalità attraverso tutti i repository che la consumano prima di modificarla. Conserva i contratti di stato condiviso, la proprietà del pacchetto, la proprietà del servizio e entrambi i cataloghi di localizzazione.

`argvus-config` è l'unico scrittore di `data/generated/`. Aggiungi o modifica un file di consumo lì, in `crates/argvus-config/src/project.rs` e nei suoi sottomoduli sotto `project/` (file Hyprland in `project/hyprland_projection.rs`), non in un helper della shell. Se un'applicazione consumer non può leggere `config.json`, aggiungi un adattatore nel pacchetto proprietario e chiamalo dall'orchestratore dopo il commit canonico. Elimina `data/generated/`, esegui `argvus config apply` e conferma che l'albero sia ricostruito byte per byte da `config.json`. Valida separatamente il codice sorgente, il payload del pacchetto e il comportamento della sessione live; una build del sorgente da sola non prova che una sessione Wayland installata utilizzi i nuovi file.
