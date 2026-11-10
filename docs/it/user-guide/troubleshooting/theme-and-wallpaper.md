---
title: Risoluzione dei problemi di temi e sfondi
description: Diagnosticare la propagazione del tema e i file di aspetto generati.
---
Controlla la sezione canonica `appearance` in `$XDG_CONFIG_HOME/argvus/config.json`, quindi ispeziona i file generati sotto `$XDG_CONFIG_HOME/argvus/data/generated`. I file legacy dot sono solo input per la migrazione.

`argvus-config` è l'unico scrittore sotto `data/generated/`. Per recuperare un tema o sfondo danneggiato, ricostruisci l'intera generazione dal documento canonico e applica il risultato invece di modificare direttamente i consumatori generati:

```sh
argvus config validate
argvus config apply
argvus sessionctl reload
```
`argvus theme-switch` (che esegue `argvus config theme-apply`) e `argvus config accent` sono orchestratori: validano, bloccano, mostrano il splash, eseguono il commit tramite `argvus-config` e poi riconciliano i consumatori esterni che non possono leggere `config.json`. Non scrivono mai `data/generated/`, e la transazione del tema non scrive i file GTK, di storage o `mode.css` tramite script separati. Usa `argvus config accent --apply` per rieseguire solo la riconciliazione degli accenti.

Le modifiche ufficiali del tema applicano i valori di aspetto salvati e aggiornano i componenti runtime interessati. Applicare un tema personalizzato riavvia la barra delle applicazioni, il reload di Hyprland, lo sfondo e i servizi del Pannello di Controllo. Passare da Sticky o Spaced riapplica la spaziatura, i bordi e i margini della barra delle applicazioni di quella modalità, quindi riavvia la barra delle applicazioni, i servizi di telemetria e il reload di Hyprland. Le eccezioni sono i colori delle notifiche di Dunst mentre Non Disturbare è attivo: il loro reload viene rimandato fino a quando DND non viene disattivato. Se un'altra superficie mantiene i vecchi colori, controlla che la build del tema sia attiva e che sia il tema che hai scelto, quindi esegui di nuovo i reload:

```sh
cat "${XDG_CONFIG_HOME:-$HOME/.config}/argvus/data/generated/theme/current/build.toml"
argvus config theme-runtime consumers
```
Ogni file a tema proviene dalla build: un tema che non è installato (nessun `theme.toml` sotto `/usr/share/argvus/themes/` o `~/.local/share/argvus/themes/`) o non viene visualizzato viene rifiutato, e il tema precedente rimane. `argvus theme check DIRECTORY` mostra il template e la riga di ogni errore di visualizzazione. Una barra Waybar ricarica i suoi colori solo con `reload_style_on_change` nel suo JSONC. Se un interruttore sembra bloccato, controlla se è ancora in esecuzione; registra se stesso in `theme-switch.active`, accanto al suo lock:

```sh
cat "${XDG_CACHE_HOME:-$HOME/.cache}/argvus/theme-switch.active"
ps -ef | grep -E 'theme-switch|theme-apply'
```
Un pid nel marker che è ancora attivo significa che lo switch non è ancora terminato. Uno switch attende al massimo 60 secondi per il blocco del tema e per la proiezione, e 5 secondi per i comandi di ricarica; un marker il cui processo è terminato è obsoleto e lo switch successivo lo ignora. Includere l'output dei comandi sopra quando si segnala il problema.

Se solo il greeter pre-login è bianco o utilizza il tema di riserva, confrontare la sorgente privata con la sua proiezione pubblica:

```sh
uid=$(id -u)
argvus config get /appearance/theme --raw
sudo cat "/var/lib/argvus/greeter/themes/$uid"
sudo cat "/var/lib/argvus/greeter/themes/$uid.accent" 2>/dev/null || true
```
Ripara la proiezione senza modificare la configurazione di greetd:

```sh
sudo argvus greeter-setup --sync-themes
```
Quel file è pubblicato per la pre-autenticazione dall'orchestratore dell'aspetto e vive al di fuori di `data/generated/`, quindi un semplice `argvus config apply` non lo aggiorna.

Se il saluto segnala `Invalid color name: 'null'` in una cache di Kitty, aggiorna il pacchetto `argvus terminal` e rigenera la cache aprendo di nuovo il saluto. Un accento canonico assente è il predefinito, non un colore di Kitty; il lanciatore del terminale corrente rifiuta JSON `null` e valori legacy non validi prima di scrivere `active_tab_foreground`.

Se il terminale predefinito smette di aprirsi dopo diversi cambiamenti di tema, controlla la cache derivata di Kitty invece di modificarla manualmente:

```sh
argvus config get /appearance/theme --raw
ls -l "${XDG_CACHE_HOME:-$HOME/.cache}/argvus/argvus-terminal"
argvus terminal --apply
argvus terminal --print-config
```
Il comando sopra legge la preferenza canonica. `.active-theme` è solo una proiezione di compatibilità. La cache viene rigenerata da `argvus terminal` e sincronizzata con l'applicazione del tema. Un `/usr/lib/argvus/loading-theme/splash` mancante o obsoleto indica un aggiornamento del pacchetto incompleto; ricostruire e installare i pacchetti coordinati prima di giudicare il risultato di una sessione live.

Se i bordi delle finestre mantengono il colore del tema precedente, non modificare manualmente il file Lua di Hyprland. I cambiamenti di tema e accento sono proiettati da `config.json`, poi `argvus sessionctl reload` ricarica Hyprland e riapplica i colori dei bordi attivi, inattivi e dei gruppi durante il ciclo di vita della sessione:

```text
config.json → projection plan → Hyprland reload → hyprctl reapply of active/inactive/group borders
```
Le *preferenze* dei bordi vengono modificate tramite `argvus config borders` (o il suo delegato `borders-switch.sh`) nel Centro di Controllo e nel Pannello di Controllo; il ricaricamento riapplica i colori confermati durante l'intero ciclo di vita della sessione e non li ricalcola.

Ispeziona il piano e i valori effettivi del compositore:

```sh
jq . "${XDG_STATE_HOME:-$HOME/.local/state}/argvus/config-projection.json"
hyprctl -j getoption general:col.active_border
hyprctl -j getoption general:col.inactive_border
argvus sessionctl reload
```
Se `hyprctl` non riesce a raggiungere il socket corrente di Hyprland, la proiezione del bordo viene segnalata come fallita e i componenti dipendenti non sono considerati ricaricati con successo. Controlla il journal della sessione utente prima di riprovare.
