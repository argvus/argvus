---
title: Sovrapposizione di caricamento
description: La sovrapposizione mostrata mentre la sessione si ricarica direttamente.
---
Il sovrapposto di caricamento è una superficie layer-shell GTK 4 mostrata mentre ARGVUS ricarica direttamente la sessione (`argvus sessionctl reload`). Un cambio di tema non lo utilizza: il desktop cambia sul posto. È un componente di supporto fornito nel pacchetto `argvus` in `/usr/lib/argvus/loading-theme/splash`, e non viene posizionato in `/usr/bin`.

Il login non lo utilizza: il greeter splash e lo spinner di avvio della sessione sono la modalità Wayland del helper di handoff, che disegna lo stesso spinner senza GTK e mostra il suo primo frame in millisecondi (vedi [Greeter, lock and splash](/docs/developer-guide/subsystems/greeter-lock-and-splash/)). Con `--spinner-only` l'overlay mostra solo quello spinner e termina su `SIGTERM`. Con `--detached` non termina quando il processo che lo ha avviato termina; un reload della sessione usa entrambi i flag e lo interrompe tramite il suo file pid. Legge il tema selezionato dalla directory dei dati ARGVUS e ricade sulla palette Dark ARGVUS fornita nel pacchetto. Riferisce la disponibilità solo dopo il primo frame di ogni superficie che mappa, non quando il processo parte.
