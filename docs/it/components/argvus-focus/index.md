---
title: Focus ARGVUS
description: Pomodoro e sessioni di concentrazione con pause, notifiche e una cronologia locale delle sessioni.
---
ARGVUS Focus è un'applicazione GTK 4 per lavorare in sessioni di concentrazione separate da pause. Si apre come un popup accanto alla barra delle applicazioni. Aprilo dal launcher, cliccando sull'indicatore della barra delle applicazioni, con **SUPER + Shift + F**, o con:

```sh
argvus focus              # open the popup near the pointer
argvus focus --toggle     # hide the popup if it is visible, show it otherwise
argvus focus --toggle --x 1600 --y 30  # open next to a taskbar click position
argvus focus --start      # start or resume the current phase without opening the window
argvus focus --pause      # pause the running phase
argvus focus status --waybar         # print one Waybar JSON status and exit
argvus focus status --waybar --follow  # stream status changes for Waybar
```
La barra delle applicazioni passa le coordinate del clic radice in modo che il popup utilizzi il monitor cliccato e si allinei con la barra; un avvio senza coordinate utilizza la posizione del puntatore. Se il puntatore rimane al di fuori del popup per 5 secondi, questo si chiude automaticamente; spostare il puntatore all'interno del popup annulla quel conto alla rovescia. **Esc** nella pagina principale lo nasconde ancora immediatamente, e **Esc** in una sottopagina ritorna alla pagina precedente. L'indicatore della barra delle applicazioni mostra solo la sua icona mentre è inattivo e il rimanente `MM:SS` durante una fase, con il colore della fase; una fase in pausa mantiene il tempo rimanente e aggiunge uno stato di pausa. Il comando `status` viene eseguito all'interno di `argvus` senza avviare GTK. `argvus --focus` è un alias. Viene eseguita un'unica istanza di Focus per sessione: un secondo avvio passa le sue opzioni a quella in esecuzione.

ARGVUS Focus, ARGVUS Clock e il Pannello di Controllo condividono lo stesso bordo dello schermo. Aprire Focus chiude prima Clock e il Pannello di Controllo; aprire una qualsiasi di queste superfici chiude Focus.

## Sessioni

- La modalità **Pomodoro** alterna sessioni di concentrazione e pause brevi, e termina ogni ciclo con una pausa lunga. Le durate predefinite sono di 25, 5 e 15 minuti, con una pausa lunga dopo 4 sessioni di concentrazione.
- La modalità **Singola sessione** esegue una sola sessione di concentrazione della sua durata (45 minuti per impostazione predefinita) senza pause.
- La finestra mostra la fase (concentrazione, pausa breve, pausa lunga, ciascuna con un colore proprio), il tempo rimanente all'interno di un indicatore a cerchio, la posizione nel ciclo e il totale di oggi.
- **Inizia concentrazione / Inizia pausa**, **Pausa**, **Riprendi**, **Annulla sessione** (interrompe la fase e non registra nulla) e **Completa ora** (termina anticipatamente la sessione di concentrazione e registra il tempo effettivamente trascorso). Durante una pausa, **Salta pausa** torna alla concentrazione.
- Quando una fase termina, la fase successiva aspetta te, a meno che non siano attivi **Avvia automaticamente le pause** o **Avvia automaticamente la concentrazione dopo una pausa**.

Tastiera: **Spazio** avvia o mette in pausa, **Ctrl+,** apre le impostazioni, **Ctrl+H** la cronologia, **Esc** torna indietro, **Ctrl+W** nasconde la finestra e **Ctrl+Q** esce.

## Impostazioni

I preset (Classico 25/5/15, Esteso 50/10/30, Lavoro profondo 90/20/30) completano le durate, che rimangono modificabili. Le impostazioni includono anche le sessioni prima di una pausa lunga, gli avvii automatici, la notifica alla fine di una fase e i **suoni della sessione**: un breve campanello con il proprio volume e un pulsante **Test suono**. Questi suoni sono separati dai suoni di allarme di [ARGVUS Clock](/docs/components/argvus-clock/).

## Cronologia

Ogni sessione di concentrazione completata viene registrata con inizio, fine e tempo trascorso; le pause e le sessioni annullate non vengono registrate. La pagina della cronologia mostra le sessioni e i minuti di oggi, i totali e le sessioni recenti. I record più vecchi di 365 giorni vengono eliminati. **Cancella cronologia** richiede conferma.

## Finestra, sfondo e sospensione

- Chiudere la finestra durante un ciclo la nasconde solo: la sessione continua e ti avvisa quando la fase termina. Senza nulla in corso, chiudere esce.
- Se il sistema viene sospeso mentre una fase è in esecuzione, la fase viene messa in pausa al momento della sospensione e un messaggio indica quanto tempo il sistema è rimasto in sospensione. Il tempo di sospensione non viene contato come tempo di concentrazione.
- La sessione viene salvata ad ogni cambiamento e ogni 30 secondi. Se ARGVUS Focus viene chiuso durante una fase (logout, crash, **Ctrl+Q**), alla successiva apertura ricompare in pausa al punto in cui era rimasto; non fa mai finta che il timer abbia continuato a funzionare mentre non poteva notificare.
- I tempi sono misurati sull'orologio di avvio del sistema, quindi cambiare l'orologio o il fuso orario non accorcia né allunga una fase.

## Percorsi

| Percorso | Contenuto |
| --- | --- |
| `$XDG_CONFIG_HOME/argvus/data/focus/settings.json` | Impostazioni |
| `$XDG_DATA_HOME/argvus/focus/history.json` | Sessioni completate |
| `$XDG_STATE_HOME/argvus/focus/session.json` | Checkpoint della sessione corrente |
| `/usr/share/argvus/sounds/argvus-bell.oga` | Campanello della sessione |

I file vengono scritti in modo atomico. Un file che non può essere letto viene rinominato in `<name>.corrupt-<time>` e vengono utilizzati i valori predefiniti; un messaggio comunica dove è stato spostato.

I colori seguono il tema ARGVUS attivo e il colore di evidenziazione (consumatore del tema `time`). Le stringhe provengono dai domini di traduzione `focus` e `time`.

Il calendario della barra delle applicazioni, ARGVUS Focus, ARGVUS Clock e il Pannello di controllo sono mutuamente esclusivi: aprirne uno chiude gli altri.
