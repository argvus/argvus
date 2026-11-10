---
title: Requisiti
description: Requisiti di sistema e prerequisiti per ARGVUS.
---
ARGVUS è destinato ad Arch Linux e richiede uno stack grafico compatibile con Wayland, Hyprland e le dipendenze runtime incluse nei pacchetti. Un gestore di display è opzionale: ARGVUS supporta un flusso di login grafico tramite greetd e un flusso manuale su TTY.

Prima dell'installazione, assicurati che:

- il sistema sia completamente aggiornato con `pacman`;
- sia installato un kernel funzionante e un driver grafico;
- l'utente possa avviare una sessione Wayland;
- la rete sia disponibile quando il repository dei pacchetti è configurato.

Le macchine virtuali potrebbero richiedere le impostazioni di compatibilità descritte in [Virtual machines](/docs/user-guide/virtual-machines/)].
