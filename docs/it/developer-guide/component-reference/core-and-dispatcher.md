---
title: Nucleo e dispatcher
description: L'area di lavoro argvus, il suo dispatcher e i crate Rust condivisi.
---
`argvus` è l'ambiente Cargo in `de/argvus` e il pacchetto che fornisce `/usr/bin/argvus`. La crate `argvus-cli` è il dispatcher: legge il primo argomento ed esegue un comando di libreria in-process, esegue un helper GTK 4 o un eseguibile privato, oppure esegue uno script impacchettato. L'elenco completo dei sottocomandi è nella [riferimento alla linea di comando](/docs/reference/command-line/)].

L'ambiente contiene anche le crate che i comandi condividono:

- `argvus-config`: lo store di configurazione canonico e l'unico scrittore dell'albero generato (`data/generated/`). È anche una libreria utilizzata in-process da `argvus config`, `theme-switch` e `effects-toggle`.
- `argvus-i18n` e `argvus-i18n-core`: i cataloghi in `resources/locales/`, la loro validazione e il modulo QML `org.argvus.i18n`.
- `argvus-tui` e `argvus-theme`: interfaccia terminale condivisa e caricamento dei temi.
- `argvus-runtime`: i messaggi tradotti e gli helper di processo condivisi dai comandi.

`argvus-session` rimane un repository separato. Fornisce i punti di ingresso della sessione (`session`, `start`, `tty`) e le unità del ciclo di vita, e `argvus sessionctl` raggiunge il suo controller.
