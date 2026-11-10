---
title: Terminale
description: Apri il terminale ARGVUS e i profili dell'applicazione del terminale.
---
Apri il terminale configurato con:

```sh
argvus --terminal
```
`argvus terminal` (un sottocomando di `argvus`) fornisce l'integrazione del terminale e la configurazione di Kitty. `argvus-app-profiles` fornisce profili per applicazioni terminali come Superfile e Yazi; non fornisce il dispatcher `argvus`.

`argvus-config` proietta il tema in `data/generated/terminal/`, e `argvus --yazi` e `argvus --spf` leggono la loro configurazione dagli alberi proiettati `data/generated/yazi/` e `data/generated/superfile/` piuttosto che dalle copie materializzate nella tua directory home. Una completa sovrascrittura nativa nella directory di configurazione dell'applicazione stessa prevale ancora, quindi puoi sostituire l'albero proiettato con la tua configurazione.

Le modifiche al tema rigenerano i file derivati di Kitty sotto `$XDG_CACHE_HOME/argvus/argvus-terminal`. Il generatore serializza gli aggiornamenti e sostituisce ciascun file in modo atomico. Se il terminale ancora non si apre, controlla il tema attivo e rigenera la cache con:

```sh
argvus config get /appearance/theme --raw
argvus terminal --apply
argvus terminal --print-config
```
Il terminal legge prima il tema gestito e l'accento da `$XDG_CONFIG_HOME/argvus/config.json`. I file legacy `.active-theme` e `.accent-color` rimangono backup per installazioni più vecchie.

## Trasparenza e sfocatura

Su **Centro di Controllo → Aspetto → Terminale**, configura **Trasparenza** e **Sfocatura** indipendentemente da `0%` a `100%`. Le modifiche vengono applicate con il pulsante **Applica** e rigenerano il profilo Kitty sotto `$XDG_CACHE_HOME/argvus/argvus-terminal`; le finestre Kitty esistenti ricevono la configurazione aggiornata tramite un ricaricamento.

La trasparenza `0%` mantiene lo sfondo opaco, mentre `100%` lo rende completamente trasparente. La sfocatura utilizza il raggio di sfocatura dello sfondo supportato da Kitty e ha effetto visibile solo quando la finestra è trasparente.

## Notifica per comandi in esecuzione prolungata

I comandi che vengono eseguiti per 30 secondi o più inviano una notifica al termine, purché la finestra del terminale non sia attiva. La notifica utilizza `notify_on_cmd_finish` di Kitty con l'integrazione della shell già abilitata, quindi `~/.zshrc` e `~/.bashrc` non necessitano di modifiche. La soglia di 30 secondi è fissa nella configurazione di Kitty.

## Modalità App TUI

ARGVUS Control Center e i popup TUI della barra delle applicazioni utilizzano `argvus tui-terminal` con un profilo app dedicato di Kitty. Eredita il tema e il font ARGVUS attivi, assegna una classe di finestra stabile e esce con il suo processo figlio invece di aprire una shell. Il Control Center nasconde le schede e ha valori indipendenti di Trasparenza e Sfocatura; il monitor della barra delle applicazioni e i popup di temperatura utilizzano una barra delle schede inferiore per il loro titolo localizzato `Press q to close`. Il launcher accetta `--title TITLE` per impostare un titolo fisso della finestra e della scheda di Kitty. Il terminale normale mantiene le sue schede e impostazioni.

Quando LANG è assente dall'ambiente del launcher, `argvus terminal` fornisce la locale personale da `$XDG_CONFIG_HOME/locale.conf` (inglese per impostazione predefinita) prima che Kitty inizializzi la composizione della tastiera. Le sovrascritture dell'ambiente locale esistenti vengono preservate. Riavviare Kitty dopo l'aggiornamento: esportare LANG all'interno di una shell esistente non reinizializza la tabella di composizione di Kitty.

LC_ALL e LC_CTYPE vuoti ereditati dall'inizializzazione della sessione vengono rimossi dall'ambiente figlio di Kitty. Altrimenti, Kitty tratta una locale vuota come esplicita e non può caricare la sua tabella di composizione, anche con un LANG valido. Le sovrascritture non vuote vengono preservate. Questo vale anche per i nuovi profili e richiede la riapertura di Kitty dopo l'aggiornamento.

La finestra dedicata del Centro di Controllo (`argvus-control-center`) riceve l'override Kitty per ogni avvio `confirm_os_window_close=0`, quindi l'azione di chiusura del compositore non richiede conferma. L'override non viene scritto nel profilo terminale condiviso; altri TUI e terminali normali mantengono la loro politica esistente.
