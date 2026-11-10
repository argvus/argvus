---
title: Sensori di temperatura
description: Panoramica della temperatura in tempo reale supportata da JSON di lm_sensors.
---
`argvus sensors` apre il pannello Ratatui in elaborazione utilizzato dal popup della temperatura della CPU sulla barra delle applicazioni. Esegue `sensors -j`, raggruppa le letture per i nomi dei chip e le etichette segnalate dall'hardware e si aggiorna ogni due secondi. Il monitoraggio della GPU è gestito da `argvus system-monitor --gpu`.

Le letture utilizzano i Celsius. Il pannello mostra `max`, `crit` e lo stato di allarme quando lm_sensors li fornisce. Un indicatore è mostrato solo quando è disponibile un limite positivo. `lm_sensors` mancanti, errori di comando, JSON non valido e letture vuote rimangono visibili come stati recuperabili; il polling riprova al ciclo successivo.

Premere `q` o `Esc` per chiudere. La barra delle applicazioni apre il pannello in Kitty tramite `argvus tui-terminal`; il titolo della scheda inferiore localizzato mostra il suggerimento per chiudere. Questo comando possiede i dettagli della temperatura della CPU; l'icona della temperatura GPU apre il monitor btop solo per GPU. Il pacchetto della barra delle applicazioni dipende già da `lm_sensors`, quindi non è necessario alcun pacchetto o dipendenza separata.
