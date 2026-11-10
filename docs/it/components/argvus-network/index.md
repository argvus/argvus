---
title: Reti e Bluetooth
description: Gestisci NetworkManager e Bluetooth tramite ARGVUS.
---
Il sottocomando `argvus network` segnala e controlla NetworkManager, e `argvus bluetooth` segnala e controlla lo stato e la disponibilità dell'adattatore Bluetooth. Entrambi vengono eseguiti in-process all'interno di `argvus`; `networkctl` e `bluetoothctl` rimangono come alias. Il Pannello di Controllo e Waybar richiamano questi comandi.

Usare `--help` su uno qualsiasi dei comandi per i sottocomandi installati. L'integrazione dell'interfaccia utente di rete e Bluetooth è separata dai gestori avanzati opzionali come Blueman.

Il Centro di Controllo separa lo stato della rete dalle pagine di configurazione: **Rete → Stato** riassume lo stato attuale, mentre interfacce, Wi-Fi, Ethernet, VPN, DNS, proxy e firewall sono percorsi separati quando i loro fornitori sono disponibili. **Bluetooth → Stato**, **Dispositivi** e **Accoppia** coprono lo stato dell'adattatore e le azioni sui dispositivi. Queste pagine eseguono azioni di sistema supportate dai fornitori; la barra delle applicazioni e il Pannello di Controllo espongono la superficie rapida di stato/azione.

Le modifiche alla rete e al Bluetooth possono richiedere l'autorizzazione e possono essere applicate immediatamente tramite NetworkManager o il servizio Bluetooth. Non sono la stessa cosa che modificare l'indicatore della barra delle applicazioni ARGVUS.

Il Centro di Controllo invia le password Wi-Fi a nmcli tramite input standard con `--ask`; le password sono escluse dagli argomenti del processo e oscurate dagli errori dei comandi restituiti.

## Gestione della rete da linea di comando

Usa `argvus networkctl` per interrogare e controllare la rete senza l'interfaccia grafica:

```sh
# Show network status
argvus networkctl status

# Toggle networking on/off
argvus networkctl enable
argvus networkctl disable

# Wi-Fi control
argvus networkctl wifi on
argvus networkctl wifi off

# Open the network manager
argvus networkctl manager
```
## Gestione del Bluetooth da riga di comando

Usa `argvus bluetoothctl` per l'adattatore Bluetooth:

```sh
# Show the adapter state (key=value lines)
argvus bluetoothctl status

# Exit 0 when an adapter is available, 1 otherwise
argvus bluetoothctl available

# Turn the adapter on or off, or toggle it
argvus bluetoothctl enable
argvus bluetoothctl disable
argvus bluetoothctl toggle

# Open the Bluetooth manager (Blueman)
argvus bluetoothctl manager
```
Quando il kernel non elenca alcun controller in `/sys/class/bluetooth`, `status` risponde a `status=no-controller` immediatamente, senza chiedere a `bluetoothctl` (che altrimenti aspetterebbe BlueZ quando il servizio è fermo). Il modulo della barra delle applicazioni e le schede del Pannello di Controllo leggono lo stesso stato.

Usa `--help` su uno qualsiasi dei comandi per vedere opzioni aggiuntive e sottocomandi specifici per la versione installata.

Quando la finestra di dialogo della password Wi-Fi è aperta, gli aggiornamenti della scansione preservano l'SSID selezionato e il tipo di sicurezza. Se quella rete scompare o la sua sicurezza cambia, la conferma si interrompe con un errore e richiede una nuova selezione.

Il DNS manuale invia i server IPv4 e IPv6 alle rispettive impostazioni di NetworkManager prima di aggiornare il file del resolver. I collegamenti simbolici del resolver gestiti dal servizio vengono preservati rifiutando la sostituzione del file. Un ritorno riuscito al DNS automatico ripristina e rimuove il backup, quindi un ciclo manuale successivo salva lo stato corrente del resolver.

La chiusura dell'editor del profilo di rete attende il completamento del lavoro corrente, quindi salvare o eliminare un profilo non può essere interrotto scartando l'editor. Gli aggiornamenti in background rimangono annullabili.

Le richieste di connessione Wi-Fi condividono il controllo delle azioni di rete: un'azione in esecuzione o un lavoro dell'editor del profilo viene completato prima che possa iniziare un'altra modifica. I messaggi di avanzamento e completamento seguono la lingua selezionata.

La conferma della rimozione del Bluetooth mantiene l'indirizzo originale del dispositivo anche quando la lista dei dispositivi viene aggiornata. Le percentuali di batteria da bluetoothctl vengono riconosciute e le richieste di dettaglio dei dispositivi vengono eseguite con concorrenza limitata e timeout individuali.

La password Wi-Fi, i campi di ricerca DNS manuale e di rete accettano l'incolla tra parentesi. I controlli incollati e le interruzioni di riga vengono omessi. Le scorciatoie da tastiera non supportate solo con Ctrl o solo con Alt non inseriscono lettere. Gli editor accettano eventi AltGr stampabili codificati come Ctrl+Alt; Ctrl+S salva solo senza Alt. L'editor di testo condiviso mantiene la navigazione verticale nelle colonne visive e allinea lo scorrimento orizzontale per i caratteri larghi.
