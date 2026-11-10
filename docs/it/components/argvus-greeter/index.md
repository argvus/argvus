---
title: Accogliente
description: Il saluto di accesso che accoglie gli utenti all'avvio è fornito nel pacchetto argvus.
---
`argvus-greeter` è il greeter di login per greetd. È un frontend terminale orientato alla tastiera che scopre le sessioni Wayland installate, mostra gli account con i loro avatar e applica il tema ARGVUS attivo per l'account selezionato.

Fa parte del pacchetto `argvus`, l'unico pacchetto del desktop. Il suo codice sorgente si trova in `crates/argvus-greeter` del repository `argvus`, come membro del workspace root di Cargo, ed è compilato insieme al resto di `argvus`.

Il greeter non autentica gli utenti da solo. greetd gestisce l'autenticazione tramite PAM. Il launcher apre il greeter con `argvus tui-terminal`, quindi il greeter non ha bisogno di un programma terminale separato.

I file di configurazione del greeter sono installati sotto `/etc/argvus`. Usa `argvus greeter-setup --help` per vedere le opzioni di configurazione che il pacchetto fornisce. Per il flusso di login, vedere [Sessions](/docs/user-guide/sessions/)].

Il socket di login handoff autorizza ogni client con le credenziali kernel peer. Solo root, il proprietario dell’helper e l’account autenticato da greetd possono inviare comandi di controllo. Altri account locali vengono rifiutati anche se il socket consente connessioni tra utenti. Un frame DRM preparato rimane legato al suo descrittore originale; l’helper attende quel descrittore prima di prendere il controllo del display.

Quando il campo di autenticazione è attivo, i caratteri stampabili (inclusi `j` e `k`) vengono inseriti nella risposta. Tab e Shift-Tab spostano il focus; Backspace rimuove l’ultimo carattere. Ciò si applica sia alle richieste segrete che non segrete.

La lista degli account si scorre per mantenere visibile l’account selezionato mentre si naviga o si ridimensiona il terminale.
