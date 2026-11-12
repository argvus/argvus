---
title: Centro di Controllo
description: Configura le parti persistenti del tuo desktop ARGVUS.
---
Il Centro di Controllo è l'applicazione per le impostazioni di ARGVUS pensata principalmente per l'uso da tastiera. Usalo quando vuoi configurare il desktop piuttosto che eseguire un'azione una tantum.

## Centro di Controllo e Pannello di Controllo

| Usa… | Per… |
| --- | --- |
| **Centro di Controllo** | Temi, sfondi, effetti, layout, comportamento della barra delle applicazioni, font, applicazioni predefinite, impostazioni della tastiera e dell'input, lingua, regione e configurazione del sistema. |
| **Pannello di Controllo** | Stato attuale e azioni frequenti come volume, luminosità, rete, notifiche, controllo dell'alimentazione e della sessione. |

Il Pannello di Controllo può mostrare controlli rapidi per un'impostazione configurata permanentemente nel Centro di Controllo. Ad esempio, usa il pannello per un'azione rapida su aspetto o display, e il Centro di Controllo quando vuoi cambiare la configurazione del desktop dietro di esso. Vedi Pannello di Controllo e [Dove configurare le cose](/docs/user-guide/where-to-configure/).

## Aree principali

L'attuale schermata principale del Centro di Controllo offre queste aree:

La schermata principale è un catalogo di destinazioni raggruppate in sei schede. Ogni destinazione apre direttamente la sua pagina, e la ricerca la trova in qualsiasi scheda.
- **Aspetto** — personalizza il desktop: Stile del Desktop (tema, modalità di aspetto, colore accentato, sfondi, icone e caratteri), Stile dell'Applicazione (trasparenza del terminale e del launcher), Barra delle Applicazioni, Pannello di Controllo e Telemetria dei Widget.
- **Desktop & Input** — configura lo spazio di lavoro: Hyprland (Windows: Regole, Spazi e Modalità, animazioni, sfocatura, bordi, ombra e versione installata), Schermi, Tastiera & Scorciatoie e Mouse & Touchpad.
- **Preferenze** — impostazioni personali e regionali: Lingua & Regione (la lingua, fuso orario, formati e layout della tastiera), Data & Ora, Applicazioni Predefinite, Alimentazione e Sessione.
- **Hardware & Connettività** — dispositivi e connessioni: Panoramica dell'Hardware, Rete & Wi-Fi, Bluetooth, Audio e Memoria.
- **Amministrazione di Sistema** — gestire il sistema operativo: Pacchetti, Servizi, Avvio, Nome Host, Utenti e Gruppi, Firewall e Notifiche (Non Disturbare).
- **Impostazioni** — il Centro di Controllo stesso, elencato dopo le schede: Lingua (la lingua dell'utente) e Stile (icone decorative e trasparenza). Premere `s` per aprirlo.
- **Strumenti e Diagnostica** — strumenti per sviluppatori e manutenzione: ARGVUS Welcome, Progetti, Snippet (testo salvato per `argvus snippets`, digitato con `SUPER + ALT + N`), chiavi SSH (coppie di chiavi in `~/.ssh` gestite da `argvus keys`), chiavi GPG (portachiavi gestito da `argvus gpg`; gpg-agent usa pinentry-gtk), Servizi Dev, Dottore di Sistema e Informazioni su ARGVUS.

Quando Non Disturbare è attivo, i popup di notifica vengono chiusi e le nuove notifiche vengono conservate per dopo. Il Pannello di Controllo nasconde la cronologia delle notifiche fino a quando Non Disturbare non viene disattivato.

## ARGVUS Benvenuto

Scegli **Open ARGVUS Welcome** sotto **Tools & Diagnostics**, cerca `welcome`, o esegui `argvus welcome`. Welcome è integrato nel Control Center e utilizza la sua finestra singola. Premi Invio per aprire la pagina delle impostazioni selezionata; quando hai finito, torna alla guida con Esc. F6 Next spunta il passaggio corrente e apre il prossimo nella stessa finestra. Esc torna solo indietro, e Invio nella checklist apre sempre la pagina selezionata. O rivede nuovamente il passaggio, S salta senza spuntarlo, e F termina la guida anche con passaggi opzionali ancora pendenti. Annullare o tornare indietro non completa un passaggio. Il progresso esistente viene conservato.

La guida copre lingua e regione, display, hardware e driver, rete, audio, Bluetooth, aspetto, alimentazione, applicazioni predefinite e avvio della sessione. Non installa mai driver o pacchetti e non applica modifiche di sistema da sola. Il progresso è memorizzato in `$XDG_STATE_HOME/argvus/welcome/progress.json` (o `~/.local/state/argvus/welcome/progress.json`).

Dopo che il desktop è pronto, la sessione esegue `argvus welcome --if-needed` per TTY, ARGVUS Greeter e altri gestori di login. Apre Welcome a meno che i record di progresso per utente `opened` non indichino completamento o chiusura. Il primo rendering riuscito salva `opened` in `${XDG_STATE_HOME:-$HOME/.local/state}/argvus/welcome/progress.json`, impedendo la riapertura ai login successivi. Non dipende da `/etc/skel`. Gli account esistenti senza quel record ricevono comunque un'apertura automatica. I fallimenti prima del rendering consentono un altro tentativo al login successivo. `argvus welcome` manuale rimane disponibile.

Le pagine esatte del fornitore possono dipendere dai pacchetti installati e dai permessi. Non presumere che un dominio opzionale mancante sia un Control Center rotto.

Quando la griglia contiene più voci di quante ne possano entrare nello schermo, appare una barra di scorrimento lungo il margine destro per indicare che c'è altro in basso.

## Trovare un'impostazione

La schermata principale registra le impostazioni per titolo, categoria, identificatore e parole chiave. La ricerca non distingue gli accenti e accetta termini pratici come `theme`, `wallpaper`, `shortcut`, `mouse`, `monitor`, `network`, `user` e `DPI`. Le corrispondenze esatte del titolo sono classificate per prime, seguite da un prefisso del titolo, qualsiasi parola del titolo o della categoria che contenga il testo digitato, e infine le corrispondenze delle parole chiave — quindi digitare qualsiasi parte di una parola (non solo l'inizio) può comunque trovare la destinazione.

Il menu visibile è consapevole delle capacità. Una rotta può essere assente quando il suo provider non è installato, quando l'hardware non è disponibile o quando la sessione corrente non può fornire la capacità richiesta. La ricerca è quindi anche un modo utile per scoprire cosa questa installazione può configurare.

L'albero principale delle impostazioni attualmente include:

| Area | Destinazioni correnti |
| --- | --- |
| Aspetto | Stile del desktop: tema, modalità aspetto, colore accentuato, sfondi, icone e caratteri; Stile dell'applicazione: trasparenza del terminale e del launcher; Barra delle applicazioni; Pannello di controllo; Telemetria dei widget |
| Desktop & Input | Hyprland (Windows: Regole, Spazi e Modalità, animazioni, sfocatura, bordi e ombre), Display, Tastiera & Scorciatoie, Mouse & Touchpad |
| Preferenze | Lingua & Regione (lingua, fuso orario, formati, locali di sistema e layout della tastiera), Data & Ora, Applicazioni predefinite, Risparmio energetico, Sessione |
| Hardware e Connettività | Panoramica dell'hardware (sintesi, CPU, GPU, memoria, dispositivi), Rete e Wi-Fi, Bluetooth, Audio, Memoria |
| Amministrazione di Sistema | Pacchetti, Servizi, Avvio, Nome Host, Utenti e Gruppi, Firewall, Notifiche |
| Impostazioni | Lingua (lingua utente), Stile (icone e trasparenza) |
| Strumenti e Diagnostica | Progetti, Frammenti, Chiavi SSH, Chiavi GPG, Servizi Dev, Dottore del Sistema, Informazioni su ARGVUS |

Alcune di queste sono pagine complete e alcune sono sottopagine raggiunte tramite ricerca o panoramica di un dominio. L'elenco esatto è determinato dai provider ARGVUS installati.

Ogni pagina mostra il proprio percorso nell'intestazione, ad esempio `Control Center > Hardware & Connectivity > Audio`. Il percorso inizia con la scheda Home che elenca la pagina, quindi la stessa pagina appare uguale dalla Home, dalla ricerca e dal CLI. I nomi root e delle schede sono tradotti: l'interfaccia portoghese mostra `Central de Controle > Hardware e conectividade > Áudio`.

## A cosa serve ogni area

### Aspetto

L'aspetto è l'area principale di personalizzazione. Il suo gruppo Desktop contiene il tema (con la modalità Sticky o Spaced), la modalità di aspetto, il colore di accento, gli sfondi, il tema delle icone e la pagina dei Font, mentre il gruppo Stile Applicazioni contiene la trasparenza del terminale e del launcher. Barra delle applicazioni, Pannello di Controllo e Telemetria dei Widget sono elencati direttamente sotto Aspetto nella home e riutilizzano la pagina di trasparenza condivisa di Aspetto per le proprie superfici. Blur è un'impostazione globale, sotto Hyprland > Blur, e si applica a tutte le superfici ARGVUS che hanno il blur abilitato. Quella pagina modifica il suo interruttore Abilita e i valori di blur di Hyprland (Dimensione, Passaggi, Luminosità, Rumore, Contrasto, Vividezza e Oscurità vividezza) come bozza, e nulla cambia fino a quando non viene selezionato **Applica**. Le animazioni funzionano allo stesso modo sotto Hyprland > Animazioni: il suo interruttore Abilita, le curve personalizzate (Bézier, Spring, Lineare e Ease-out) e le cinque categorie di animazione formano una bozza che è applicato insieme a **Apply**. Bézier e Spring hanno parametri modificabili (i punti Bézier e la massa, la rigidità e l'ammortizzazione della molla); Linear ed Ease-out sono curve fisse, quindi la loro pagina elenca solo i punti di controllo e non può essere modificata. Le scelte di stile seguono ciò che disegna Hyprland 0.56.2: Opening e Closing offrono slide, popin e gnomed; layer entrance e exit offrono slide, popin e fade; e workspace leaves offre slide, fade, slidefade, slidevert e slidefadevert. Questi controlli coordinano con i componenti di apparizione e sessione invece di modificare solo la finestra del Control Center.

### Barra delle applicazioni

La barra delle applicazioni configura la posizione del pannello, lo spazio intorno ad esso e alle sue finestre, la sua altezza e il bordo, il gruppo di icone di utilità e i blocchi di data/ora, oltre alla propria trasparenza. È elencata sotto **Appearance** nella home. In modalità Sticky, **Spaces > Bottom** predefinito è `2`; Spaced mantiene la sua baseline pixel `18`.

**Altezza** sceglie 24, 28 (impostazione predefinita), 32 o 36 pixel. **Bordo** ha un interruttore **Abilita** e uno **Spessore** da 1 a 3 pixel, con 1 come predefinito. Con il bordo disattivato, la finestra della barra delle applicazioni, il gruppo a destra e il lettore MPRIS non hanno bordo. Queste modifiche sono una bozza fino a quando non viene selezionato **Applica**. L'altezza e il bordo applicati vengono mantenuti quando il tema cambia; finché non vengono applicati, li decide il tema.

La pagina **Icone** apre **Moduli opzionali**, **Presentazione spazio di lavoro**, **Launcher** e **Utility**. I moduli opzionali possono essere mostrati o nascosti individualmente: lettore audio, Focus, launcher, rete, profilo di alimentazione, layout della tastiera, memoria, CPU e letture della temperatura, data, orologio, batteria, area di notifica, notifiche, dispositivi rimovibili, registrazione, Bluetooth, ingresso/uscita audio e mantenimento attivo. Gli spazi di lavoro rimangono disponibili e possono mostrare i loro nomi (predefinito) o le icone delle applicazioni. Il titolo della finestra attiva, Impostazioni e le azioni di Alimentazione sono sempre disponibili. La visibilità del launcher è controllata nei moduli opzionali; il suo sottomenu cambia l'icona personalizzata. Le modifiche hanno effetto dopo **Applica**, che riavvia la barra delle applicazioni.

### Pannello di controllo

Il Pannello di controllo configura la visibilità e l'ordine delle sue schede di azioni rapide, oltre alle proprie sessioni e alla trasparenza.

La pagina si apre e accetta input non appena vengono letti i parametri della scheda. Le schede che dipendono dall'hardware (Bluetooth e Luminosità) vengono verificate in background: fino a quando tale controllo non è completato, mantengono la disponibilità vista l'ultima volta in questa finestra, e l'elenco si aggiorna automaticamente quando arriva il risultato.

### Telemetria dei Widget

La Telemetria dei Widget configura le sessioni e la trasparenza del widget delle informazioni di sistema.

### Caratteri

A Caratteri si accede da **Aspetto > Stile Desktop > Caratteri**. Permette di selezionare una famiglia e una dimensione per la barra delle applicazioni, la telemetria/informazioni di sistema, il Pannello di Controllo, l'interfaccia di sistema ARGVUS, le applicazioni, il terminale e il browser. Espone anche le impostazioni di rendering dei caratteri come antialiasing, hinting, modalità subpixel e DPI. La famiglia primaria predefinita è IBM Plex Mono; i singoli obiettivi possono essere modificati indipendentemente e ripristinati dalla pagina Caratteri.

### Applicazioni predefinite

Le applicazioni predefinite selezionano i programmi installati per ruoli come terminale, file manager, editor di testo, editor da terminale, browser, visualizzatore di immagini, visualizzatore PDF, lettore video, lettore audio, strumento per archivi e launcher. La selezione viene salvata da ARGVUS e può aggiornare le associazioni XDG standard. I file manager TUI usano l'editor da terminale selezionato quando si aprono file di testo; gli editor di testo grafici rimangono predefiniti per le associazioni di file del desktop.

### Snippet

Gli snippet sono la pagina per il testo che `argvus snippets` digita con `SUPER + ALT + N`. Si trova in **Strumenti & Diagnostica → Snippet** (cerca `snippets` o esegui `argvus control-center snippets`).
- **Nuovo snippet** — digita un **Nome** e un **Contenuto**, poi **Salva snippet**. Un nome già esistente sostituisce il contenuto di quello snippet.
- **Snippet salvati** — elencati con la loro posizione (`1  email`) e il contenuto su una sola riga. Premi Invio sul contenuto per modificarlo. **Tipo** indica il tipo di snippet, e **Rimuovi** chiede conferma.
- **Apri selettore** — apre lo stesso selettore di `SUPER + ALT + N`.

La digitazione inizia 3 secondi dopo aver premuto Invio, perché il Centro di Controllo stesso ha il focus in quel momento. Clicca sul campo che deve ricevere il testo entro quel tempo. La pagina modifica solo contenuti su una riga; uno snippet su più righe può essere creato in un terminale con `argvus snippets add`.

La pagina legge le voci da `argvus-config` e le scrive tramite `argvus snippets`, quindi il launcher mantiene la validazione. Quando `argvus snippets` non è installato, la pagina lo segnala.

### Chiavi SSH

Le chiavi SSH sono la pagina per le coppie di chiavi che `argvus keys` gestisce. Si trova in **Strumenti e Diagnostica → Chiavi SSH** (cerca `ssh keys` o esegui `argvus control-center ssh-keys`).

La pagina è composta da quattro parti, dall'alto verso il basso:

```
Search        filters the key pairs by name or comment
Generate      name, type and the Generate key pair action
Keys          one block per key pair; a scrollbar appears when they do not fit
Actions       Refresh and Unload all keys from agent
```
#### Come usarlo

1. Apri **Centro di Controllo → Strumenti e Diagnostica → Chiavi SSH**. Dalla Home, puoi anche cercare `ssh keys`.
2. **Trova una chiave:** premi `/`, poi digita per mostrare solo le coppie il cui nome o commento corrisponde. `Enter` termina la ricerca ed esegue la riga evidenziata, come nelle altre liste, quindi controlla la riga prima di premerla; `Esc` cancella la ricerca.
3. **Crea una coppia:** in **Genera**, seleziona **Nome**, premi `Enter`, digita il nome e premi di nuovo `Enter`. Seleziona **Tipo** e premi `Enter` per passare tra `ed25519` e `rsa`. Seleziona **Genera coppia di chiavi** e premi `Enter`. La nuova coppia appare subito sotto **Chiavi**, senza passphrase.
4. **Copia la chiave pubblica:** nel blocco della coppia, seleziona **Copia chiave pubblica** e premi `Enter`. Il contenuto del file `.pub` va negli appunti di Wayland, pronto per essere incollato in un `authorized_keys` del server o in un host Git. La chiave privata non viene mai copiata. Questo richiede `wl-clipboard` (`wl-copy`); senza di esso, la pagina segnala che la chiave non è stata copiata.
5. **Carica una coppia nell'agente:** nel blocco della coppia, seleziona **Carica nell'agente** e premi `Enter`. Una chiave senza passphrase viene caricata immediatamente, senza finestra. Una chiave con passphrase apre un terminale ARGVUS, dove OpenSSH la richiede; chiudi il terminale con `Enter` quando dice che la chiave è caricata. La pagina controlla l'agente ogni pochi secondi mentre è aperta, quindi la riga cambia da sola in **Scarica dall'agente**. Per aggiornare subito, usa **Aggiorna** in **Azioni**.
6. **Rimuovere una chiave dall'agente:** nel blocco della coppia, seleziona **Scarica dall'agente**. Per rimuovere ogni chiave, utilizzare **Scarica tutte le chiavi dall'agente** in **Azioni**. I file chiave rimangono sul disco.
7. **Elimina una coppia:** nel blocco della coppia, seleziona **Elimina** e premi `Enter`. La conferma si concentra su **Annulla**, quindi un `Enter` vagante non fa nulla. Premere `y` per confermare oppure `n`/`Esc` per annullare. Non è possibile eliminare una chiave caricata nell'agente; scaricalo prima.

Muoversi: `↑`/`↓` o `j`/`k` si sposta nell'elenco, `Enter` attiva una riga, `Esc` torna indietro e `?` mostra l'aiuto. `Tab` sposta lo stato attivo su **Azioni**, dove `←`/`→` sceglie un pulsante e `Enter` lo esegue. `↑`/`↓` riportano il focus sull'elenco.

Le passphrase non vengono mai digitate nel Centro di controllo e non vengono mai passate come argomenti di comando. La pagina legge la lista con `argvus keys list` e scrive solo tramite `argvus keys`. Quando `argvus-keys` non è installato, la pagina lo dice.

#### Risoluzione dei problemi

| Sintomo | Causa e soluzione |
| --- | --- |
| `Could not open a connection to your authentication agent` dopo **Carica nell'agente** | Il terminale non ha ricevuto il socket dell'agente. Hyprland mantiene l'ambiente con cui è stato avviato, e l'agente potrebbe essere stato avviato successivamente. Aggiorna `argvus-launcher`, che ricade su `$XDG_RUNTIME_DIR/ssh-agent.socket`, e controlla `systemctl --user status argvus-ssh-agent.service`. |
| **Agente** mostra `Agent unavailable` | Il launcher non riesce a raggiungere l'agente. Controlla il servizio come sopra. |
| La pagina dice `argvus-keys is not installed` | Aggiorna il pacchetto `argvus`: `argvus keys` è un sottocomando di esso. |
| **Elimina** mostra un errore | La chiave è caricata nell'agente e rimane lì. Scaricala prima, poi eliminala di nuovo. |

### Chiavi GPG

Le chiavi GPG sono la pagina per il portachiavi che `argvus gpg` gestisce. Si trova in **Strumenti e Diagnostica → Chiavi GPG** (cerca `gpg keys` o esegui `argvus control-center gpg-keys`).

La pagina ha queste sezioni, dall'alto verso il basso:

```
Pinentry         the pinentry gpg-agent uses, for information only: pinentry-gtk
Generate key ▾   full name, e-mail, comment, algorithm, key strength, expiration date,
                 passphrase, and the Generate key and Cancel actions (expanded at first)
Import ▾         Import public key… and Import private key… (expanded at first)
Keys ›           the keyring, on its own page, with the search at the top
```
`▾` segna una sezione espansa e `▸` una collassata. `Enter` o `Space` la espandono o la collassano.

#### Come usarlo

1. Apri **Centro di Controllo → Strumenti & Diagnostica → Chiavi GPG**. Dalla Home, puoi anche cercare `gpg keys`.
2. **Pinentry** è fissato a `pinentry-gtk`. La pagina lo mostra solo. La prima volta che viene generata una chiave, ARGVUS scrive quel pinentry in `gpg-agent.conf`, tra le sue linee di marcatori, e mantiene le tue altre linee. Se hai già scritto una linea `pinentry-program` al di fuori di quei marker, la generazione si interrompe con un errore in modo che tu possa rimuovere quella linea.
3. **Crea una chiave:** in **Genera chiave ▾**, digita **Nome completo** e **E-mail**. Entrambi sono obbligatori, e **Genera chiave** resta disabilitato finché entrambi non contengono testo. **Commento** è opzionale. L'ID utente diventa `Name (comment) <e-mail>`.
- **Algoritmo** passa tra **RSA** e **DSA ElGamal**. Una chiave DSA ottiene una sottchiave di crittografia ElGamal; una chiave RSA ottiene una sottchiave RSA.
- **Forza della chiave** cicla le dimensioni dell'algoritmo: RSA 1024, 2048 e 4096; DSA 1024, 2048 e 3072. gpg non crea chiavi DSA superiori a 3072 bit, quindi 4096 non è offerto per DSA.
- **Data di scadenza** apre la propria pagina. Inserisci la data nel formato della tua lingua: `DD/MM/YYYY` in portoghese, spagnolo, francese, italiano, tedesco e russo; `YYYY/MM/DD` in inglese, cinese, giapponese e coreano. La data deve essere successiva ad oggi. Attiva **Mai scade** per rendere la chiave permanente; il campo della data viene quindi disabilitato e non utilizzato.
- **Frase d'accesso** apre la propria pagina. **Proteggi con una frase d'accesso** è attivo per impostazione predefinita; digita la **Frase d'accesso** e poi **Conferma frase d'accesso**. Le due devono corrispondere. Disattivando la protezione si cancellano entrambi i campi, e la pagina richiede conferma prima di generare una chiave non protetta.
4. Seleziona **Genera chiave** e premi `Enter`. La chiave ottiene una chiave di firma e una sottinchiave di crittografia. La passphrase viene inviata al launcher tramite il suo input standard, mai come argomento del comando. Se la generazione fallisce, gli altri campi rimangono, ma le passphrase vengono cancellate e devono essere digitate di nuovo.
5. **Annulla** cancella tutti i campi del modulo e restituisce date e dimensioni ai valori predefiniti.
6. **Importa:** in **Importa ▾**, scegli **Importa chiave pubblica…** o **Importa chiave privata…**. Il gestore di file si apre in modo da poter scegliere il file. Una chiave privata richiede prima una conferma, e la sua passphrase viene richiesta da gpg quando la chiave viene usata. Se il gestore di file è grafico, non può segnalare il file alla pagina; la pagina apre quindi un campo dove digiti il percorso.
7. **Chiavi ›** apre il portachiavi. Ogni chiave mostra la sua identità, impronta digitale, tipo, scadenza e se la chiave privata è nel portachiavi. Ogni chiave ha queste azioni:
   - **Modifica chiave ›** apre la pagina della chiave stessa (descritta di seguito).
- **Esporta chiave pubblica…** chiede un percorso per il file, proposto come `~/<fingerprint>.asc`. Il file viene creato con modalità `600`. Un file esistente non viene mai sovrascritto.
- **Esporta chiave privata…** appare solo per le chiavi con una parte privata. Chiede conferma, poi un percorso per il file, proposto come `~/<fingerprint>-private.asc`. Il file viene creato con modalità `600`.
- **Copia chiave pubblica** e **Copia impronta** copiano negli appunti di Wayland con `wl-copy`.
- **Elimina chiave…** chiede conferma, con il focus su Annulla. Rimuove la chiave e la sua parte privata.
8. **Trova una chiave:** nella pagina **Chiavi**, la barra di ricerca in alto filtra le chiavi. Digita per restringere la lista; la ricerca corrisponde all'identità (che include il commento) e all'impronta digitale. `Esc` cancella la ricerca.
9. **Modifica chiave ›** mostra i dettagli della chiave, poi queste azioni:
   - **Data di scadenza ›** modifica la scadenza della chiave e di ciascuna delle sue sottochiavi. Usa **Mai scade** per rimuovere la scadenza. **Applica** salva la modifica.
- **Cambia passphrase** richiede la vecchia e la nuova passphrase nella finestra pinentry.
- **Firma chiave con ›** certifica questa chiave con una delle tue altre chiavi che ha una parte privata. Viene richiesta la passphrase del firmatario. La pagina chiede prima la conferma.
- **Revoca chiave ›** elenca i motivi: nessun motivo, chiave compromessa, chiave sostituita, chiave non più utilizzata. Ognuno richiede conferma. Una chiave revocata non può più essere utilizzata.
- **Fiducia** imposta quanto ti fidi della chiave: non impostata, marginale, completa o ultima.
- **Pubblica sul server chiavi…** invia la chiave pubblica al server chiavi impostato in `gpg.conf`, dopo conferma. **Recupera dal server chiavi** scarica la chiave da quel server chiavi.

Spostarsi: `↑`/`↓` o `j`/`k` si muovono nella lista, `Enter` attiva una riga, `Esc` torna indietro e `?` mostra l'aiuto.

Le frasi segrete devono essere digitate solo nei campi mascherati della pagina delle frasi segrete, o nella finestra pinentry. La pagina invia la frase segreta di generazione a `argvus gpg` sull'input standard, e il launcher la passa a gpg su un file descriptor, quindi non appare mai nella lista dei processi, in un argomento del comando o nei log. Le modifiche alle chiavi vengono eseguite in background; mentre una è in esecuzione, le altre attendono. La pagina legge il portachiavi con `argvus gpg list` e scrive solo tramite `argvus gpg`. Quando `argvus gpg` non è installato, la pagina lo segnala.

Non ancora presente nella pagina: ricerca della chiave sul keyserver per nome o e-mail. Usare **Recupera dal keyserver** con un fingerprint noto.

#### Risoluzione dei problemi

| Sintomo | Causa e soluzione |
| --- | --- |
| La pagina dice `argvus gpg is not available` | Il `argvus-launcher` installato precede il comando GPG. Esegui `argvus gpg list` in un terminale: `unknown command` significa che il pacchetto deve essere ricostruito e installato. Anche le chiavi necessitano `gnupg`. |
| **Genera chiave** resta disabilitato | Digita sia **Nome completo** che **E-mail**. |
| Una riga di stato dice `argvus-gpg: pinentry-gtk is not installed` | Installa il pacchetto `pinentry`, che fornisce `pinentry-gtk`. |
| La generazione si ferma con un errore `pinentry-program` | `gpg-agent.conf` imposta un pinentry al di fuori del blocco ARGVUS. Rimuovi quella riga; poi ARGVUS scrive la propria. |
| **Pinentry** mostra `Not set` | gpg-agent usa il proprio default incorporato fino a quando la prima chiave non viene generata dalla pagina. |
| La pagina dice che `argvus-launcher` installato è più vecchio del Centro di Controllo | Reinstalla `argvus-launcher`: esegui `make build` e poi `make install` nel suo repository, e riapri la pagina. Le azioni principali restano disattivate fino a quando le versioni non corrispondono. |
| Il gestore file si apre, ma la pagina chiede un percorso | Il gestore file è grafico e non può riportare il file scelto. Digita il percorso nel campo. |
| **Copia chiave pubblica** dice che non è possibile copiare | `wl-copy` non è installato o la sessione Wayland non è disponibile. Installa `wl-clipboard`. |
| **Modifica chiave ›** dice che la chiave non è più nel portachiavi | Il portachiavi è cambiato da quando la pagina è stata aperta. Torna a **Chiavi ›** e aggiorna. |
| Una riga di stato mostra `argvus-gpg: ...` | Il launcher ha rifiutato l'azione e il testo dà il motivo, come una scadenza non valida, un file già esistente o un `--yes` mancante. |
| Un'azione è in attesa e non succede nulla | La finestra di inserimento del PIN è nascosta o non è stata aperta. Controlla se la sessione espone `WAYLAND_DISPLAY` o `DISPLAY`. |
| L'importazione è rifiutata | Il file non contiene alcuna chiave del tipo selezionato. Le chiavi pubbliche passano attraverso **Importa chiave pubblica…**; le chiavi private attraverso **Importa chiave privata…**. |

### Lingua e Regione

Questa scheda elenca Lingua (locale regionale), Fuso Orario, Data e Ora, Locali di Sistema e Layout della Tastiera, e ciascuno si apre direttamente dalla home. Le pagine della tastiera contengono anche variante e mappa dei tasti della console. La ricerca dalla home mostra anche scorciatoie da tastiera e impostazioni del mouse/touchpad come percorsi diretti alle impostazioni.

L'elenco delle lingue rileva i cataloghi installati e mostra i nomi nativi per tutte e dieci le lingue. Abilitare il locale di sistema UTF-8 corrispondente prima di selezionarlo.

Sia **Preferenze > Lingua e Regione > Lingua** sia **Impostazioni > Lingua** salvano la stessa preferenza utente: `LANG` in `$XDG_CONFIG_HOME/locale.conf` (predefinito `~/.config/locale.conf`), ad esempio `LANG=en_US.UTF-8`. Commenti e variabili regionali sono preservati. Non viene scritto alcun file globale o impostazione linguistica ARGVUS parallela. Un utente non configurato riceve l'inglese; i file legacy `session.language` e `data/language` vengono ignorati. Effettuare il logout e accedere nuovamente per applicare la lingua all'intera sessione e alle applicazioni.

**Locali di sistema** modifica `/etc/locale.gen` e esegue `locale-gen`, quindi l'applicazione richiede l'autorizzazione dell'amministratore. Una notifica persistente **[WARN]** sopra il piè di pagina identifica le pagine con impostazioni amministrative. La stessa notifica copre fuso orario, data/ora, nome host, mappa tastiera della console, modifica di account e gruppi, firewall, servizi di sistema, governor della CPU, profili e politiche di alimentazione, configurazione di avvio, operazioni sui pacchetti, snapshot e modifiche della rete protetta. Le preferenze personali, i servizi dell'utente e le pagine di sola lettura non ricevono questa notifica. La notifica è informativa; l'autorizzazione viene richiesta solo quando un'azione lo richiede.

### Hardware e display

Quando le capacità corrispondenti sono installate, il Centro di Controllo può mostrare informazioni sull'hardware come CPU, GPU, memoria, alimentazione e dispositivi. La pagina di input configura il comportamento del mouse e del touchpad; l'area di visualizzazione gestisce la risoluzione del monitor, la frequenza di aggiornamento, la scala, la posizione, l'orientamento, il display principale, VRR e HDR dove supportati dalla sessione in esecuzione. Le modifiche al display si applicano immediatamente. Dopo una modifica che può rendere lo schermo inutilizzabile (risoluzione o frequenza di aggiornamento, posizione, mirroring, disabilitazione di un monitor, profondità del colore) o dopo **Applica**, il Centro di Controllo chiede se mantenerla, con il focus su **Ripristina**: premere `y` per mantenerla; `n`, `Esc` o `Enter` per ripristinarla, e senza una risposta entro 15 secondi la configurazione precedente torna automaticamente. La rimozione della configurazione salvata di un monitor scollegato e l'eliminazione di un profilo richiedono conferma; i profili aprono la propria pagina con **Applica profilo**, **Rinomina** e **Elimina profilo**. Il layout è salvato attraverso `argvus config monitor-set`, `monitor-primary`, `monitor-workspaces` e `monitor-remove`, quindi la sezione canonica `/displays` di `config.json` è l'unica fonte del layout del monitor. Solo il catalogo dei profili rimane in `display/config.toml`.

### Connettività e audio

L'area di rete espone lo stato e le pagine di connessione del provider di rete installato, inclusi Wi-Fi, Ethernet, VPN, DNS e percorsi proxy quando disponibili. Il firewall è elencato sotto **Amministrazione di Sistema**. Il Bluetooth è condizionato dal supporto Bluetooth. L'area audio è fornita quando la sua capacità opzionale è abilitata e si integra con il servizio audio installato.

La home di **Network & Wi-Fi** e la sua pagina **Stato** si aggiornano man mano che ogni query di NetworkManager risponde, invece che dopo quella più lenta: prima la lista dei dispositivi, poi il radio, gli indirizzi e il DNS delle interfacce connesse, le connessioni salvate e le reti Wi-Fi che NetworkManager conosce già (una nuova scansione viene eseguita solo dalla pagina Wi-Fi con `r`). Fino all'arrivo della lista dei dispositivi, le righe dicono *Caricamento...*, e l'interruttore Wi-Fi viene mostrato solo quando il computer ha una scheda Wi-Fi. Iniziare un'azione (l'interruttore Wi-Fi, la connessione, DNS) mentre una pagina si sta aggiornando sostituisce l'aggiornamento, e la pagina si aggiorna di nuovo quando l'azione termina.

#### Connessioni, ispirato da nmtui

**Rete e Wi-Fi > Connessioni** elenca tutti i profili di connessione di NetworkManager, raggruppati per tipo, con il dispositivo su cui sono attivi (`★ Active · eno1`), se sono una porta di un bridge, bond o team, e se sono solo in lettura. `Enter` apre il profilo nell'editor, `a` lo connette o disconnette, `/` filtra per nome e `r` aggiorna. **Attiva una connessione** è la stessa lista in cui `Enter` porta su o giù un profilo, come in `nmtui`. Le pagine di dettaglio di un'interfaccia, di una rete Wi-Fi salvata e di una VPN hanno una riga **Modifica connessione** che apre il loro profilo.

**Aggiungi connessione** crea profili Ethernet, Wi-Fi, Hotspot, DSL (PPPoE), Bridge, Bond, Team, VLAN, tunnel IP, Loopback e WireGuard, e un profilo VPN per ogni plugin VPN NetworkManager installato (OpenVPN ha il suo modulo; altri plugin modificano le impostazioni come voci `key=value`). Importa anche un file WireGuard `.conf` o un file plugin come un `.ovpn` (`nmcli connection import`). Team necessita del pacchetto `libteam`, DSL necessita di `ppp` e una VPN del suo plugin (per esempio `networkmanager-openvpn`); senza di essi la riga è disabilitata e indica cosa installare. WEP, InfiniBand, MACsec e Open vSwitch non sono offerti.

L'editor è una lista di sezioni pieghevoli: **Generale** (nome, interfaccia, connetti automaticamente, priorità, disponibile per tutti gli utenti, misurato), le impostazioni proprie del tipo, **IPv4** e **IPv6** (metodo, indirizzi, gateway, server DNS, domini di ricerca, rotte, ignora DNS o rotte automatiche, non usare mai come rotta predefinita, metrica della rotta, estensioni di privacy, generazione indirizzo). Le scelte e i numeri cambiano con `←/→`; `Enter` digita un valore; le liste (indirizzi, DNS, rotte, IP consentiti) aprono la propria pagina, dove `Enter` modifica una voce, cancellarla la rimuove e **Aggiungi voce** ne aggiunge una. I campi che non si applicano al metodo scelto sono mostrati disabilitati. Un bridge, bond o team sceglie le sue **Porte** Ethernet in una sotto-pagina; WireGuard modifica i suoi **Peer** in una sotto-pagina e può **Generare coppia di chiavi** (con `wg`, da `wireguard-tools`, una dipendenza del pacchetto `argvus`); la chiave pubblica della chiave privata è mostrata sotto di essa.

Niente raggiunge NetworkManager fino a **Salva**, che resta disabilitato finché nulla è stato modificato o finché un campo è invalido, e poi indica quale campo e perché. Solo le impostazioni che sono state modificate vengono salvate. Salvare una connessione attiva richiede prima conferma e poi la riconnette, così le nuove impostazioni si applicano subito (questo può interrompere brevemente la rete). `Esc` con modifiche non salvate chiede prima di scartarle. **Elimina connessione**, nella zona pericolosa, chiede conferma con il focus su Annulla; eliminare un bridge, bond o team elimina anche le sue porte.

Le password, le chiavi private e le chiavi precondivise non vengono mai lette: le loro righe mostrano *Invariato* fino a quando non viene digitato un nuovo valore, e il nuovo valore viene visualizzato mascherato. Vengono scritte tramite l'input standard di `nmcli connection edit`, mai come argomenti di comando. Le modifiche vengono eseguite come utente della sessione: ai membri locali di `wheel` è consentito farlo dalla regola polkit di NetworkManager, e agli altri utenti viene richiesta la password dell'amministratore dall'agente polkit della sessione.

### Potenza e sessione

L'alimentazione fornisce il controllo dell'alimentazione del sistema e le pagine delle politiche disponibili per l'installazione, incluso l'interruttore Mantieni attivo. Quando abilitato, Mantieni attivo impedisce alla politica inattiva ARGVUS di avviare automaticamente il blocco dello schermo e i timer di spegnimento del display. Sessione espone lo stato della sessione ARGVUS, i componenti, l'avvio automatico, la diagnostica e i registri. Su **Alimentazione**, la barra **Azioni** sotto l'elenco contiene **Sospendi**, **Ibernazione** e **Aggiorna** (anche `r` aggiorna): `Tab` o `←/→` si spostano su di essa, `Enter` esegue il pulsante selezionato, `↑`/`↓` riportano il focus alla lista e `Esc` torna indietro. Sospendi e Ibernazione chiedono prima conferma. Queste pagine possono richiedere permessi di sistema e devono essere distinte dalle azioni rapide nel Pannello di controllo.

### Strumenti di sistema

La sezione del sistema può mostrare informazioni di avvio, pacchetti, servizi, archiviazione, diagnostica, amministrazione utente/gruppo e informazioni sul sistema. Questi sono strumenti amministrativi o diagnostici; una pagina può essere di sola lettura o richiedere autorizzazione a seconda dell'operazione.

**Servizi** ha quattro pagine: unità di sistema, unità utente, unità fallite e il registro (Log). Ogni elenco di unità mostra una linea `Filter:`, che `f` cicla, e una casella `Search:` bordata, aperta con `/`. Ogni unità mostra il proprio stato Attivo e Abilitato come punti pieni o vuoti. Nei Log, `b` passa tra l'avvio corrente e quello precedente, `u` cicla l'unità e `p` cicla la priorità; la linea `Filter:` mostra i valori in uso. `Enter` apre un'unità o una voce del log.

**Dev Services** segnala tre sorgenti dedicate agli sviluppatori in un'unica pagina di sola lettura: i servizi systemd dell'utente corrente, le porte TCP attualmente in ascolto (`ss`) e i container in esecuzione da qualsiasi runtime di container installato (Podman, Docker). Compare sulla griglia Home solo quando almeno una di queste è disponibile sul sistema — una macchina senza sessione utente systemd, senza Podman e senza Docker non mostra la voce. `r` aggiorna le tre sorgenti in modo indipendente, quindi una di esse lenta o malfunzionante (ad esempio un runtime di container che ha smesso di rispondere) non impedisce alle altre di mostrare i propri dati.

**Pacchetti** è raggruppato per origine. **Repository ufficiali** e, quando è installato un helper AUR (`paru` o `yay`), **AUR (Arch User Repository)** aprono ciascuno **Installa**, **Rimuovi** e **Aggiorna**; seguono **Cache**, **Cronologia**, **Downgrade** e **Mirror**. I tre compiti condividono uno schermo: la casella **Cerca** fissata in alto prende le lettere che digiti, e i pacchetti sono elencati sotto di essa nel blocco **Risultati** (**Aggiornamenti** su Aggiorna), dove una lista vuota indica il motivo (per esempio **Nessun risultato**, **Nessun elemento corrispondente** o **Nessun aggiornamento in sospeso**). **Installa** cerca nei repository (`pacman -Ss`) o nell'AUR quando premi `Enter` nella barra di ricerca in alto; **Rimuovi** elenca i pacchetti installati (la lista AUR mostra i pacchetti costruiti fuori dai repository, `pacman -Qm`, e mentre viene mostrato il menu AUR lasciano la lista ufficiale); **Aggiorna** elenca gli aggiornamenti in sospeso (`pacman -Qu`, o `paru -Qua`/`yay -Qua` per l'AUR). Su un pacchetto, `Enter` esegue l'attività della schermata e `→` apre la pagina del pacchetto, con **Installa** (o **Reinstalla**) e, nella sua **Zona di pericolo**, **Rimuovi**; `←` o `Esc` torna indietro. **Repository ufficiali → Rimuovi** inizia con **Orfani**, dove `Space` contrassegna i pacchetti (`[x]`), `Enter` ne apre uno e **Rimuovi contrassegnati** li rimuove insieme. Dall'elenco dei risultati dell'aggiornamento, `Tab` si sposta sulla barra **Azioni** sotto l'elenco (**Aggiorna tutto** e **Aggiorna database**, o **Aggiorna tutto (AUR)** su AUR), `←`/`→` si sposta tra i suoi pulsanti, `Enter` esegue quello focalizzato e `↑`/`↓` (o digitando) riporta il focus sulla lista; **Aggiorna tutto** è disattivato e viene saltato quando non è elencato alcun aggiornamento. Il breadcrumb e la ricerca globale seguono lo stesso percorso, ad esempio **Packages > Official Repositories > Remove > Orphans**. Le pulizie della cache si trovano nella zona di pericolo della pagina Cache e **Mirror → Configura mirror** imposta le opzioni del riflettore prima **Genera anteprima**. Ogni operazione sui pacchetti mostra il suo piano, o per l'AUR l'avviso PKGBUILD, e chiede conferma (`y` conferma, `n` o `Esc` annulla) prima di eseguirla, e la sua uscita viene mostrata mentre viene eseguita. Le operazioni AUR eseguono l'helper come il tuo utente (`-S <package>` per installare o aggiornare uno, `-Sua` per aggiornare tutti); le rimozioni AUR utilizzano lo stesso `pacman -Rns` dei pacchetti ufficiali. Quando altri pacchetti installati dipendono dal pacchetto che rimuovi (ufficiale o AUR), la conferma li nomina e offre due modi: **Rimuovi solo** il pacchetto selezionato (`pacman -Rdd`, che salta i controlli delle dipendenze, quindi i pacchetti che ne hanno bisogno potrebbero smettere di funzionare) o **Rimuovi con dipendenti** (`pacman -Rcns`, con il numero di pacchetti che rimuove). Il focus inizia su **Annulla**, che mantiene tutti i pacchetti; `Enter` esegue la riga sul focus e non c'è alcun collegamento `y` in questa scelta.

La barra superiore di **Cerca** contiene la query del pacchetto. `/` lo focalizza senza cancellarne il testo; anche la digitazione sposta l'attenzione lì. Modifica con i tasti freccia `Home`, `End`, `Backspace`, `Delete` o incolla. In **Installa**, `Enter` invia una query repository/AUR anche quando esistono risultati precedenti, quindi sposta l'attenzione sui risultati; le ricerche ufficiali richiedono una query non vuota e le ricerche AUR richiedono almeno due caratteri. La modifica di una query di installazione annulla la richiesta in sospeso e cancella i vecchi risultati; una nuova richiesta attende `Enter`. In **Rimuovi**, **Aggiorna** e **Orfani**, la modifica filtra immediatamente l'elenco caricato senza un'altra query sul repository. `↓` passa dalla Ricerca ai risultati, `↑` al primo risultato torna alla Ricerca e `Esc` in Ricerca ritorna all'elenco preservando la query. `Enter` su un risultato esegue l'azione di quel pacchetto. La barra delle azioni Aggiorna rimane raggiungibile da `Tab` e `/` ritorna alla ricerca.

Tutte le operazioni in **Pacchetti** aprono un terminale interattivo integrato: pacchetti ufficiali, AUR, pulizia della cache, mirror e modifiche al repository/chiave. Rispondi alle richieste del programma digitando e premendo `Enter`; pacman funziona senza `--noconfirm` in questo terminale. `Esc` nasconde la vista e mantiene il processo in esecuzione. `F8` riapre la stessa sessione da qualsiasi pagina Pacchetti; **Mostra terminale** è disponibile anche nei suoi menu. Il piè di pagina mostra se la transazione è in esecuzione, completata o fallita. Dopo un comando riuscito, `Done!` appare all'interno del terminale dopo il suo output finale, anche quando il terminale era nascosto. `Shift+PageUp` / `Shift+PageDown` scorre la cronologia, `Shift+End` ritorna all'output live e `Ctrl+C` invia un'interruzione al processo figlio. Mentre il terminale è focalizzato, `q` e `?` sono input anziché scorciatoie globali. L'output, i colori ANSI e il cursore vengono preservati e il ridimensionamento aggiorna il terminale figlio. La storia mantiene fino a 10.000 righe fino alla prossima operazione o fino all'uscita dell'applicazione. Solo una transazione viene eseguita alla volta; nascondere il suo terminale non permette all'applicazione di chiudersi mentre è in esecuzione. Nessuna shell libera viene aperta.

L'anteprima dell'aggiornamento legge i database di sincronizzazione esistenti (`pacman -Su --print`) senza modificarli o richiedere i privilegi di root. Dopo la conferma, l'aggiornamento completo esegue `pacman -Syyu` tramite l'helper privilegiato con `pkexec`; la sincronizzazione può rivelare aggiornamenti aggiuntivi. L'installazione di pacchetti, la rimozione, l'aggiornamento del database, la pulizia della cache, il downgrade, le modifiche al repository/alle chiavi e la scrittura sui mirror utilizzano anch'essi quell'helper privilegiato. Gli helper AUR continuano a essere compilati come utente.

**Repository ufficiali → Repository** aggiunge e gestisce repository pacman non ufficiali. **Aggiungi** è una bozza: scegli la **Sorgente** della configurazione (**URL** di un file `.conf`, **Testo** digitato o incollato in un editor a più righe, o **Form** con **Nome**, **Server** e **SigLevel**, scritto nel layout di Arch Wiki) e, sotto **Chiave di firma**, una **Chiave** da un **URL** o **Testo** (o **Nessuno**). Le scelte si spostano con `←`/`→` o `Enter`; i campi di testo aprono un editor dove `Enter` salva una sola riga, `Ctrl+S` salva più righe, incolla funziona e `Esc` annulla. I download accettano solo indirizzi `https://`. **Review and add** scarica e controlla tutto come tuo utente e poi chiede conferma, elencando i repository, i server e SigLevel e l'ID utente e l'impronta digitale della chiave in gruppi di quattro, per confrontarli con quello pubblicato dal repository. Confermando importa la chiave e la firma localmente (`pacman-key --add`, `--lsign-key`), scrive `/etc/pacman.d/<name>.conf` (chiamato come la sua prima sezione) e aggiunge `Include = /etc/pacman.d/<name>.conf` a `/etc/pacman.conf` (è abilitata invece una riga commentata esistente per lo stesso file). Successivamente, il Centro di Controllo offre `pacman -Syyu`, così i nuovi database vengono sincronizzati senza un aggiornamento parziale. Lasciare **Aggiungi** con una bozza richiede conferma prima di scartarla. **Elenco** mostra ogni repository non ufficiale, scritto in `pacman.conf` stesso o incluso da `/etc/pacman.d/`, abilitato o disabilitato; le sezioni ufficiali non sono mai elencate. Una pagina di repository mostra il suo file, SigLevel, server e chiave registrata, un interruttore **Abilitato** e, nella **Zona pericolosa**, **Rimuovi**. Disabilitare commenta la riga `Include`, o aggiunge `#argvus-off ` all'inizio di ogni riga di una sezione inline; rimuovere elimina la riga o la sezione e il file incluso. I repository dello stesso file incluso vengono abilitati, disabilitati e rimossi insieme. Quando il Centro di Controllo importata la chiave, **Rimuovi** offre anche la possibilità di rimuoverla dal keyring di pacman (`pacman-key --delete`). Ogni modifica a `/etc/pacman.conf` mantiene un backup accanto ad essa (`pacman.conf.argvus-backup-…`), e i pacchetti installati non vengono mai rimossi.

### Utenti e gruppi

Quando il provider dell'account è disponibile, apri **Centro di Controllo → Amministrazione di Sistema → Utenti e Gruppi → Utenti** oppure cerca **utenti**. L'interfaccia può elencare account normali e di sistema, creare un utente, modificare i metadati dell'account, gestire gruppi supplementari e primari, cambiare o bloccare una password, sbloccarla, richiedere un cambio password al prossimo login, impostare o rimuovere un avatar e abilitare il login automatico.

La creazione di un utente richiede un nome utente, il nome completo, la shell, i gruppi supplementari e una conferma opzionale della password. Una password vuota crea l'account con la password bloccata; non crea silenziosamente una password utilizzabile. Una pagina utente raggruppa i campi dell'account (nome completo, shell, gruppi primari e supplementari, seguiti da **Salva modifiche**), l'interruttore **Accesso automatico**, le azioni **Password** (cambia, blocca, sblocca, richiedi un cambio al prossimo accesso) e le azioni **Avatar**. Nome utente, UID/GID e directory home sono mostrati solo a scopo informativo. La **Zona pericolosa** alla fine contiene le azioni distruttive separate per eliminare l'account mantenendo la sua directory home o per eliminare anche la directory home. Leggere attentamente il testo di conferma prima di scegliere quest'ultima opzione. Le operazioni sugli account sono privilegiate e possono aprire il prompt di autorizzazione del sistema. Gli avatar possono essere utilizzati dal greeter tramite il volto standard dell'account. integrazione delle immagini.

**Accesso automatico** avvia la sessione dell'account selezionato al prossimo avvio senza mostrare la schermata di login, secondo la stessa convenzione usata da GNOME, LightDM e SDDM per questa funzionalità: nessuna password viene memorizzata da nessuna parte. Abilitandolo per un account, lo si disabilita per qualsiasi account precedentemente configurato (solo un account può accedere automaticamente alla volta). Si applica una sola volta, al prossimo avvio; dopo un logout manuale, la schermata di login normale ricompare per il resto di quella sessione. L'interruttore viene applicato immediatamente, con un prompt di conferma, ed è disponibile solo per gli amministratori.

I gruppi hanno le proprie pagine di elenco e di modifica, inclusa la gestione dei membri e la cancellazione del gruppo. Il Centro di Controllo non sostituisce la politica degli account del sistema: gli account di sistema protetti e le operazioni che richiedono autorizzazione rimangono soggetti alle restrizioni del sistema operativo.

### Impostazioni del Centro di Controllo

**Impostazioni** è l'ultima voce della home. La sua pagina **Lingua** imposta la lingua dell'utente tramite locale.conf, e la sua pagina **Stile** controlla le icone decorative e la trasparenza del Centro di Controllo. Lo Stile non contiene sfocatura: la sfocatura è un'impostazione globale di Hyprland, sotto Hyprland > Sfocatura. La pagina Stile modifica la presentazione dell'applicazione delle impostazioni; non è un interruttore che disabilita i servizi ARGVUS o rimuove i fornitori di impostazioni. Salvare le icone può richiedere l'autorizzazione, mentre la modifica visiva viene applicata all'applicazione corrente. Esc su Stile ritorna alla pagina delle Impostazioni, e Esc lì ritorna alla home.

### Informazioni su ARGVUS

**Informazioni su ARGVUS** si trovano in **Strumenti & Diagnostica → Informazioni su ARGVUS** (cerca `about`, `hardware` o `firmware`, oppure esegui `argvus control-center about`). Ha cinque schede, che si possono cambiare con `Tab`/`Shift+Tab`, `←`/`→` o `h`/`l`: **Sistema & Hardware**, **Informazioni**, **Dona**, **Crediti** e **Copyright**. L'intestazione mostra il percorso completo, ad esempio `Control Center > Tools & Diagnostics > About ARGVUS > System & Hardware`.

**Sistema & Hardware** è in sola lettura e raggruppa i dettagli di questa macchina in tre sezioni, con i valori allineati in una colonna:

| Sezione | Campi |
| --- | --- |
| Avvio | Firmware (fornitore, versione e data), Tipo (UEFI o Legacy (BIOS)), Secure Boot (solo UEFI), Virtualizzazione, Bootloader |
| Sistema | ARGVUS, Nome host, Sistema operativo, Distributore, Kernel, Architettura, Gestore finestre (Hyprland), Compositore (Wayland), Driver GPU, GTK, Tema (con Scuro o Chiaro), Monitor (una riga per ciascun monitor), Audio (PipeWire), Shell (Quickshell), Login (greetd), Init (systemd), Pacchetti, Tempo di attività, Locale |
| Hardware | Batteria (una riga per batteria), CPU (con frequenza massima), Thread, Core, GPU, Memoria (utilizzata / totale), Swap, Archiviazione, Modello (solo quando è diverso dalla scheda madre), Scheda madre |

Ogni valore viene letto dal sistema in esecuzione, senza privilegi, e la pagina funziona allo stesso modo su qualsiasi macchina. Un valore che non può essere letto è mostrato come `N/A`, e una riga che non si applica viene lasciata vuota:

- **Firmware**, **Modello** e **Scheda madre** provengono da DMI (`/sys/class/dmi/id`); i valori segnaposto come `To be filled by O.E.M.` vengono ignorati. **Tipo** e **Secure Boot** provengono da `/sys/firmware/efi`, e **Virtualizzazione** da `systemd-detect-virt`.
- Il **Bootloader** viene letto dalla variabile EFI `LoaderInfo`, che systemd-boot, Limine e altri bootloader che seguono l'Interfaccia Boot Loader pubblicano. GRUB non lo pubblica, quindi viene riconosciuto da `/boot/grub` (o `/efi/grub`) e mostrato con la versione del pacchetto `grub`.
- Le versioni di ARGVUS, Wayland, Mesa, PipeWire, Quickshell, greetd, systemd e GRUB provengono da una query `pacman -Q` e sono mostrate senza l'epoch e il rilascio del pacchetto (`1:25.2.3-1` diventa `25.2.3`). La versione di Hyprland proviene da `hyprctl version`, o dal pacchetto quando nessuna sessione risponde.
- **Driver GPU** indica il driver associato a ciascuna scheda DRM in `/sys/class/drm`. Un driver proprietario mostra la versione del suo modulo (per esempio `nvidia 580.82.07`); un driver in kernel mostra la versione di Mesa (per esempio `amdgpu (Mesa 25.2.3)`).
- **Schermi** elenca i monitor attivi da `hyprctl monitors -j`: connettore, risoluzione, frequenza di aggiornamento e scala (la scala viene omessa se è 1).
- **Pacchetti** conta tutti i pacchetti installati, quelli installati esplicitamente e quelli esterni (AUR o costruiti localmente), da `pacman -Qq`, `-Qqe` e `-Qqm`.
- **Tempo di attività** proviene da `/proc/uptime` ed è il valore al momento dell'apertura della pagina. **Locale** proviene da `LANG` in `/etc/locale.conf`.
- **Batteria** elenca le batterie del computer da `/sys/class/power_supply`, selezionate per tipo, quindi periferiche come mouse wireless non sono conteggiate. Ogni riga mostra la carica, lo stato, la salute (capacità completa rispetto alla capacità di progetto) e i cicli di carica quando il firmware li riporta. Una macchina senza batteria mostra **Non rilevata**.
- **CPU** aggiunge la frequenza massima più alta tra tutti i core (cpufreq), che è la massima nominale sui processori ibridi; viene omessa dove cpufreq non è disponibile, come nella maggior parte delle macchine virtuali. **Thread** e **Core** provengono da `/proc/cpuinfo`.
- **Memoria** e **Swap** provengono da `/proc/meminfo` (compreso zram); un sistema senza swap mostra **Disabilitato**.
- **Archiviazione** descrive il disco sotto `/`: il suo modello, il suo tipo (NVMe, SSD, HDD o eMMC/SD; i dischi virtuali non mostrano alcun tipo), il file system e lo spazio utilizzato e totale. Il disco viene individuato attraverso le partizioni, LUKS e LVM fino al dispositivo fisico.

I valori vengono letti in background la prima volta che viene visualizzata la pagina, quindi né l'avvio del Centro di Controllo né l'apertura della pagina attendono per questi; nel frattempo nella scheda viene mostrato **Caricamento informazioni di sistema...**. Premi `c` su questa scheda per copiare l'intero rapporto come testo semplice negli appunti (con `wl-copy`), ad esempio per incollarlo in un report di bug.

## Ricerca e funzionalità opzionali

La schermata principale dispone di un registro delle rotte ricercabile. I termini di ricerca includono concetti rivolti all'utente come aspetto, temi, sfondi, caratteri, scorciatoie da tastiera, mouse, touchpad, display, rete e sessione. I provider opzionali sono inclusi solo quando la loro funzionalità è compilata e la loro capacità di runtime è rilevata, quindi il menu visibile è intenzionalmente consapevole delle capacità.

Quando la ricerca nella Home è attiva, i tasti stampabili vengono inseriti nella query, inclusi `j` e `k`; usare i tasti freccia `↑` e `↓` per spostarsi tra le rotte corrispondenti. Premere `Enter` per aprire la rotta selezionata, `Backspace` per modificare la query e `Esc` per cancellare la query o uscire dalla modalità di ricerca.

## Flusso di lavoro Aspetto

Aprire **Aspetto** per vedere i controlli visivi integrati. I suoi gruppi sono:

- **Desktop** — **Tema** (scegli una famiglia inclusa e la sua modalità Sticky o Spaced, oppure esporta, importa, applica ed elimina i tuoi temi, vedi [Importazione ed esportazione dei temi](/docs/components/argvus-themes/import-and-export/)), **Modalità Aspetto** (Sticky o Spaced, indipendente dal tema), **Colore Accento** (un accento incluso o un colore RGB a sei cifre valido), **Sfondo** (uno sfondo incluso o un file personalizzato) **Icone** (un tema di icone installato: cerca nella lista con `/`, scegline uno con Space o Invio, poi seleziona **Applica**; nulla cambia prima di Applicare) e **Caratteri** (i target del carattere e le impostazioni di rendering dei caratteri, aperte nella pagina Caratteri).
- **Stile Applicazione** — la trasparenza di **Terminale** e **Launcher**.

La scheda Aspetto contiene le superfici che utilizzano queste impostazioni: **Widget Telemetry** (attiva la superficie di telemetria, seleziona i suoi blocchi disponibili e riordinateli con `Shift+Up`/`Shift+Down` su un blocco selezionato) e **Control Panel** (attiva, disattiva e riordina le schede del pannello con `Shift+Up`/`Shift+Down` su una scheda selezionata, applicato immediatamente attraverso lo stesso primitivo che usa il drag-and-drop del pannello reale).

Queste azioni aggiornano lo stato logico ARGVUS e applicano la configurazione di runtime interessata. Vedi [Aspetto](/docs/user-guide/appearance/), [Temi](/docs/user-guide/appearance/) e [Finestre e layout](/docs/argvus-hyprland/windows-and-layout/).

## Uso della tastiera

L'applicazione delle impostazioni supporta la navigazione tramite tastiera. Ogni pagina è un unico elenco: utilizzare i tasti freccia o `j`/`k` per spostarsi, `Enter` per aprire o eseguire la riga selezionata, `Space` per alternare, `Esc` per tornare indietro, `/` per cercare elenchi e `?` per aiuto; il piè di pagina mostra le chiavi che si applicano alla riga selezionata. Le azioni sono righe dell'elenco, non pulsanti, e `Tab` non si sposta su di esse, tranne che nelle pagine con una barra **Azioni** sotto l'elenco (App predefinite, Fuso orario, Layout tastiera, Impostazioni internazionali di sistema e Chiavi SSH e Crea/Modifica collegamento). Le righe che mostrano solo informazioni vengono saltate. Mentre è attivo un campo di testo, una ricerca o un filtro elenco, `q` e `?` vengono digitati come testo invece di uscire o aprire la guida. Nelle **App predefinite**, **Ripristina impostazioni predefinite** è il pulsante nella barra **Azioni** sotto l'elenco: `Tab` o `←/→` lo focalizzano e `Enter` richiede il ripristino. **Ripristina impostazioni predefinite** nelle altre pagine e **Ripristina tutti i collegamenti** sono righe in **Pericolo zone** alla fine delle loro pagine; `r` richiede lo stesso ripristino e nell'elenco delle scorciatoie `r` ripristina la scorciatoia selezionata dopo una conferma. Su **Lingua e regione → Layout tastiera**, `/` seleziona la casella **Cerca** nella parte superiore della pagina e `Esc` la cancella; `●` segna il layout in uso. Segue la barra delle applicazioni e la scorciatoia da tastiera che scorre i layout e la pagina si aggiorna in circa un secondo senza scrivere nulla nella configurazione. `Enter` imposta il layout predefinito, il primo con cui inizia Hyprland e quello etichettato `Default` nel dettaglio; `Space` aggiunge o rimuove un layout (`[x]`). Questi tasti modificano solo una bozza e il pulsante **Applica** nella barra **Azioni** sotto l'elenco scrive i layout e il valore predefinito in Hyprland in un solo passaggio. Dall'elenco, `Tab` o `←/→` sposta il focus su quella barra mentre **Applica** è abilitato (il piè di pagina mostra quindi `←/→ Actions`); `←/→` scorre i pulsanti e `↑/↓` torna alla lista. Lasciando la pagina con modifiche non applicate viene prima richiesta conferma. Su **Lingua e regione → Fuso orario** il layout è lo stesso: `/` focalizza la casella **Cerca** in alto, `Enter` seleziona una zona in una bozza (contrassegnata con `●`) e il pulsante **Applica** nella barra **Azioni** sotto l'elenco la salva nella `environment.d/20-timezone.conf` dell'utente; `Tab`, `←/→` e `↑/↓` funzionano come nel layout tastiera. Uscendo con una zona non applicata si chiede prima conferma. Nelle **Impostazioni locali del sistema** il layout è lo stesso del Fuso orario: `/` focalizza la casella **Cerca** in alto, `Enter` o `Space` contrassegna o deseleziona una lingua (`[x]`) in una bozza e il pulsante **Applica** nella barra **Azioni** sotto l'elenco scrive la selezione su `/etc/locale.gen` ed esegue `locale-gen`. Poiché ciò rigenera le impostazioni locali del sistema, **Applica** richiede prima conferma; `Tab`, `←/→` e `↑/↓` funzionano come gli altri due. Su **Desktop e input → Scorciatoie da tastiera → Tutto Scorciatoie** la casella di ricerca è anch'essa posizionata in alto, con `/` che la mette a fuoco, ma la pagina non ha una barra **Azioni**: la sua lista mantiene le sezioni delle scorciatoie e la **Zona pericolosa**, e il piè di pagina elenca `r Restore default` solo nella lista stessa, non su **Modifica scorciatoia**.

Le pagine che raccolgono le modifiche prima di salvarle (un utente, un gruppo, la creazione dell'account, la modifica della password e la configurazione del firewall) terminano quel blocco con una riga **Salva** o **Applica** che rimane disattivata fino a quando qualcosa non viene modificato. Se tornando indietro si rischierebbe di scartare modifiche non salvate, il Control Center chiede conferma prima. Ogni conferma mostra **Conferma** e **Annulla** come due righe con il focus su **Annulla**: `Enter` esegue la riga a fuoco, `y` conferma e `n` o `Esc` annullano.

In **Aspetto**, ogni pagina è un'unica lista: il piè di pagina mostra solo i tasti che si applicano alla riga selezionata, e `Space` apre o seleziona anche la riga. Le pagine che raccolgono modifiche prima di applicarle (Barra delle applicazioni, Telemetria dei Widget, Pannello di Controllo e i cursori degli effetti) terminano con una riga **Applica**, che rimane attenuata fino a quando qualcosa non viene cambiato. Selezionare **Applica** mostra la schermata di caricamento solo quando la modifica ricrea una superficie: un cambiamento dell’altezza della barra delle applicazioni, del gruppo di utilità, dell’icona o del formato di data e ora riavvia la barra delle applicazioni; attivare o disattivare la Telemetria dei Widget o modificare i suoi blocchi o l’ordine riavvia il widget; attivare o disattivare il Pannello di Controllo, o modificare le sue schede mentre è attivo, riavvia il pannello. La schermata rimane quindi fino all’applicazione dell’ultima modifica della pagina, così nessuna modifica appare prima, e la superficie viene riavviata una sola volta. Bordi, trasparenza, sfocatura, animazioni, spazi, caratteri, sfondi e le altre impostazioni si applicano sul posto, senza il schermata di caricamento. Nelle righe percentuali, `←`/`→` (o `+`/`-`) cambia il valore a passi di 5 e `Enter` permette di digitarlo; usa `Esc` per tornare indietro. Se tornare indietro comporterebbe la perdita di modifiche non applicate, Control Center chiede prima conferma. Eliminare un tema personalizzato (`d`) o sostituire uno importato richiede conferma; il focus parte su **Annulla**, `y` conferma e `n` o `Esc` annullano.

## Persistenza e reset

Le pagine delle impostazioni applicano le modifiche tramite il provider di appartenenza e salvano lo stato utente supportato. Le modifiche all'aspetto vengono memorizzate nella configurazione canonica, e `argvus-config` è l'unico componente che scrive i file derivati per il consumatore sotto `~/.config/argvus/data/generated/`; quei file generati non sono il luogo per effettuare modifiche permanenti. Le pagine di input e di associazione dei tasti hanno il proprio stato persistente e le azioni di reset. Altri cambiamenti di sistema possono richiedere permessi o un riavvio del servizio.

Non esiste un reset globale per ogni impostazione ARGVUS. Ripristina una modifica dalla pagina che la possiede o utilizza la procedura di recupero documentata della funzione pertinente.

Il supporto al reset è volutamente per dominio:

- **Aspetto** fornisce un reset per il colore di evidenziazione al tema attivo predefinito. Non fornisce un reset globale unico dell'aspetto; ripristina altri valori dell'aspetto dai loro controlli.
- **Caratteri** possono ripristinare tutte le impostazioni dei caratteri, un obiettivo o un'impostazione individuale.
- **Applicazioni predefinite** possono ripristinare tutti i valori predefiniti, una categoria o un selettore individuale.
- **Scorciatoie da tastiera** possono ripristinare un collegamento oppure tutti i collegamenti e poi ricaricare i collegamenti di sessione generati.
- **Regole delle finestre** si trovano in **Desktop & Input → Hyprland → Finestre → Regole** (cerca `window rules` o esegui `argvus control-center window-rules`). Ogni regola ha una riga **Spazio di lavoro** (`Enter` o `←/→` scorrono da 1 a 10) e una riga **Classi della finestra** che apre un campo di testo con espressioni regolari separate da virgola. **Aggiungi regola** crea `rule-N` con spazio di lavoro 1 e senza classi. Ogni **Rimuovi regola** della regola si trova nella **Zona pericolosa** e richiede conferma. Ogni modifica viene scritta in `hyprland.window_rules` in `argvus config` e ricarica immediatamente la configurazione; i modelli di classe non validi vengono rifiutati e mostrati come errore.
- **Progetti** si trova in **Strumenti e diagnostica → Progetti** (cerca `projects` o esegui `argvus control-center projects`). **Aggiungi cartella progetto** e **Aggiungi cartella principale** si trovano in cima alla pagina e aprono un campo di testo con il percorso; il launcher verifica che sia una directory e mostra un errore altrimenti, inclusa la situazione in cui la cartella supererebbe il numero effettivo di progetti pari a 9 (`SUPER + CTRL + 1..9` non può gestire di più). Ogni cartella configurata, contrassegnata come principale (le sue sottocartelle sono progetti) o singolo progetto, ottiene il proprio blocco sotto il modulo con una riga **Rimuovi** che richiede conferma. La pagina chiama `argvus-projects`, quindi le modifiche vengono scritte dal launcher e si applicano la volta successiva che `SUPER + O` o `SUPER + CTRL + 1..9` viene eseguito. Se `argvus-projects` non è installato, la pagina lo segnala.
- **Altre pagine di sistema** espongono azioni di reset, applica, elimina o ripristina solo quando il provider sottostante le supporta.

Rimuovere una preferenza utente può far sì che un provider torni al suo stato predefinito/confezionato, ma eliminare i file manualmente non è una procedura generale di recupero. Preferisci l'azione di reset della pagina o il comando documentato specifico per la funzione.

## Punti di accesso CLI

L'aiutante è `argvus-control-center`, eseguito tramite `argvus control-center`. Può aprire direttamente aree specifiche, ad esempio:

```sh
argvus control-center appearance themes
argvus control-center appearance wallpapers
argvus control-center input
argvus control-center keybindings
```
Esegui `argvus control-center --help` per le rotte disponibili nella versione installata. La rotta da linea di comando è un collegamento diretto alla stessa applicazione delle impostazioni, non un sistema di configurazione separato.

I percorsi diretti utili includono `apps`, `window-rules`, `projects`, `snippets`, `ssh-keys`, `gpg-keys`, `firewall`, `fonts`, `locale`, `input`, `keybindings`, `language`, `config`, `system`, `hardware`, `services`, `dev-services`, `network` (con le pagine `status`, `connections`, `activate`, `add`, `interfaces`, `wifi`, `ethernet`, `vpn`, `dns` e `proxy`), `audio`, `bluetooth`, `boot`, `packages` (con le pagine `official`, `search`, `installed`, `updates`, `orphans`, `aur`, `aur-remove`, `aur-updates`, `repositories`, `repositories-add`, `repositories-list`, `cache`, `history`, `downgrade` e `mirrors`), `storage`, `system-doctor` (`diagnostics` e `diagnostic` rimangono alias), `about` (con le schede `system`, `about`, `donate`, `credits` e `copyright`), `power`, `session`, `displays` e `appearance`. L'aspetto accetta anche percorsi mirati come `themes`, `wallpapers`, `icons`, `accents`, `effects`, `spaces`, `taskbar` e `widget-telemetry`; il display accetta `resolution`, `refresh`, `scale`, `position`, `orientation`, `primary`, `vrr` e `hdr`. Utilizzare l'output `--help` installato durante la creazione di script perché la disponibilità dipende ancora dalla build installata.

## Correlati

- [Prima configurazione](/docs/getting-started/)
- Pannello di controllo
- [Aspetto](/docs/user-guide/appearance/)
- [Scorciatoie da tastiera](/docs/argvus-hyprland/keyboard-shortcuts/)
- [Mouse e touchpad](/docs/argvus-hyprland/input/)

### Dottore di sistema

Aprire **Strumenti e diagnostica → Dottore di sistema**, cercarlo o eseguire `argvus control-center system-doctor`. I precedenti percorsi `diagnostics` e selettori di categoria `diagnostic` rimangono disponibili. Premere `r` per aggiornare, Invio per ispezionare un controllo, `o` in un dettaglio per aprire la sua pagina correlata, ed Esc per tornare indietro. La raccolta viene eseguita in background ed è di sola lettura.

I controlli utilizzano i conteggi delle unità fallite di systemd, strumenti grafici e lo snapshot dello Storage per l'uso del filesystem e la salute SMART. Un problema richiede prove; strumenti non disponibili o dati SMART inaccessibili sono mostrati come non disponibili, e una sonda fallita o scaduta è considerata un errore. Le categorie senza prove raccolte non implicano un controllo riuscito. Gli aggiornamenti dei pacchetti utilizzano il database pacman locale e non sincronizzano i repository; un file di blocco può essere obsoleto e non dimostra che un'operazione sia in corso. SMART può richiedere permessi e non viene richiesto accesso elevato. Il riepilogo registra l'ultima raccolta completata come tempo Unix.

Storage e System Doctor condividono uno snapshot in memoria per un massimo di 10 secondi. Premendo `r` si raccolgono esplicitamente dati nuovi; le richieste concorrenti riutilizzano una raccolta appena completata. L'output dei sottoprocessi Diagnostic e Storage è limitato a 1 MiB per flusso, e l'output di dimensioni eccessive è trattato come una raccolta fallita.

## Luce Notturna

Apri **Desktop & Input > Schermi > Luce Notturna** oppure esegui `argvus control-center displays night-light`. Configura la temperatura, l'attivazione manuale e gli orari locali fissi tramite `argvus config display night-light`. Vedi [Luce Notturna](/docs/user-guide/night-light/) per comandi, stato e requisiti di esecuzione.

## Istantanea

Apri **Centro di Controllo > Amministrazione di Sistema > Snapshot**, cerca `Snapper`, oppure esegui `argvus control-center snapshot`. La funzionalità `snapshot` Cargo è inclusa in `full` (il default) ed è indipendente da `storage`. La pagina è contrassegnata come **Sperimentale** perché la funzionalità è ancora in sviluppo e potrebbe essere incompleta.

Questa pagina gestisce solo una configurazione Snapper esistente il cui `SUBVOLUME` è `/` e il cui `FSTYPE` è `btrfs`. Controlla il mount contenente `/` con `findmnt`; una configurazione chiamata `root` che punta altrove non è accettata. Configurazioni corrispondenti multiple richiedono una selezione. Il pacchetto Arch `snapper` è opzionale e deve essere installato manualmente. Snapper deve supportare `--jsonout` e colonne attive/predefinite esplicite. Nessuna configurazione viene creata automaticamente.

Su una root non Btrfs la pagina mostra **Questo filesystem non è Btrfs** e non offre operazioni di snapshot. Fallimento del rilevamento, Snapper mancante, configurazione della root mancante, negazione dei permessi ed errori del backend sono stati separati. Se le query dell'utente normale vengono negate, premere **a — Autorizza** per richiedere l'accesso a Polkit. Le azioni amministrative vengono eseguite nell'helper `system-settings` limitato, non in un TUI con privilegi elevati; l'autenticazione potrebbe essere richiesta nuovamente nelle query successive. È necessario un agente grafico Polkit esistente per l'autenticazione.

Tasti: **↑/↓** seleziona, **Enter** dettagli/configurazione, **c** crea, **d** elimina, **p** politiche di conservazione, **r** aggiorna, **Esc** indietro, **?** aiuto. Nella visualizzazione delle politiche di conservazione, **↑/↓** seleziona una politica, **e** modifica, **c** cicla tra gli algoritmi di pulizia e **Enter** apre la conferma. La creazione apre una descrizione opzionale (fino a 256 caratteri stampabili); una descrizione vuota utilizza “Snapshot manuale ARGVUS”. Sia la creazione che l'eliminazione richiedono conferma, inizialmente focalizzata su Annulla. L'eliminazione si applica a un singolo snapshot selezionato e rifiuta lo snapshot 0, lo snapshot attualmente attivo e lo snapshot predefinito. L'assistente ricontrolla queste condizioni prima di invocare Snapper. Le liste si aggiornano dopo le operazioni; un'operazione interrotta non viene mai ripetuta automaticamente. Ispeziona prima la lista aggiornata.

Per confrontare gli snapshot, seleziona la base e premi **b**, quindi seleziona una destinazione diversa e premi **v**. La pagina mostra gli ID ordinati, i conteggi di quelli creati,
percorsi rimossi e modificati e sei indicatori di stato di Snapper. Seleziona un percorso e premi **Invio** per richiedere la differenza del contenuto; **Esc** torna ai percorsi modificati e poi all'elenco degli snapshot. Gli indicatori codificano creazione/rimozione o contenuto/tipo in posizione 1, permessi in 2, proprietario in 3, gruppo in 4, attributi estesi in 5 e ACL in 6. Un punto significa nessuna modifica in quella posizione. Lo stato elenca i percorsi; diff mostra il contenuto testuale. L'output della diff binaria/non UTF-8 non viene visualizzato. L'output diff superiore a 256 KiB viene segnalato come troncato e non mostrato. Le linee di stato ambigue o malformate vengono rifiutate. Il formato delle righe di Snapper non può rappresentare percorsi con interruzioni di riga in modo univoco; tali nomi non sono supportati. Gli errori di permesso possono essere ritentati tramite **a — Autorizza**; uno snapshot rimosso o una configurazione root modificata richiede l'aggiornamento dell'elenco. I test del backend utilizzano un falso `ProcessRunner` e non richiedono Snapper, Btrfs o i permessi di root.

Dalla comparazione, selezionare un percorso e premere **r** per esaminare il ripristino del file dallo snapshot di destinazione. L'assistente verifica che entrambi gli snapshot appartengano ancora alla configurazione principale, che il percorso sia ancora presente nella comparazione e che la fonte sia un file regolare nella visualizzazione standard `/.snapshots/<id>/snapshot` di Snapper. La differenza limitata di Snapper viene mostrata quando è testuale; le differenze binarie o sovradimensionate vengono identificate senza caricare l'intero contenuto nell'interfaccia utente. **Enter** apre una conferma separata che nome la fonte e la destinazione; **Esc** annulla senza modificare il file. La conferma crea prima uno snapshot di sicurezza `single`, quindi utilizza Snapper `undochange` solo per quel percorso e verifica che corrisponda allo snapshot selezionato. L'ID dello snapshot di sicurezza viene riportato e lo snapshot viene mantenuto. Il ripristino condivide il blocco utilizzato dagli aggiornamenti dei pacchetti del pacchetto ARGVUS, quindi non può sovrapporsi a una transazione ufficiale di pacman del Control Center.

La prima versione accetta solo file regolari di proprietà root all'interno di directory genitore di proprietà root non scrivibili da gruppo o altri. Rifiuta directory, collegamenti simbolici, link fisici, file speciali, punti di montaggio/subvolume, la directory interna di Snapper e posizioni scrivibili dall'utente. I file mancanti dallo snapshot selezionato non vengono mai eliminati. La root `undochange` di Snapper non garantisce la coerenza di applicazioni o database e non è atomica; lo snapshot di sicurezza è un punto di recupero, non una garanzia. Snapper conserva solo i metadati supportati dalla sua build. Subvolume separati e file al di fuori della root configurata rimangono al di fuori dell'operazione.

Gli snapshot manuali sono `single`, di sola lettura per impostazione predefinita di Snapper, senza alcun algoritmo di pulizia assegnato da questa pagina. I timer esistenti di Snapper e le operazioni esterne continuano ad applicarsi. Gli snapshot non sono immagini a sistema completo: subvolume separati,
incluso un `/home` separato, non sono inclusi. Questa versione non fornisce **nessun ripristino completo del sistema, installazione o creazione di una configurazione Snapper**. Premere **p** su una configurazione root supportata per ispezionare le impostazioni di conservazione conosciute. I valori mancanti sono etichettati come fallback ARGVUS, non come valori letti dal modello di creazione Snapper installato; le chiavi sconosciute sono solo diagnostiche. Premere **e** per modificare un valore supportato. La conferma mostra i vecchi e i nuovi valori; dopo il salvataggio, la pagina legge di nuovo la configurazione e la verifica. Premere **c** per ciclare tra gli algoritmi di pulizia e **Invio** per rivedere e confermare uno. La pulizia utilizza l'algoritmo ufficiale di Snapper; non esiste un'anteprima esatta e il conteggio rimosso riportato viene misurato confrontando le liste dopo l'esecuzione. Arch Snapper 0.13.2 fornisce `snapper-timeline.timer` e `snapper-cleanup.timer`; la creazione della timeline richiede
il timer della timeline e le politiche di pulizia configurate non vengono eseguiti senza la pianificazione della pulizia. La pagina legge solo lo stato dell'unità e non abilita i timer. I limiti di spazio richiedono qgroup Btrfs configurati e non sono visibili qui.

### Operazioni sui pacchetti e snapshot

L'installazione, l'aggiornamento e la rimozione dei pacchetti non richiedono Snapper, Btrfs o una configurazione di snapshot. **Aggiorna tutto** esegue `pacman -Syyu` tramite l'helper privilegiato con `pkexec`; gli aggiornamenti ufficiali individuali eseguono `pacman -S <package>`. ARGVUS non crea snapshot automatici per queste operazioni. La pagina Snapshot rimane indipendente e mantiene i propri requisiti Btrfs/Snapper. Gli hook configurati separatamente in pacman rimangono controllati dal sistema.

### Validazione manuale

- Su una root ext4/xfs, apri la pagina in inglese e in portoghese. In portoghese, verifica il messaggio esatto **Este sistema de arquivos não é Btrfs** e verifica che c/d non aprano un’operazione.
- In una VM Btrfs usa e getta, testa Snapper assente, Snapper senza una configurazione root e una configurazione chiamata `root` che punta a un altro sottovolume.
- Con una configurazione root esistente, elenca gli snapshot, apri i dettagli, crea uno snapshot con spazi/caratteri speciali, annulla una cancellazione, quindi elimina solo lo snapshot appena creato dopo aver confermato il suo numero e la descrizione.
- Controlla autenticazioni negate/annullate, snapshot attivi/predefiniti protetti, aggiorna, navigazione da tastiera, ridimensionamento e entrambe le lingue. Non testare la cancellazione sugli snapshot necessari per il recupero.

Solo il pacchetto `argvus` cambia. Ricostruisci da `de/argvus` con `make build` per produrre il pacchetto `.pkg.tar.zst`; la sua installazione è un’azione dell’operatore separata.

La lingua sotto Lingua e Regione salva la localizzazione nel profilo utente (`~/.config/locale.conf`) senza modificare `/etc/locale.conf`. La sessione utilizza questa preferenza dopo il nuovo accesso. Durante la schermata di Benvenuto, F6 Avanti o il pulsante Avanti nel footer registra la revisione e procede; Shift+Tab mette a fuoco il pulsante e Invio lo attiva, anche sulle pagine informative.

Bluetooth appare nella schermata di Benvenuto solo quando il Centro di Controllo rileva un adattatore. Il progresso esistente per un passaggio Bluetooth nascosto viene preservato.

La lingua e la tastiera della sessione sono preferenze personali e non richiedono privilegi di amministratore. I nuovi profili hanno come impostazione predefinita l'inglese e la tastiera statunitense; le preferenze personali esistenti vengono preservate. Data/ora, NTP, generazione della localizzazione di sistema e la mappa dei tasti della console (TTY) mantengono il loro comportamento amministrativo a livello di sistema.

Per una tastiera sessione brasiliana, selezionare `br`, applicare il layout, quindi selezionare `abnt2` sotto Variante. Questo scrive la configurazione personale di Hyprland senza autorizzazione amministrativa; la mappa dei tasti della console è separata. ABNT2 utilizza tasti morti: `~` seguito da `a` produce `ã`, e `~` seguito da Spazio produce `~`. Applicare la variante attiva il layout predefinito configurato; i fallimenti di attivazione vengono segnalati.

Il Fuso Orario salva TZ sotto `$XDG_CONFIG_HOME/environment.d/20-timezone.conf` senza autorizzazione. Effettuare il logout e il login per consentire alle applicazioni di ereditarlo. Data e Ora continua a mostrare e modificare l'orologio di sistema nel fuso orario del sistema, con autorizzazione amministrativa; NTP rimane invariato.

I profili display e le assegnazioni degli spazi di lavoro vengono applicati tramite la configurazione Lua generata e un reload di Hyprland. L'alimentazione del monitor utilizza il dispatcher Lua DPMS supportato da Hyprland 0.56.2.

La potenza di rendering delle ombre è limitata a 1–4 (predefinito 4) per la compatibilità con Hyprland 0.57. I valori memorizzati più vecchi superiori a 4 vengono limitati durante il caricamento della configurazione.

## Vibrazione della finestra

Apri **Desktop & Input → Hyprland → Animazioni → Finestre** e utilizza i controlli **Vibrazione**. Cerca `wobble`, oppure apri direttamente la pagina:

```sh
argvus control-center hyprland animations windows
```
Il Wobble è disabilitato di default. I suoi valori predefiniti ARGVUS sono stiffness `100`, damping `10`, mass `1`, intensity `1` e mesh resolution `8`. Stiffness e mass accettano `0.0001–1000`, damping `0–1000`, intensity `0–2` e mesh un intero `2–32`. Una risoluzione della mesh più alta aumenta il costo di rendering. Usa i tasti freccia per regolare i valori o Invio per digitarli. **Ripristina valori predefiniti di Wobble** modifica il draft; **Applica** salva tutte le modifiche di animazione in sospeso e ricarica Hyprland una volta, senza riavviare la sessione. Tornare indietro in Animations mantiene il draft condiviso.

L'interruttore globale Animations sopprime il Wobble mantenendo le sue impostazioni individuali. I valori canonici risiedono sotto `effects.wobble_*` nello stato ARGVUS; `hypr/state.lua` viene generato e non deve essere modificato manualmente.

Ciò richiede Hyprland **0.57.0 o successivo**, applicato da `argvus-hyprland`
dipendenza del pacchetto. La versione 0.56 non è supportata per questa configurazione. Il
l'intensità massima segue la fonte [ rilasciata 0.57.0](https://github.com/hyprwm/Hyprland/blob/v0.57.0/src/config/values/ConfigValues.cpp),
che lo limita a `2`, anche se la documentazione più recente potrebbe elencare `3`.

Il benvenuto si apre con il logo ARGVUS e BENVENUTO. Premere F6 Avanti per visualizzare l'elenco di controllo senza completare un passaggio.

Le nuove installazioni hanno come impostazione predefinita il fuso orario personale `America/New_York` e solo il layout della tastiera `us` è abilitato. Le impostazioni personali esistenti vengono conservate; il fuso orario del sistema rimane invariato.

Le opzioni del modulo della barra delle applicazioni per Batteria e Bluetooth appaiono solo quando l'hardware corrispondente viene rilevato. Nascondere 'Mantieni attivo' nasconde il suo indicatore senza modificare l'inibitore di sospensione. Le utilità supportano un cassetto a comparsa ridotto. La presentazione delle icone dello spazio di lavoro mostra le icone delle finestre aperte; il vassoio di sistema controlla le icone delle applicazioni in background.

La pagina delle scorciatoie da tastiera include **selettore agente AI (pacchetto argvus-ai opzionale)** con SUPER + A predefinito. Usa l'ID catalogo app.ai, quindi la rimappatura e la disattivazione seguono la stessa configurazione canonica keyboard_shortcuts delle altre voci. Il suo bind eseguibile Lua è fornito da argvus-ai; la sola voce del catalogo non lo attiva senza quel pacchetto opzionale.


Centro di Controllo > Aspetto > Barra delle applicazioni > Icone > Moduli opzionali ora include **Agenti AI**. È abilitato di default e viene memorizzato in /taskbar/icons/ai_agents_enabled. Attivalo o disattivalo nel progetto della barra delle applicazioni e applica per mantenere e rigenerare il gruppo di utilità; la disattivazione non modifica il collegamento SUPER + A né le voci del launcher delle applicazioni.

Gli interruttori di avvio automatico modificano la voce `.desktop` dell’utente o creano una copia utente di una voce di sistema. Nomi, comandi e flag di abilitazione provengono da `[Desktop Entry]`; i gruppi di azioni desktop aggiuntive rimangono invariati.

Le conferme dei servizi mantengono l’ambito utente/sistema selezionato insieme all’unità e all’azione. Un’unità con lo stesso nome in un altro ambito non può reindirizzare un’azione confermata; le pagine di dettaglio mantengono anche l’ambito originale.

L'archiviazione include punti di mount nidificati come `/home`, `/boot` e `/boot/efi` dall'inventario del filesystem. La diagnostica utilizza lo stesso inventario per i controlli sull'utilizzo del filesystem root e home esistenti.

I nuovi snapshot di sicurezza per il ripristino utilizzano l'algoritmo di pulizia `number` di Snapper. La loro rimozione segue la politica di conservazione configurata quando viene eseguita la pulizia. Gli snapshot esistenti senza un algoritmo di pulizia non vengono modificati e la creazione manuale mantiene la politica esistente.

Il confronto degli snapshot segnala separatamente i record dei percorsi non supportati. I file validi rimangono disponibili per la revisione; i percorsi con codifica non valida, caratteri di controllo o componenti non supportati non offrono azioni di differenza o ripristino.

Le impostazioni di alimentazione scrivono il logind drop-in ARGVUS sotto `[Login]`, creando la sua directory quando necessario. I fallimenti nel ricaricamento vengono segnalati. Il comportamento del coperchio della batteria viene mappato su `HandleLidSwitch`; il contesto legacy `default` è un alias, mentre il comportamento AC e docked utilizza le proprie chiavi logind separate.

L'applicazione Mirrorlist accetta solo commenti, righe vuote e voci `Server` HTTP/HTTPS. I payload sono limitati a 64 KiB e controllati prima dell'elevazione e di nuovo dall'helper privilegiato; le direttive di pacman come `SigLevel` e `XferCommand` vengono rifiutate.

La pulizia della cache per i pacchetti disinstallati non mantiene versioni nella cache. Le operazioni sui repository rifiutano include condivisi da più sezioni del repository. I nomi host con spazi bianchi circostanti e una selezione di locale generata vuota vengono rifiutati. La scrittura dei file di impostazioni rifiuta destinazioni collegate tramite symlink e file temporanei esistenti.

Le ricerche dei pacchetti e il caricamento delle pagine scartano i risultati di una pagina o query precedente. La pagina corrente viene quindi caricata di nuovo, quindi un risultato vecchio non può contrassegnare una vista diversa come aggiornata di recente.

L'output della transazione viene sanificato e limitato. Il display mantiene fino a 10.000 righe o 4 MiB e segnala l'output omesso. Gli stream dei processi catturati rispettano il loro limite configurato, impostato di default a 8 MiB per l'esecuzione live; l'output in eccesso viene drenato e segnalato dopo la fine del processo.

L'inventario di avvio legge il kernel Arch `pkgbase` per individuare le immagini del kernel, i file initramfs e i preset mkinitcpio mentre visualizza separatamente la release del kernel. Gli array letterali di mkinitcpio vengono letti senza includere le parentesi o eseguire la configurazione.

Le modifiche di alimentazione per il coperchio, il pulsante di accensione e la modalità di mantenimento attivo vengono salvate nella configurazione canonica solo dopo che l'azione di runtime ha avuto successo. Annullare l'autorizzazione lascia intatto il valore precedentemente salvato. Quando nessun logind drop-in di ARGVUS fornisce un valore, viene letta come fallback la configurazione principale di logind.

Dopo il fallimento di una modifica di retention, la pagina rilegge le impostazioni correnti prima di mostrarne nuovamente i valori. La selezione della configurazione inizia dalla prima voce.

Le operazioni del pacchetto ARGVUS condividono un blocco esclusivo con il ripristino selettivo. Installazione, rimozione, reinstallazione, downgrade, aggiornamento dei metadati, pulizia della cache e modifiche al repository o al mirror vengono rifiutate mentre quel blocco è attivo. Questo coordinamento copre le operazioni avviate tramite l'helper ARGVUS; strumenti esterni come pacman e AUR non utilizzano questo blocco.

Mentre un'operazione in background sta applicando modifiche, `q` e `Ctrl+C` mantengono aperto il Centro di Controllo e mostrano un avviso di attesa. Questo vale anche dopo aver navigato lontano dalla pagina che ha avviato l'azione. Attendere il suo risultato, quindi richiedere nuovamente l'uscita. I normali aggiornamenti in sola lettura non bloccano l'uscita.

I campi per la password Wi-Fi, DNS manuale e ricerca della rete accettano incolla tra parentesi. I controlli incollati e le interruzioni di riga vengono omessi. Le scorciatoie da tastiera non supportate solo con Ctrl o solo con Alt non inseriscono lettere. Gli editor accettano eventi stampabili AltGr codificati come Ctrl+Alt; Ctrl+S salva solo senza Alt. L'editor di testo condiviso mantiene la navigazione verticale nelle colonne visive e allinea lo scorrimento orizzontale per i caratteri larghi.

Le modifiche al volume audio e alla modalità silenziosa per microfoni e uscite non predefinite non sovrascrivono più le preferenze di uscita predefinita salvate. Il backend mantiene tali preferenze solo quando riesce a identificare il nodo selezionato come uscita predefinita corrente. La pagina mostra volumi amplificati oltre il 100% come riportato da PipeWire; le modifiche esplicite del volume rimangono limitate a 0–100%.

Hardware Overview identifica i modelli GPU separatamente dal loro indirizzo PCI e dalla classe del dispositivo, preservando i nomi commerciali tra parentesi. Il suo riepilogo mostra il renderer OpenGL interrogato per la sessione e tutti i nomi dei dispositivi restituiti dalla probe Vulkan, inclusi i dispositivi software. Questi risultati globali non sono presentati come capacità di ciascuna GPU fisica.

Quando un amministratore blocca o rimuove l'account configurato per il login automatico, il fornitore dell'account disabilita prima la voce di login automatico di greetd. Se questa operazione di pulizia fallisce, l'operazione sull'account viene rifiutata. Se in seguito il blocco o la rimozione fallisce, il login automatico rimane disabilitato. Il login automatico per un altro account viene preservato; lo sblocco di un account non abilita il login automatico. Le sessioni esistenti non vengono chiuse e greetd non viene riavviato.

## Creare e gestire le scorciatoie da tastiera

**Desktop & Input → Scorciatoie da Tastiera** apre un menu con **Crea Scorciatoie** e **Tutte le Scorciatoie**. Gli alias CLI esistenti (`keybindings`, `shortcuts`, `keyboard`) aprono questo menu; aggiungere `create` o `list` per aprire direttamente un figlio. La ricerca globale trova anche entrambi i figli.

**Crea Scorciatoie** richiede una descrizione, una combinazione di tasti e un tipo di azione. Seleziona **Scorciatoia corrente** e premi `Enter`, quindi premi la combinazione di tasti desiderata. **Cambia scorciatoia** avvia la stessa cattura; nessuna apre un editor di testo. La cattura modifica solo la bozza. Se una combinazione è già in uso, il conflitto viene mostrato immediatamente, anche prima che descrizione e comando siano compilati. Confermare **Sostituisci** seleziona la combinazione e registra la tua scelta; non crea la scorciatoia né disabilita un'altra fino a **Crea**/**Salva**. Annullare il conflitto mantiene la combinazione precedente. Un **Comando** viene eseguito quando la scorciatoia viene premuta; la creazione non lo esegue mai. Un **dispatcher Hyprland** utilizza un nome di dispatcher Lua supportato, come `window.float`, con argomenti opzionali come oggetto JSON, per esempio `{"action":"toggle"}`. Sottomappe, assegnazioni del mouse e flag avanzati rimangono al di fuori di questo modulo.

Usa `Tab` per passare tra il modulo e **Azioni**, `←/→` per spostarti tra **Crea** e **Annulla**, e `Enter` per eseguire l'azione selezionata. Annulla o `Esc` scartano la bozza. Dopo la creazione, **Tutti i collegamenti** si apre con il nuovo elemento selezionato e visibile. Le voci personalizzate appaiono in **Collegamenti personalizzati**; la ricerca corrisponde a descrizione, tasti e azione. `Enter` modifica descrizione, azione e tasti, `Space` abilita/disabilita, e selezionando la riga **Elimina** e premendo `Enter` si apre una conferma focalizzata su Annulla. Confermare elimina il record e ritorna alla lista aggiornata; annullare lo mantiene.

I collegamenti abilitati nello stesso contesto non possono riutilizzare una combinazione silenziosamente: i conflitti configurabili richiedono conferma per essere sostituiti, e i collegamenti protetti non possono essere sostituiti. La sostituzione salva il nuovo collegamento e disabilita i conflitti in un'unica transazione di configurazione. **Ripristina predefinito** si applica solo alle voci ufficiali; **Ripristina tutti i collegamenti** preserva i record personalizzati. Se il ripristino dovesse entrare in conflitto con un collegamento personalizzato attivo, modifica o disabilita prima quel collegamento personalizzato.

Salva e applica vengono eseguiti in background. Se il salvataggio fallisce, la bozza resta disponibile. Se l'applicazione alla sessione fallisce dopo il salvataggio, la pagina indica che è stato salvato e mantiene lo stesso ID per un nuovo tentativo. I record personalizzati si trovano in `/hyprland/custom_shortcuts` nella configurazione canonica, proiettati in `data/generated/hypr/custom-shortcuts.lua`; non modificare i file generati.

Gli errori di scorciatoia rimangono all'interno del TUI: il popup si ridimensiona al messaggio a capo e mantiene **Enter OK** visibile. La diagnostica tecnica delle scorciatoie va nel journal locale (`journalctl -t argvus-control-center`) invece che su stdout/stderr. Una cancellazione che viene salvata ma non può essere applicata segnala tale distinzione; ricarica la configurazione della sessione per rimuovere il collegamento precedentemente attivo.

**Stato Aggiornamento** in **Barra delle attività → Icone → Moduli opzionali** mostra il conteggio degli aggiornamenti disponibili ed è abilitato di default. **Verifica AUR** è disabilitato di default e richiede `paru` o `yay`; applica la bozza della barra delle attività per salvare una delle due impostazioni. Le verifiche avvengono all'avvio e ogni ora; cliccando sull'indicatore si apre la pagina ufficiale degli aggiornamenti dei Pacchetti.

## Modalità finestra globale

Apri **Centro di Controllo → Desktop & Input → Hyprland → Finestre → Modalità** e seleziona **Finestre a Piastrelle (Predefinito)** o **Finestre Galleggianti**. Lo stesso selettore è disponibile dall'icona della modalità finestra nel gruppo di utilità della barra delle applicazioni.

La preferenza si applica a ogni monitor e spazio di lavoro ed è ricordata dopo il logout. Galleggiante converte le finestre delle applicazioni esistenti e le nuove finestre aperte. Piastrellato ripristina solo le finestre convertite da questa politica; finestre di dialogo e altre finestre già galleggianti rimangono galleggianti. Una finestra fissata esplicitamente mentre Galleggiante è attivo rimane anch'essa galleggiante per preservare il suo fissaggio. Il layout a piastrelle viene ricalcolato; il suo albero di divisione precedente esatto non viene ripristinato.

`SUPER + SHIFT + Space` mantiene il suo comportamento normale per finestra in Tile. In Floating non cambia una finestra tornando a tiling. Pannelli, la barra delle applicazioni e le notifiche sono superfici layer-shell e non sono influenzate. La modalità a schermo intero e la massimizzazione continuano a usare il comportamento normale di Hyprland.

La preferenza canonica è `/hyprland/window_mode` in `$XDG_CONFIG_HOME/argvus/config/hyprland.json`. La sua proiezione generata è `data/generated/hypr/window-mode.lua`; non modificarla. Il registro di ripristino per finestra si trova sotto `$XDG_RUNTIME_DIR/argvus`, limitato all'istanza del compositor, e sopravvive al ricaricamento della configurazione. Non viene riutilizzato in una nuova sessione.

Se l'applicazione è in attesa o fallisce, l'interfaccia distingue questo dalla preferenza salvata. Seleziona di nuovo l'opzione per ritentare. Senza un compositor, la preferenza viene salvata per il prossimo accesso.

## Pulsanti della finestra e barra delle applicazioni

La modalità flottante aggiunge una barra del titolo ARGVUS con **Riduci a icona**, **Ingrandisci / Ripristina** e **Chiudi** alle finestre delle applicazioni normali. Facendo doppio clic sulla barra del titolo si alterna tra l'ingrandimento e la dimensione normale. La decorazione aggiuntiva è nascosta in modalità Tile e a schermo intero. Le applicazioni che disegnano i propri controlli possono visualizzare entrambe le barre; i controlli uniformi ARGVUS sono intenzionali.

Una seconda Waybar appare su ogni monitor, sul lato opposto della barra delle applicazioni principale. Elenca solo le finestre dell'area di lavoro corrente di quel monitor, comprese le finestre ridotte a icona. Le schede di gruppo nascoste non creano voci aggiuntive. Le voci ridotte a icona mantengono l'associazione originale a monitor/area di lavoro. Clicca su una finestra inattiva per attivarla, su una finestra attiva per ridurla a icona, o su una finestra ridotta a icona per ripristinarla; il suo menu contestuale offre ripristina, riduci a icona, ingrandisci/ripristina e chiudi. Le icone delle applicazioni sconosciute ricadono su un simbolo della finestra.

`SUPER + Alt + Up` posiziona la barra principale in alto e la barra della finestra in basso. `SUPER + Alt + Down` fa il contrario. Ripetere una direzione lascia le posizioni invariate. La piastrella ha solo la barra principale. Tornare alla Piastrella ripristina tutte le finestre minimizzate prima di rimuovere la barra della finestra e poi preserva la politica originale a piastrella/galleggiante.

La minimizzazione utilizza lo spazio di lavoro riservato `special:argvus-minimized`. Non utilizzare manualmente quello spazio di lavoro. Il processo `argvus windows service` registra l'identità, lo spazio di lavoro/monitor originale, la geometria, lo stato a schermo intero/massimizzazione e lo stato di pinning in un registro runtime specifico per istanza del compositore sotto `$XDG_RUNTIME_DIR/argvus`. I record sopravvivono a un riavvio del servizio o a un ricaricamento della configurazione, ma non attraversano le sessioni del compositore. La chiusura di una finestra rimuove il suo record. Se un monitor scompare, il ripristino utilizza un'uscita disponibile. Una barra di recupero mancante o guasta impedisce nuove minimizzazioni; un ripristino fallito mantiene il record e la barra di recupero.

La sessione possiede `argvus-windows.service` e `argvus-window-bar.service`. Quest'ultimo mantiene il suo figlio Waybar dormiente in Tile. Il plugin hyprbars è incluso da `argvus-hyprland`, costruito per l'ABI Hyprland 0.57.0 su x86_64; non è compilato nella directory home dell'utente. Ricompila il plugin con un aggiornamento dell'ABI del compositore.
Comandi: `argvus windows list --json`, `watch --json`, `position top|bottom`, `restore-all` e `minimize|restore|activate|maximize|close ADDRESS`. Gli indirizzi delle finestre provengono dalla lista. `position top --persist` memorizza/proietta la preferenza senza riavviare la barra principale. I dettagli della configurazione e degli errori di runtime vengono scritti nel giornale; anche i fallimenti delle azioni visibili mostrano una notifica tradotta.

## Sottomenu Finestre e chiusura dell'applicazione

**Desktop e Input → Hyprland → Finestre** raggruppa **Regole**, **Spazi** e **Modalità**. Regole modifica il posizionamento per applicazione; Spazi modifica i margini interni ed esterni; Modalità contiene il selettore immediato Tile/Floating, separato dal modulo delle regole. Ricerca apre le stesse pagine, e Indietro ritorna a Finestre prima di Hyprland.

Le rotte dirette sono `argvus control-center hyprland windows`, seguite facoltativamente da `rules`, `spaces` o `mode`. Le rotte esistenti `window-rules` e `appearance window-spaces` rimangono alias e mostrano le nuove briciole di pane.

`SUPER + Q` chiude immediatamente la finestra dedicata del Centro di Controllo, senza la conferma della finestra OS di Kitty. Le bozze non applicate vengono eliminate quando la finestra viene chiusa in questo modo. La sovrascrittura è limitata alla classe di finestre `argvus-control-center`; i terminali normali e altre interfacce TUI mantengono la loro policy di conferma alla chiusura.

`SUPER + Alt + Space` attiva o disattiva la modalità Tile/Fluttuante globale, utilizzando la stessa preferenza della barra delle applicazioni e del Centro di Controllo. Il comando equivalente è `argvus config window-mode toggle`. Le invocazioni concorrenti vengono serializzate; l'attivazione utilizza la preferenza salvata anche quando l'applicazione è in attesa. La scorciatoia può essere cambiata o disabilitata nelle Scorciatoie da tastiera; `SUPER + Shift + Space` mantiene la sua azione a finestra singola.
