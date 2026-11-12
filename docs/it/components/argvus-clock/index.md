---
title: Orologio ARGVUS
description: Orologio, orologi mondiali, cronometro, timer a conto alla rovescia e sveglie con suono.
---
ARGVUS Clock è un'applicazione GTK 4 con l'ora corrente e gli strumenti temporali: orologi mondiali, un cronometro, un timer di conto alla rovescia e sveglie. Aprila dal launcher, cliccando sull'ora nella barra delle applicazioni, con **SUPER + Shift + C**, o con:

```sh
argvus clock                     # open the popup near the pointer
argvus clock --toggle            # hide the popup if it is visible, show it otherwise
argvus clock --toggle --x 1600 --y 30  # open next to a taskbar click position
argvus clock --page alarms       # open on a page: clock, world, stopwatch, timer, alarms
argvus clock --background        # run without a window, to keep alarms active
```
L'orologio ARGVUS si apre come popup allineato con la barra delle applicazioni, con una distanza di 4 pixel. La barra delle applicazioni passa le coordinate del clic principale, quindi il popup utilizza il monitor cliccato; avviarlo senza coordinate utilizza la posizione del puntatore. Se il puntatore resta fuori dal popup per 5 secondi, questo si chiude automaticamente; spostare il puntatore nel popup annulla il conto alla rovescia. **Esc** su una scheda principale lo nasconde immediatamente, e **Esc** su una pagina intera torna alla scheda precedente. `argvus --clock` è un alias. Cliccando sulla data nella barra delle applicazioni si apre ancora il calendario; l'orologio ARGVUS non mostra eventi.

ARGVUS Focus, l'orologio ARGVUS e il Pannello di Controllo condividono lo stesso bordo dello schermo. Aprire l'orologio chiude prima Focus e il Pannello di Controllo; aprire una di queste superfici chiude l'orologio.

## Pagine

- **Orologio**: l'ora in cifre grandi (24 ore di default, 12 ore nelle impostazioni, secondi opzionali), la data localizzata, il fuso orario locale e il suo offset UTC, e la prossima sveglia.
- **Mondo**: città aggiunte cercando una città o un fuso orario IANA (accents e maiuscole/minuscole ignorati). Ognuna mostra l'ora, la differenza rispetto all'ora locale e se lì è ieri, oggi o domani. L'ora legale deriva dal database dei fusi orari IANA; nulla è calcolato con offset fissi. Le città possono essere riordinate o rimosse.
- **Cronometro**: avvia, pausa, riprendi, resetta e giri (tempo del giro e totale).
- **Timer**: ore, minuti e secondi, o i preset 5, 10, 15, 25, 30 e 60 minuti. Pausa, riprendi, riavvia e annulla. Quando finisce suona come una sveglia. Questo timer è indipendente dalle sessioni ARGVUS Focus.
- **Sveglie**: creare, modificare, accendere e spegnere ed eliminare le sveglie. Ognuna ha un orario, un nome opzionale, i giorni della settimana in cui si ripete (nessuno significa che suona una sola volta), un suono e una durata del snooze.

Tastiera: **Ctrl+1** a **Ctrl+5** cambia pagina, **Ctrl+N** crea una sveglia, **Ctrl+,** apre le impostazioni, **Esc** torna indietro, **Ctrl+W** chiude la finestra e **Ctrl+Q** esce.

## Suono della sveglia

Le sveglie riproducono un suono reale finché non le si risponde:

- Suoni: **Chime** (predefinito) e **Beacon**, inclusi in `/usr/share/argvus/sounds/`.
- **Volume della sveglia** (da 10 a 100, predefinito 80) e **Ascolta il suono della sveglia** si trovano nelle impostazioni; l'editor ha un pulsante di prova per il suono scelto.
- Una sveglia che suona mostra una scheda in cima allo schermo, sopra le altre finestre, e una notifica, entrambe con **Posticipa** e **Chiudi**. Con la scheda selezionata, **Invio** chiude e **S** posticipa.
- Il posticipo suona di nuovo dopo la durata del posticipo della sveglia (da 1 a 60 minuti, predefinito 10).
- Una sveglia non risposta si interrompe dopo **Interrompi suoneria dopo** (5, 10, 15 o 30 minuti, predefinito 10) e lascia una notifica.
- Due sveglie che suonano insieme sono entrambe elencate; la seconda non silenzia mai la prima.
- Se il suono non può essere riprodotto, la scheda e la finestra lo indicano con l'errore, e l'errore viene registrato. ARGVUS Clock non segnala mai un suono che non è stato riprodotto.

La riproduzione utilizza il supporto multimediale di GTK (GStreamer), quindi funziona con PipeWire, PulseAudio o ALSA. Il pacchetto dipende da `gst-plugins-base` (Ogg Vorbis) e `gst-plugins-good` (output audio).

## Quando suonano le sveglie

Le sveglie suonano solo mentre ARGVUS Clock è in esecuzione nella tua sessione:

- Chiudere la finestra mantiene ARGVUS Clock in esecuzione mentre sveglie, un timer in esecuzione o un cronometro in esecuzione ne hanno bisogno (**Mantieni in esecuzione quando la finestra è chiusa**, attivo di default). Se non ci sono operazioni in sospeso, si chiude.
- **Avvia con la sessione** abilita l'unità utente `argvus-clock.service`, che esegue `argvus clock --background` dopo il login. È disattivato di default.
- Le sveglie non possono suonare mentre sei disconnesso, il computer è spento o è in sospensione.
- Un allarme rilevato fino a 5 minuti in ritardo suona comunque. Più tardi di così (l'orologio non era in funzione, o il sistema era in sospensione o spento) non suona: viene segnalato come **mancato** in una notifica e nella pagina Allarmi. Un allarme ripetuto passa al giorno successivo; un allarme una tantum si spegne.
- Ogni occorrenza viene salvata come gestita prima che suoni, quindi riaprendo ARGVUS Clock lo stesso allarme non suona due volte e solo un'istanza è attiva per sessione.
- Gli allarmi ripetuti vengono calcolati sull'ora locale nei giorni della settimana scelti, quindi mantengono il loro orario anche durante i cambi di ora legale. Un orario saltato con il cambio di primavera suona al primo minuto dopo il vuoto; un orario che si presenta due volte suona la prima volta.
- Un timer continua a contare durante una sospensione. Se è terminato mentre il sistema era in sospensione, suona al risveglio e indica quanto è in ritardo. Dopo che ARGVUS Clock è stato chiuso, un timer o un cronometro in esecuzione torna indietro in pausa.

## Percorsi

| Percorso | Contenuto |
| --- | --- |
| `$XDG_CONFIG_HOME/argvus/data/clock/settings.json` | Impostazioni e orologi mondiali |
| `$XDG_DATA_HOME/argvus/clock/alarms.json` | Sveglie |
| `$XDG_STATE_HOME/argvus/clock/timers.json` | Timer e cronometro checkpoint |
| `/usr/lib/systemd/user/argvus-clock.service` | Avvia con la sessione |
| `/usr/share/argvus/sounds/` | Suoni inclusi |

I file vengono scritti in modo atomico; un file che non può essere letto viene rinominato in `<name>.corrupt-<time>` e vengono utilizzati valori predefiniti. I colori seguono il tema ARGVUS attivo e il colore di evidenziazione (consumer del tema `time`). Le stringhe provengono dai domini di traduzione `clock` e `time`.

Il calendario della barra delle applicazioni, ARGVUS Focus, ARGVUS Clock e il Pannello di Controllo si escludono a vicenda: aprirne uno chiude gli altri.
