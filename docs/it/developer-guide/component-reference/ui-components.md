---
title: Componenti dell'interfaccia utente
description: Pacchetti dell'interfaccia utente ARGVUS.
---
I pacchetti UI includono `argvus-control-panel`, `argvus-taskbar` e `argvus-launcher`, e i sottocomandi `argvus` `argvus control-center` (helper in `argvus`), `argvus taskbar-calendar`, `argvus widget-telemetry`, `argvus terminal`, `argvus system-monitor`, `argvus removable-devices`, `argvus focus` e `argvus clock`. Il calendario, Focus e Clock utilizzano la crate condivisa `argvus-taskbar-popup` per il posizionamento e la chiusura layer-shell; Focus inoltre espone il suo lettore di stato Waybar attraverso la route in-process `argvus focus status`.

Il loro comportamento visibile all'utente è documentato per funzione; questa pagina registra la proprietà dei pacchetti.
