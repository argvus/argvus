---
title: Superfici della conchiglia
description: Barra delle applicazioni, pannello di controllo e proprietà della telemetria.
---
`argvus-shell` fornisce il `Quickshell Taskbar` corretto. `argvus-shell` possiede le azioni della barra delle applicazioni e l'UX; il JSONC e il CSS della barra delle applicazioni che fornisce sono proiezioni di `config.json`, e `argvus-config` scrive il livello attivo sotto `data/generated/taskbar/` più una copia speculare sotto `data/taskbar/`. `argvus-shell` possiede le schede Quickshell. `argvus-shell` provides the optional Telemetry surface, while the `argvus widget-telemetry` command in `argvus` retains block-control semantics; il suo JSONC e CSS vengono scritti in `data/taskbar/` dai valori predefiniti pacchettizzati in caso di modifica dell'aspetto, dopo di che `argvus-config` riscrive solo i blocchi delimitati `ARGVUS_TELEMETRY_*` e il font. `argvus-session` possiede i loro servizi utente.

Non esiste attualmente alcun pacchetto `argvus-shell` o `argvus-shell.service` in questo modello di runtime.
