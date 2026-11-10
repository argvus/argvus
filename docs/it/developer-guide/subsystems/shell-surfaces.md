---
title: Superfici della conchiglia
description: Barra delle applicazioni, pannello di controllo e proprietà della telemetria.
---
`argvus-waybar` fornisce il `/usr/bin/waybar` corretto. `argvus-taskbar` possiede le azioni della barra delle applicazioni e l'UX; il JSONC e il CSS della barra delle applicazioni che fornisce sono proiezioni di `config.json`, e `argvus-config` scrive il livello attivo sotto `data/generated/waybar/` più una copia speculare sotto `data/waybar/`. `argvus-control-panel` possiede le schede Quickshell. `argvus-widget-telemetry` possiede la barra telemetrica opzionale e la sua semantica di blocco; il suo JSONC e CSS vengono scritti in `data/waybar/` dai valori predefiniti pacchettizzati in caso di modifica dell'aspetto, dopo di che `argvus-config` riscrive solo i blocchi delimitati `ARGVUS_TELEMETRY_*` e il font. `argvus-session` possiede i loro servizi utente.

Non esiste attualmente alcun pacchetto `argvus-shell` o `argvus-shell.service` in questo modello di runtime.
