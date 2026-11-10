---
title: Linea di comando
description: Il comando argvus, i suoi sottocomandi, i flag mantenuti per le associazioni di tasti e dove ciascuno di essi viene eseguito.
---
`/usr/bin/argvus` è il punto di accesso pubblico di ARGVUS. È un programma Rust costruito dal crate `argvus-cli`. Ogni sottocomando viene eseguito all'interno di quel programma, esegue un eseguibile di supporto dalla directory dei comandi oppure esegue uno script confezionato.

Esegui `argvus --help` per l'elenco installato con la tua versione. I sottocomandi sono scritti senza il prefisso `--`. Le forme `--` vengono mantenute solo quando un collegamento da tastiera o una voce del desktop le utilizza.

## Dove viene eseguito ogni comando

- **In-process:** il comando è una libreria del workspace e viene eseguito all'interno di `argvus`. Non esiste un eseguibile separato. Questi sono `config`, `theme`, `i18n`, `keys`, `gpg`, `projects`, `terminal`, `system-monitor`, `network`, `bluetooth`, `focus status`, `widget-telemetry`, `tui-terminal`, `sensors` e `setup`.
- **Assistente:** un eseguibile separato in `/usr/lib/argvus/commands/`, installato dal pacchetto `argvus`. Si tratta di `control-center` (applicazione terminale), `argvus-du` (lanciato con `argvus du`) e le applicazioni GTK 4 `taskbar-calendar`, `removable-devices`, `focus` e `clock`.
- **Eseguibile privato:** un eseguibile che un altro pacchetto installa in `/usr/lib/argvus/commands/`. `argvus` lo avvia solo. Questi includono `session`, `start`, `tty`, `sessionctl`, `launcher`, `snippets`, `ssh`, `git`, `app-profiles-yazi-keymap`, `notifications`, `displayctl`, `firewall`, `accounts`, `taskbar-actions` e i punti di ingresso `greeter-*`.
- **Script:** uno script di shell in pacchetto eseguito con `/bin/sh`. Questi sono i comandi dello script Hyprland elencati di seguito.

`ARGVUS_COMMANDS_DIR` cambia la directory dei comandi, per test e controlli `DESTDIR`. Un eseguibile privato che non viene trovato viene cercato su `PATH`.

## Sottocomandi

| Comando | Corre | Note |
| --- | --- | --- |
| `argvus config ...` | In elaborazione | Modello di configurazione e strumenti. Raggiunto anche da `theme-switch` e `effects-toggle`. |
| `argvus theme ...` | In elaborazione | `apply ID` è l'interruttore del tema (lo stesso di `theme-switch`). Costruzioni del tema: `list` i temi installati, `check DIRECTORY` un tema rispetto ai modelli installati, `render ID [--variant sticky\|spaced] [--accent #RRGGBB] [--output DIRECTORY]`. Vedi [Aspetto e temi](/docs/developer-guide/subsystems/appearance-and-themes/). |
| `argvus i18n ...` | In elaborazione | Traduzioni: `get DOMAIN KEY [name=value...]`, `locale`, `list`, `validate [LOCALE]`. |
| `argvus keys ...` | In elaborazione | Gestore chiavi SSH. `gpg` è un alias per i sottocomandi GPG. |
| `argvus gpg ...` | In elaborazione | Chiavi GPG: `list`, `status`, `generate`, `export-public`, `export-private`, `import`, `import-private`, `delete`, `edit`, `keyserver`. |
| `argvus projects ...` | In lavorazione | Selettore di progetto. |
| `argvus terminal ...` | In lavorazione | Il profilo del terminale ARGVUS. |
| `argvus system-monitor ...` | In lavorazione | Il monitor di sistema btop; `--gpu` seleziona solo le schede GPU, e `--panel [--cpu|--mem|--gpu]` lo incorpora nella shell TUI di ARGVUS. |
| `argvus du` | Helper (applicazione terminale) | Apre l'ARGVUS Disk Usage Analyzer in una finestra dedicata di Kitty ed esegue ncdu. |
| `argvus network ...` | In lavorazione | Stato e controlli di NetworkManager (`networkctl` viene mantenuto come alias). |
| `argvus bluetooth ...` | In lavorazione | Stato e controlli del Bluetooth (`bluetoothctl` viene mantenuto come alias). |
| `argvus widget-telemetry block ...` | In lavorazione | Il blocco di telemetria di Waybar. Questo è il percorso principale della barra. |
| `argvus widget-telemetry toggle ...` | In lavorazione | Attiva e disattiva i blocchi di telemetria, e ne gestisce l'ordine. |
| `argvus game-snake` | Eseguibile privato | Il gioco del serpente nel terminale, dal progetto indipendente `argvus-game-snake`, un pacchetto opzionale. Fuori da un terminale ne apre uno. |
| `argvus control-center [page]` | Helper (applicazione da terminale) | Apre il Centro di Controllo o mette a fuoco la finestra già aperta. |
| `argvus welcome [--if-needed]` | Dispatcher → Centro di Controllo | Apre la pagina di Benvenuto integrata nel Centro di Controllo. `--if-needed` si apre una volta sola a meno che i record di progresso per utente non indichino che la Benvenuto sia già stata mostrata, completata o chiusa. |
| `argvus taskbar-calendar [args]` | Applicazione GTK 4 | Predefinito su `toggle`. |
| `argvus removable-devices [args]` | Applicazione GTK 4 | Predefinito su `menu`. |
| `argvus focus [--toggle] [--start\|--pause] [--x X --y Y] [--bar-bottom PX --bar-right PX]` | Applicazione GTK 4 | Popup di focus ARGVUS. Un'istanza per sessione. |
| `argvus focus status --waybar [--follow]` | In lavorazione | Emette l'indicatore della barra delle applicazioni Focus come JSON di Waybar; `--follow` trasmette le modifiche senza caricare GTK. |
| `argvus clock [--toggle] [--background] [--page PAGE] [--x X --y Y] [--bar-bottom PX --bar-right PX]` | Applicazione GTK 4 | Popup dell'orologio ARGVUS. Un'istanza per sessione; `--background` mantiene gli allarmi attivi senza un popup. |
| `argvus loading-theme ...` | Pacchettizzato `splash` a `/usr/lib/argvus/loading-theme/splash` | Il sovrapposizione dei ricaricamenti diretti della sessione; un cambio tema non lo utilizza. |
| `argvus session`, `start`, `tty`, `sessionctl` | Eseguibili privati | Punti di ingresso della sessione e controllore della sessione. |
| `argvus greeter`, `greeter-session`, `greeter-tui`, `greeter-setup` | Eseguibili privati | Punti di ingresso del greeter. |
| `argvus accounts`, `firewall` | Eseguibili privati | Strumenti per account e firewall. |
| `argvus launcher`, `snippets`, `ssh`, `git` | Eseguibili privati | Strumenti di avvio e selezione. |
| `argvus tui-terminal` | In-process | Wrapper del terminale per applicazioni TUI. |
| `argvus sensors` | In-process | Panoramica della temperatura in tempo reale da `sensors -j`. |
| `app-profiles-yazi-keymap` | Eseguibile privato | Assistente per la mappatura dei tasti. |
| `argvus notifications`, `displayctl`, `taskbar-actions` | Eseguibili privati | Notifiche, visualizzazione e azioni nella barra delle applicazioni. |
| `argvus setup [options]` | In-process | Copia i valori predefiniti confezionati nella configurazione dell'utente. Vedi sotto. |

Gli argomenti vengono inoltrati senza modifiche, inclusi gli spazi e i valori vuoti. L'input standard, l'output, l'errore, i segnali e lo stato di uscita appartengono al comando che viene eseguito. Un nome sconosciuto fallisce mostrando un suggerimento per eseguire `argvus --help`. Un comando eseguito in-process rifiuta un argomento che non è valido UTF-8 (stato di uscita 2) invece di passare un valore alterato; i comandi che eseguono un programma separato ricevono i byte senza modifiche.

## Flag e alias

Questi nomi esistono affinché keybindings, voci del desktop e script più vecchi continuino a funzionare. Preferire la forma con sottocomando nel nuovo codice.

| Flag o alias | Equivalente |
| --- | --- |
| `--control-center`, `--about`, `--default-apps` | `control-center` (con pagine `about` e `apps`) |
| Pagina `--displays`, `--monitors` | `control-center displays` |
| `--network-interfaces` | `control-center network interfaces` |
| `--system-monitor`, `--terminal`, `--game-snake` | `system-monitor`, `terminal`, `game-snake` |
| `--calendar`, `--removable-devices`, `--storage` | `taskbar-calendar`, `removable-devices` |
| `--focus`, `--clock` | `focus`, `clock` |
| `--spf`, `--superfile` | il file manager (`spf`) |
| `--yazi`, `--yazy` | il browser dei file (`yazi`) |
| `--setup` | `setup` |
| `--start-desktop` | avvia `tty` (nessun argomento viene inoltrato) |
| `--version`, `-V`, `version` | stampe `ARGVUS <version>` |
| `help`, `--help`, `-h` | stampa l'utilizzo |
| `networkctl`, `bluetoothctl` | `network`, `bluetooth` |
| `widget-telemetry-toggle`, `widget-telemetry-block` | `widget-telemetry toggle`, `widget-telemetry block` |
| `theme-switch` | `config theme-apply` |
| `effects-toggle` | `config effects` |

Il Centro di controllo accetta i flag di pagina legacy come primo argomento e li associa ai nomi delle pagine. Esempi: `--apps` diventa `apps`, `--keyboard-layout` diventa `locale` e `--hostname` diventa `system`.

## Comandi script Hyprland

Questi eseguono script pacchettizzati e sono utilizzati dalla configurazione di Hyprland:

`cheatsheets-hypr`, `cheatsheets-launcher`, `scratchpad`, `hypr-screenshot`, `spaces-switch`, `emoji-picker`, `toggle-sidebar`, `weather-location`, `layout-mode-menu`, `brightness-switch`, `toggle-mode`, `idle-timeout`, `keep-awake`, `hypr-power-menu`.

Uno script nella configurazione dell'utente sostituisce quello pacchettizzato. L'ordine di ricerca è lo stesso di `_config_path` in `hyprland.lua`: la copia della configurazione dell'utente, poi la copia dei dati ARGVUS, poi la copia dello stato, poi la copia generata, e solo allora il file pacchettizzato sotto `/usr/share/argvus`.

## Esempi

```sh
argvus control-center displays
argvus taskbar-calendar toggle
argvus widget-telemetry block header system
argvus widget-telemetry toggle status
argvus i18n get widget-telemetry label.device
argvus config get /appearance/theme
argvus network status
argvus bluetooth status
argvus gpg list
```
## Configurazione

`argvus setup` copia i valori predefiniti confezionati da `/usr/share/argvus/<app>` in `~/.config/argvus/data/<app>`, in modo che tu possa personalizzarli. Non modifica mai i file confezionati.

```sh
argvus setup --copy <app>          # copy one packaged app
argvus setup --copy-all            # copy every packaged app directory
argvus setup --copy <app> --force  # back up the existing destination first
argvus setup --copy <app> --dry-run
```
Una destinazione che esiste già viene segnalata e lasciata intatta, a meno che non sia fornito `--force`. Con `--force`, la directory esistente viene rinominata in `<name>.bak-<timestamp>` prima della copia.

L'app `argvus` è speciale: copia il contenuto della sua directory pacchettizzata nella radice di configurazione. Poiché la radice di configurazione esiste già quando viene eseguita l'installazione, `--copy argvus` ha bisogno di `--force` nel codice corrente.

L'installazione valida ogni opzione prima di modificare qualsiasi cosa. I nomi delle app non validi (vuoti, che iniziano con `.` o che contengono `/`) vengono respinti.

## Sessione e ricarica

`SUPER + SHIFT + R` esegue `argvus sessionctl reload`. Il controller di sessione applica la configurazione generata e riavvia solo i componenti di sua proprietà. Non copia `/usr/share/argvus` nella directory home e non sostituisce un file JSON malformato.

## Luce notturna

Apri **Desktop e Input > Schermi > Luce Notturna** oppure esegui `argvus control-center displays night-light`. Configura temperatura, attivazione manuale e orari locali fissi tramite `argvus config display night-light`. Vedi [Luce Notturna](/docs/user-guide/night-light/) per comandi, stato e requisiti di esecuzione.

`argvus tui-terminal --frame TITLE FOOTER COMMAND [ARG...]` esegue un comando interattivo all'interno del frame terminale condiviso ARGVUS. Ctrl+E chiude il frame; il chiamante fornisce il footer localizzato. Usa questo all'interno di un terminale, oppure come comando passato a `argvus tui-terminal --term kitty -- ...`.

`argvus tui-terminal --term kitty --profile ai-agents` utilizza una configurazione Kitty dedicata con schede nascoste e solo scorciatoie per gli appunti. Il chiamante fornisce il comando e il titolo; la posizione della finestra è gestita dalla regola Hyprland per la classe `argvus-ai`.
