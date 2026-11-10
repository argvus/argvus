---
title: ARGVUS Analizzatore dell'Uso del Disco
description: Sfoglia l'utilizzo del disco in modo interattivo con ncdu in una finestra Kitty a tema.
---
Apri **ARGVUS Disk Usage Analyzer** dal launcher delle applicazioni o esegui:

```sh
argvus du
```
L'analizzatore apre una finestra Kitty dedicata e centrata e incorpora l'interfaccia reale di ncdu all'interno del suo frame ARGVUS. Avvia in `$HOME`. Modifica il percorso della directory o scegli **Sfoglia** per aprire il file manager configurato. Con Superfile, naviga in una directory e premi **Spazio** per selezionarla; `q` o `Esc` annulla. Altri file manager configurati si aprono normalmente, ma potrebbero non restituire una directory selezionata, quindi inserisci o incolla il percorso nell'analizzatore.

Usa il tasto Tab o i tasti freccia per spostarti tra la directory, le opzioni e le azioni. Il grafico, la percentuale, lo schema di colori scuro e l'ordinamento con le directory prima sono attivi di default. Le restanti opzioni di scansione e visualizzazione possono essere attivate o disattivate prima di selezionare **Analizza**. Nei Risultati, usa i controlli da tastiera di ncdu; `q` torna al modulo e `Esc` interrompe la scansione. Premi `d` per eliminare il file o la directory selezionata. ncdu chiede conferma; l'eliminazione è permanente e utilizza i permessi dell'utente corrente. Non sono richiesti privilegi elevati.

L'interfaccia esterna segue il tema ARGVUS attivo e la localizzazione. Sono richiesti Kitty e ncdu; il pacchetto `argvus-devtools` fornisce ncdu. Se Kitty o ncdu mancano, l'applicazione segnala la dipendenza mancante.
