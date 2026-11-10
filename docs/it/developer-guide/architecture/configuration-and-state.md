---
title: Configurazione e stato
description: Stato condiviso, override e file di runtime generati.
---
L'utente normale root è `$XDG_CONFIG_HOME/argvus` (di solito `~/.config/argvus`). Le preferenze logiche gestite sono memorizzate in `config.json`. Tema, accento, sfondo, spaziatura e preferenze dei bordi sono canonici lì; i file puntati rimangono proiezioni di compatibilità derivate per gli attuali consumatori Lua e shell di Hyprland mentre questi consumatori migrano a `data/`.

`argvus-config` è l'unico scrittore sotto `generated/`. Ogni file consumer di proprietà ARGVUS è una proiezione di `config.json`, inclusi Hyprland, GTK, Qt6ct, Waybar, Quickshell, Dunst, Hyprlock, la barra delle attività e i profili di telemetria dei widget di Waybar, gli alberi di configurazione Yazi e Superfile, gli alberi dei profili del terminale e di Qt, i blocchi di font gestiti e il foglio di stile dei dispositivi rimovibili. I componenti risolvono prima le sovrascritture esplicite dell'utente, poi l'output generato, infine i valori predefiniti del pacchetto. Rofi non è una proiezione: `argvus-launcher` possiede i suoi file, e l'utente copia i file live sotto `data/rofi/`. La normale proiezione tema/ricarica non promuove mai né i file generati né quelli dei pacchetti a sovrascrittura utente. La documentazione e gli strumenti dovrebbero identificare la fonte logica invece di chiedere agli utenti di modificare l'output generato.

`argvus-appearance` possiede quindi l'orchestrazione di tema e accento, mentre `argvus-config` possiede lo stato canonico e la proiezione strutturata. `argvus theme-switch` (la transazione `argvus config theme-apply`) e `argvus config accent` convalidano, bloccano e confermano tramite `argvus-config` quando disponibile, quindi conciliano i consumatori di compatibilità che non possono leggere direttamente `config.json`. Quegli adattatori sono il file nativo Qt6ct, GTK `settings.ini` e `gsettings` hints, le applicazioni terminale e system-monitor, foot, la directory di configurazione propria di superfile e il greeter pre-autenticazione. Un helper che necessita di un nuovo adattatore deve aggiungerlo lì, mai all'interno di `generated/`.

La proprietà del runtime è intenzionalmente divisa per responsabilità: Rust possiede il deposito canonico, la validazione dello schema, le migrazioni, il modello dei percorsi, i lock, le scritture atomiche, gli hash delle proiezioni e i renderer strutturati. Shell rimane solo come piccoli punti di ingresso di compatibilità o adattatori per consumatori esterni come `hyprctl` e script di tema posseduti dagli asset. Python è riservato a test, audit e strumenti di sviluppo; non fa parte del runtime desktop critico.

`argvus config apply` è il confine di ricostruzione (`project` rimane l’ortografia di compatibilità). Rigenera output mancanti anche quando il documento canonico non è cambiato, rende i proiettori in staging e scambia atomicalmente l’albero `data/` solo dopo che tutte le proiezioni hanno avuto successo. Il ciclo di vita della sessione rimane di proprietà di `argvus sessionctl`, che utilizza quel manifesto per pianificare ricariche mirate.

Quando un chiamante ha già impegnato una generazione — `apply-theme`, un cambiamento di accento, un `set`/`patch` o un cambiamento di blocco widget-telemetria — `reload` **applica** quella generazione invece di proiettarla di nuovo. Legge il piano dal manifesto, ripristina i consumatori target e proietta solo quando il manifesto è assente o non ha lavori in sospeso. Riprojettare a quel punto ricostruirebbe un albero identico e scarterebbe il piano prodotto dalla mutazione appena effettuata.

Il manifesto registra anche la generazione operativa, l'hash della configurazione effettiva, i domini modificati e l'elenco dei file generati. `data/internal/generation.json` è un indicatore locale di proiezione usa e getta e non è mai una preferenza utente. `argvus config status` confronta questi record con la configurazione risolta e segnala file generati mancanti o in eccesso.

Le mutazioni seguono lo stesso confine del motore: `UserConfig` viene clonata in una `NextUserConfig`, convalidata, risolta in `EffectiveConfig`, proiettata e solo allora persistere. `.config-transaction.json` è lo stato di recupero operativo; `argvus config recover` riproietta deterministicamente il documento canonico precedente dopo un crash tra la pubblicazione di `data/` e `config.json`. Il ricaricamento viene richiesto solo dopo il commit, e un fallimento del ricaricamento non annulla una configurazione coerente persiste.

## Modello di risoluzione

```text
config.json logical model
    ↓
ARGVUS native compatibility state and per-app override
    ↓
ARGVUS generated runtime file
    ↓
/usr/share/argvus packaged default
    ↓
upstream default
```
I candidati esatti variano a seconda del componente, ma il principio è coerente: un override dell'utente ha priorità, l'output generato è uno stato derivato e i file pacchettizzati rimangono input di sola lettura. La sessione esporta `ARGVUS_CONFIG_HOME` e `ARGVUS_SYSTEM_CONFIG` in modo che gli helper e i servizi utente risolvano le stesse radici.

Il recupero durante il ricaricamento segue `recover → ensure → migrate → apply → runtime reload`, e il passaggio di proiezione viene saltato quando il manifest in arrivo descrive già una generazione commessa. `ensure` materializza `config.json` dai valori predefiniti dello schema solo quando il profilo manca, e riempie i campi assenti senza sostituire valori espliciti `false`, `0` o personalizzati. Un documento esistente malformato rimane un errore piuttosto che essere sostituito silenziosamente.

Le mutazioni persistenti confrontano le vecchie e nuove configurazioni risolte tramite `ChangedDomains`. Dopo il commit, viene inviata una richiesta di ricarica tramite `argvus sessionctl`; il fallimento della ricarica viene segnalato e registrato senza annullare la configurazione commessa. L'avvio della sessione esegue il recupero prima della materializzazione e proiezione del bootstrap, quindi i consumatori non iniziano contro una transazione abbandonata.

| Stato | Scopo |
| --- | --- |
| `config.json` | Preferenze logiche gestite canoniche; la singola fonte per tutto ciò che segue `generated/`. |
| `.active-theme`, `.accent-color`, `.accent-custom` | Input di migrazione legacy; non sono fonti runtime e non vengono rigenerati. |
| `.wallpaper-custom` | Input di migrazione legacy per `appearance.wallpaper`; non è una fonte runtime. |
| `.spaces`, `.borders` | Input di migrazione del layout legacy; i valori canonici sono archiviati sotto `layout` in `config.json`. |
| `fonts.conf` | Proiezione di compatibilità delle selezioni di font canoniche. |
| `generated/effects` | Proiezione di compatibilità degli effetti canonici; usa e getta. |
| `$XDG_STATE_HOME/argvus/config-projection.json` | Manifesto di proiezione: sezioni modificate, consumatori interessati e se è necessario un ricaricamento. Letto da `reload`; mai fonte di verità. |
| `$XDG_STATE_HOME/argvus/widget-telemetry-blocks` | Preferenze del blocco di telemetria legacy. Letto solo come fallback di migrazione; i valori canonici risiedono in `/control_panel/widget_telemetry_blocks` e il file non viene più scritto. |

Una proprietà è canonica quando viene letta da `config.json` e proiettata da lì. Una proprietà è derivata quando viene proiettata ma anche consumabile come input di migrazione. Qualsiasi cosa un componente scriva ancora durante l'esecuzione — una cache, un file di configurazione nativo sotto `$XDG_CONFIG_HOME/<app>`, un file di stato del greeter — è un target dell'adattatore e non deve essere riletta come configurazione.

La proiezione copre i consumatori per Hyprland, GTK/Qt, Waybar, Quickshell, terminali, notifiche, schermata di blocco e applicazioni native. Hyprland legge i colori dei bordi attivi e inattivi dal tema selezionato; un `appearance.accent_custom=true` esplicito è l'unico caso che sovrascrive il bordo attivo/di gruppo con l'accento dell'utente. Il cambio del tema riapplica questi valori tramite il riavvio del compositore, quindi non può mantenere i colori del tema precedente. Se la configurazione dell'utente viene ricreata, gli effetti predefiniti sono animazioni abilitate. Eliminando `generated/` ed eseguendo `argvus config apply` lo ricrea byte per byte da `config.json`; modificare direttamente i file generati non è duraturo. La diagnosi in fase di esecuzione deve identificare la tupla completa di esecuzione: percorso del binario, versione/revisione, root delle risorse, root della configurazione, root dei dati, root generato, root interno e manifesto della proiezione. `status --verbose` espone questa tupla e
confronta la generazione/hash pubblicata con la configurazione attualmente risolta. I test di sviluppo e dei pacchetti installati sono modalità separate; nessuno dei due può ricadere silenziosamente su un eseguibile non correlato o sulla home del sviluppatore. `apply` è anche il confine di riconciliazione per gli identificatori di tema combinati legacy: un valore come `argvus-dark-spaced` diventa `theme=argvus-dark` e `variant=spaced` prima della risoluzione. L'identità delle risorse pacchettizzate non è registrata in un manifesto in questa versione, quindi nessun file diagnostico la descrive.
