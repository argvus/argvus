---
title: Dispositivi rimovibili
description: Montare e gestire lo storage rimovibile da ARGVUS.
---
`argvus-removable-devices` osserva UDisks2 e fornisce il modulo di archiviazione della taskbar ARGVUS e il suo menu dei dispositivi. Il menu è disponibile anche come interfaccia separata in stile Rofi, quindi le stesse azioni sui dispositivi possono essere utilizzate sia dalla taskbar che da un flusso di lavoro del launcher.

Le azioni disponibili sono `open`, `mount`, `unmount`, `eject`, `poweroff`, `lock`, `unlock` e `copy`. I comandi `list`, `watch` e `menu` espongono le corrispondenti interfacce di stato, stream della taskbar e menu per le integrazioni.

## Menu della taskbar

Con `"mode": "gui"` (il sistema predefinito), un clic sull'icona di archiviazione della taskbar apre un menu GTK collegato alla Taskbar ARGVUS, come il popup del calendario: sotto la barra quando la taskbar è in alto sullo schermo e sopra di essa quando la taskbar è in basso. Ogni riga del dispositivo ha un sottomenu delle azioni con solo le azioni consentite dal suo stato.

- **Puntatore:** passando sopra o cliccando una riga del dispositivo si apre il suo sottomenu. Il sottomenu rimane aperto mentre il puntatore attraversa i bordi del menu o lascia i menu, e segue un'altra riga solo dopo che il puntatore vi resta brevemente sopra, quindi un percorso diagonale verso il sottomenu non lo chiude. Un clic al di fuori di entrambi i menu chiude il popup.
- **Tastiera:** `Up`/`Down` selezionano una riga, `Right`, `Enter` o `Space` aprono il sottomenu (o eseguono **Aggiorna**/**Esci**), `Enter` o `Space` eseguono l’azione selezionata, `Left` o `Escape` tornano alla lista dei dispositivi e `Escape` chiude lì il popup.

**Copia percorso** mette il punto di montaggio negli appunti tramite `copy_command` (predefinito `wl-copy`, da `wl-clipboard`), quindi il percorso rimane disponibile anche dopo la chiusura del menu; una notifica lo conferma quando `show_notification` è attivo. **Blocca** è offerto per ogni volume crittografato sbloccato e lo smonta prima, se necessario.

L'azione predefinita del file manager segue il file manager selezionato nel Centro di Controllo ARGVUS. Questo mantiene le azioni dei dispositivi rimovibili coerenti con le impostazioni predefinite delle applicazioni dell'utente.

La configurazione di sistema è `/etc/argvus/removable-devices/config.json`. La proiezione per utente gestita da ARGVUS è `~/.config/argvus/data/removable-devices/config.json`, con una `theme.css` corrispondente accanto. Quel foglio di stile è anche proiettato su `~/.config/argvus/data/generated/removable-devices/theme.css` dal tema canonico, accent e modalità, con un fallback famiglia/variante per i temi che non includono un asset dedicato ai dispositivi rimovibili, così un cambio di tema mantiene il modulo coerente senza alcun passaggio manuale. I percorsi dei componenti legacy sono input per la migrazione; la configurazione successiva dell'utente sovrascrive i valori predefiniti di sistema.

Le impostazioni utili includono se il modulo è nascosto quando non è presente alcun dispositivo (`hide_when_empty`), se vengono mostrati nomi e capacità, l'ordine di ordinamento dei dispositivi, il comportamento delle notifiche, il backend del menu e le icone di stato. Il suo foglio di stile viene generato dal tema ARGVUS attivo (`removable-devices/theme.css` della build del tema) e copiato nel `theme.css` del pannello ad ogni cambio di tema o accento.

L'interfaccia utilizza la lingua dell'utente da `LANG` in `$XDG_CONFIG_HOME/locale.conf`, con l'inglese come impostazione predefinita per un utente non configurato. Le variabili di localizzazione ereditate non selezionano la lingua dell'interfaccia.

## Cataloghi linguistici condivisi

L'interfaccia utilizza i cataloghi condivisi `argvus-i18n` per tutte e dieci le lingue. I testi sono memorizzati in `resources/locales/<locale>/removable-devices.json` nello spazio di lavoro principale e installati da `argvus`. L'inglese è il fallback; le nuove lingue non richiedono più un dizionario bilingue locale.
