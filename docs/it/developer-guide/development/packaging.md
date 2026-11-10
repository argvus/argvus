---
title: Confezionamento
description: Costruisci e ispeziona pacchetti ARGVUS Arch.
---
Ogni componente possiede il suo pacchetto Arch sotto il proprio repository. Ispeziona `packaging/archlinux/`, le funzioni del pacchetto e l'elenco dei file installati prima di documentare un payload. I valori predefiniti del pacchetto sotto `/usr/share/argvus` e la configurazione dell'amministratore sotto `/etc/argvus` non sono intercambiabili.

Per la verifica in fase di esecuzione, confronta i contenuti del pacchetto con `pacman -Ql <package>` e ispeziona separatamente le modifiche attive dell'utente.

Le build locali conservano gli oggetti Cargo in `build/cargo-target`, fuori dall'albero delle sorgenti del pacchetto, quindi una successiva `make build` ricompila solo ciò che è cambiato. `make clean` rimuove quella directory e la build successiva parte da zero.

Non dedurre il payload in fase di esecuzione solo dal nome del repository. Un pacchetto può possedere binari, servizi utente, valori predefiniti `/usr/share/argvus`, configurazione dell'amministratore `/etc/argvus` o risorse. Conferma il payload reale con `PKGBUILD` e `pacman -Ql` del repository dopo l'installazione.
