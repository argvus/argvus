---
title: Configurazione ARGVUS
description: Il contratto di configurazione JSON logico utilizzato da ARGVUS.
---
ARGVUS conserva le preferenze utente gestite in file JSON modulari sotto `$XDG_CONFIG_HOME/argvus/config/` (normalmente `~/.config/argvus/config/`). Questi costituiscono il modello logico canonico per le preferenze gestite da ARGVUS, e `argvus-config` è l'unico componente che scrive file sotto `data/generated/`. I file nativi e i file generati rimangono proiezioni di compatibilità o sovrascritture specifiche dell'applicazione.

## Guida utente: Avvio rapido

Per la maggior parte degli utenti, la configurazione avviene tramite il Centro di Controllo ARGVUS o questo strumento da riga di comando:

```sh
# View your configuration location and validate it
argvus config path
argvus config validate

# Get a specific value
argvus config get /appearance/theme
argvus config get /appearance/theme --raw    # Without JSON formatting

# Set a value (must be valid JSON)
argvus config set /appearance/theme '"gruvbox-dark"'

# Apply changes to reload the session
argvus config apply
```
Per gli amministratori e per la configurazione avanzata, continua a leggere qui sotto.

L'utente root contiene solo il documento canonico e i dati gestiti:

```text
~/.config/argvus/
├── config.json
└── data/
```
Il file viene scritto in modo atomico, mantenuto privato per l'utente e protetto da `.config.lock`, al di fuori dell'albero `data/` che può essere scambiato durante la proiezione. Una versione precedente è conservata in `data/backups/config.json.bak`.

## Linea di comando

Il sottocomando `argvus config` è fornito in-process dal pacchetto `argvus`:

```sh
argvus config path
argvus config validate
argvus config ensure
argvus config migrate
argvus config status
argvus config explain /layout/gaps_out
argvus config get /appearance/theme --effective --raw
argvus config set /appearance/accent '"#61AFEF"'
argvus config patch appearance-patch.json
argvus config audio volume +5
argvus config audio mute toggle
argvus config apply-theme universe --variant sticky --accent '#EEEEEE' --gtk-mode dark
argvus config apply-theme universe-spaced --variant spaced --accent '#EEEEEE' --gtk-mode dark
argvus config theme-manifest universe-spaced
argvus config unset /appearance/accent
argvus config apply
argvus config rebuild
argvus config project
argvus config project --force
```
`ensure` materializza un nuovo `config.json` con i valori predefiniti dello schema, oppure riempie i campi mancanti senza sostituire i valori espliciti. Fallisce in caso di JSON malformato. `migrate` importa lo stato di aspetto legacy esistente senza sostituire i valori già presenti in `config.json`. `argvus sessionctl` esegue `recover`, `ensure`, `migrate` e `project` prima di applicare un reload dopo le modifiche di configurazione:

```sh
argvus sessionctl apply-config
```
`apply` e `rebuild` sono alias per ricostruire ogni proiezione da `config.json`; `project` è l'ortografia storica e accetta `--force`. `apply` delega anche il ricaricamento a runtime post-commit a `argvus sessionctl`. Imposta `ARGVUS_NO_RUNTIME=1` per preparazione o test isolati. La generazione avviene in staging e l'albero `data/` viene scambiato solo dopo che tutti i proiettori hanno avuto successo. Pertanto, eliminare `data/generated/` ed eseguire `argvus config apply` è una ricostruzione riproducibile senza pubblicare una sequenza parziale di output di Hyprland o Taskbar. Il comando registra gli hash delle sezioni derivate e la decisione a runtime in `$XDG_STATE_HOME/argvus/config-projection.json` (normalmente `~/.local/state/argvus/config-projection.json`). Né il manifest né i file generati sono fonti di verità. `apply` esegue il ricaricamento post-commit tramite `argvus sessionctl`; `project` rimane solo per la proiezione a scopo di compatibilità.

Ogni mutazione persistente utilizza la stessa regola: se il documento canonico è cambiato, il CLI invoca un unico `argvus sessionctl reload` centralizzato dopo il commit. Il risultato del reload viene registrato in `data/internal/last-reload.json`.

`argvus sessionctl reload` è quindi un passo di *applicazione*, non un passo di proiezione. Quando il manifesto in arrivo segnala lavoro in sospeso, ripristina i consumatori target descritti dal piano e non chiama nuovamente `argvus config project`; riproiettare ricostruirebbe un albero identico e scarterebbe il piano appena prodotto dalla mutazione. Ripiega sulla migrazione e sulla proiezione solo quando non esiste un piano commesso, ad esempio durante l'avvio della sessione.

### Obiettivi del reload

`argvus config session-plan` risponde al reload con i target delle sezioni che sono cambiate, e `argvus sessionctl` esegue solo quelli. Lo splash copre un reload solo quando un target ricrea una superficie: una barra Waybar o il Pannello di Controllo che si sposta di nuovo sullo schermo sposta ogni finestra. Tutto il resto viene ricaricato in posizione.

| Modifica | Obiettivi | Splash |
| --- | --- | --- |
| Tema, colore evidenziato o modalità GTK | `hyprland`, `consumers`, `wallpaper`, `polkit` | no |
| Solo sfondo | `wallpaper` (hyprpaper IPC; il servizio si riavvia solo per swaybg) | no |
| `effects` (sfocatura, animazioni, trasparenza, ombra) | `hyprland`, `consumers` | no |
| `layout` delle finestre (spazi, bordi) | `hyprland` | no |
| `layout.taskbar` (posizione, margini) | `hyprland`, `taskbar`, `widget_telemetry` | sì |
| `fonts` | `consumers` | no |
| `control_panel` (interruttore, schede) | `control_panel` | sì |
| `default_apps`, `keyboard_shortcuts`, `hyprland`, `displays` | `hyprland` | no |
| `power` | `hypridle` | no |

`consumers` esegue `argvus config theme-runtime consumers` (i fogli di stile della barra, il Pannello di Controllo tramite IPC, i colori di Hyprland, il ricaricamento del tema di Dunst, btop e kitty). Dunst rinvia il ricaricamento del tema mentre Non Disturbare è attivo. `wallpaper` esegue `argvus config theme-runtime wallpaper`, che mostra l'effettivo `/appearance/wallpaper`. L'hash `layout.taskbar` è mantenuto separato dal layout delle finestre nel manifesto di proiezione, quindi una modifica del divario non riavvia le barre. Il piano di riserva `full`, utilizzato quando non esiste un piano impegnato, riavvia ogni superficie sotto lo splash.

Ogni configurazione efficace ha una generazione operativa. Essa è memorizzata in `data/internal/generation.json` e nel manifesto di proiezione sotto `$XDG_STATE_HOME/argvus/config-projection.json`; non fa parte della configurazione dei dotfiles. Riapplicare una configurazione efficace non modificata preserva la generazione. `status` segnala se l'hash del manifesto, i metadati della generazione e tutti i file generati sono coerenti; `apply` ripara una proiezione obsoleta o incompleta.

`explain /path` legge direttamente il resolver e stampa il valore efficace, la fonte, il tema e la variante. Non analizza i file generati né crea una seconda regola di precedenza.

La proiezione di temi ed effetti è implementata nello stesso `argvus-config`. La transazione del tema (`argvus theme-switch`, che esegue `theme-apply`), `argvus config accent` e `argvus effects-toggle` non proiettano: esse validano, bloccano, confermano tramite questo CLI e poi riconciliano i consumatori esterni che non possono leggere `config.json`. `argvus effects-toggle` è un delegato leggero senza output proprio. Gli assistenti dell'aspetto non vengono mai chiamati dal proiettore e non devono mai scrivere al di sotto di `data/generated/`.

### Superfici di proiezione

`project_effective` rende ogni consumatore di proprietà ARGVUS in un albero di staging prima della pubblicazione:

| Superficie | Output |
| --- | --- |
| Hyprland | `generated/hypr/`, compresi i profili del tema, `spaces-effective.conf`, `borders-effective.conf`, `hypridle.conf` e `monitors.lua` |
| Layout/effetti | `generated/effects/`, `generated/spaces.conf`, `generated/borders.conf` |
| Waybar | `generated/waybar/`, compresi `argvus-taskbar.css` e `argvus-widget-telemetry.{jsonc,css}` |
| Temi della telemetria dei widget | `generated/widget-telemetry/` e `generated/waybar/widget-telemetry-themes/` |
| QuickShell | `generated/quickshell/` |
| Notifiche | `generated/notifications/` e `dunstrc` |
| Schermata di blocco | `generated/hypr/hyprlock.conf` e i profili dei temi di Hyprtoolkit |
| GTK | `generated/gtk/` |
| Yazi | `generated/yazi/` |
| Superfile | `generated/superfile/` |
| Profili di Terminale e Qt | `generated/terminal/`, `generated/qt6ct/` |
| Caratteri | `generated/fonts.conf` più blocchi di caratteri gestiti nei fogli di stile di Waybar |
| Dispositivi rimovibili | `generated/removable-devices/theme.css` |
| Sfondo | `generated/hypr/hyprpaper.conf` e `data/hypr/hyprpaper.conf` |
| Pannello di controllo e impostazioni predefinite | `data/control-panel/`, `data/control-center/defaults.json` |
| Calendario, input, tastiera, alimentazione | calendario TOML, frammento di input, stato della tastiera, marcatore di mantenimento attivo |

Il livello del tema è anche rispecchiato al di fuori della radice generata:
`data/waybar/argvus-taskbar.{jsonc,css}`,
`data/waybar/argvus-widget-telemetry.{jsonc,css}`, `data/foot/`,
`data/qt6ct/`, `data/hypr/hyprtoolkit.conf` e
`data/hypr/application-style.conf`. Quelle copie vengono sostituite dai valori predefiniti inclusi nel pacchetto quando la sezione aspetto cambia, e il proiettore riscrive poi solo i suoi blocchi gestiti delimitati — il blocco dei font nei fogli di stile di Waybar e i blocchi `ARGVUS_TELEMETRY_*` nel profilo di telemetria — quindi il testo al di fuori dei marcatori non sopravvive a un cambio di tema. `fonts.targets` e
`control_panel.widget_telemetry_blocks` restano canonici: il Centro di Controllo
li mantiene attraverso `argvus-config` e non scrive mai questi file da solo.

La selezione del blocco di telemetria del widget è canonica a `/control_panel/widget_telemetry_blocks`, un array dei sette nomi dei blocchi (`system`, `cpu_gpu`, `memory`, `storage`, `processes`, `network`, `keys`). Il file legacy `$XDG_STATE_HOME/argvus/widget-telemetry-blocks` è di sola lettura come fallback di migrazione e non viene mai scritto. ` and ` and blocks all aggiorna l'array canonico in una singola scrittura atomica, e `apply-state` presume che la selezione sia già stata salvata dal suo chiamante.

Le preferenze di spaziatura e bordi di Hyprland seguono lo stesso confine. I comandi argvus config spaces e argvus config borders (e i loro wrapper di compatibilità storica) aggiornano /layout; Rust genera data/generated/spaces-effective.conf e data/generated/borders-effective.conf e gestisce la convalida e l'applicazione live con hyprctl. .spaces e .borders sono stato esclusivamente per migrazione e non sono mai la fonte di verità.

Lo stato dell'accento e della modalità GTK utilizza lo stesso confine Rust. Utilizzare ` or ` or argvus config accent --theme-default per la proprietà dell'accento, e `argvus config mode toggle|set dark|light` per la modalità GTK. Gli script storici di accent e modalità sono wrapper di compatibilità; `.accent-color`, `.accent-custom` e `.gtk-mode` sono solo file di migrazione/proiezione.

Anche i percorsi logici di ARGVUS sono risolti da `argvus config paths`. Il file legacy `paths.sh` espone solo funzioni di compatibilità; override nativi, dati utente, dati generati e fallback di sistema sono risolti dal modello di percorsi Rust.

`audio` cambia l'uscita predefinita e registra il risultato in `/audio` in un solo passaggio: `argvus config audio volume <N|+N|-N> [--limit PERCENT]` e `argvus config audio mute <on|off|toggle>`. Scrive lo store senza ricaricare la configurazione, perché PipeWire ha già il valore. Il modulo del volume di Waybar, i tasti multimediali di Hyprland e il Pannello di Controllo lo utilizzano, quindi `argvus config project`, che proietta `/audio` sul sink, non ripristina mai un volume precedente. Un volume superiore al 100% (i tasti di Hyprland consentono fino al 150%) è memorizzato come 100.

`patch` applica un oggetto JSON le cui chiavi sono puntatori JSON in un'unica operazione bloccata di lettura-modifica-scrittura. `apply-theme` esegue la transazione atomica equivalente del tema e sostituisce le sezioni di aspetto gestite dal profilo di aspetto opzionale del tema o dai valori predefiniti inclusi. I comandi di mutazione costruiscono un `NextUserConfig`, risolvono un `EffectiveConfig`, preparano tutti i proiettori e solo allora pubblicano il JSON canonico. Un marcatore operativo `.config-transaction.json` permette a `argvus config recover` di riproiettare la configurazione precedente dopo un crash tra i dati e la pubblicazione del JSON. I valori predefiniti del tema sono dichiarati dal manifesto di ciascun tema (`themes/<theme>/theme.toml`): l'accento predefinito (`palette.accent`), lo schema di colori GTK e lo sfondo del desktop. I file CSS, Lua, Rasi e Qt vengono generati da esso durante la compilazione del tema.

Cambiare il tema, inclusi Chiaro/Scuro, reimposta il colore di evidenziazione al valore predefinito del tema, anche quando l'utente ha scelto un colore personalizzato: il colore personalizzato viene mantenuto solo fino al prossimo cambio di tema. Usare `argvus config accent` dopo il cambio del tema per impostarlo di nuovo. Applicare un tema con `argvus config theme-apply` scrive `/appearance/accent` e `/appearance/gtk_mode` dal manifesto, e `argvus config accent` scrive il colore scelto con `accent_custom` impostato, in un unico comando.

`apply-theme` prende la variante esplicitamente come `--variant sticky|spaced` e risolve il nome richiesto attraverso il manifesto, quindi una richiesta non può mai risolversi negli asset di un'altra variante. `theme-manifest THEME` è la stessa risoluzione esposta in sola lettura: stampa l'entry del manifesto insieme a `asset_id` e `layout_variant` a cui è stata risolta, ciò di cui un chiamante ha bisogno per indirizzare gli asset confezionati. Sticky è l'id senza suffisso e solo Spaced porta un suffisso `-spaced`, quindi `dracula-sticky` e `argvus-dark-dracula-sticky` sono accettati e normalizzati in `dracula`.

Con `--raw`, i valori di tipo stringa vengono stampati senza virgolette JSON, mentre oggetti e array rimangono valori JSON. I file puntati di tema e layout non vengono più scritti dall'helper di proiezione; `config.json` rimane l'unica fonte persistente per queste preferenze. Se la proiezione fallisce, il caricamento della sessione si interrompe prima di riavviare le superfici del desktop, quindi un tema parziale più vecchio non viene presentato come corrente.

## Esportazione e importazione

Lo stesso modello logico viene utilizzato per lo scambio dei profili:

```sh
argvus config export --scope appearance --output appearance.json
argvus config import --scope appearance appearance.json
```
Usa `--scope desktop` per il modello completo di desktop gestito. I campi mancanti usano i valori predefiniti di ARGVUS; l'output generato non dovrebbe essere modificato direttamente.

L'esportazione e l'importazione del tema del Centro di Controllo scambiano un tema e il suo profilo visivo: l'archivio contiene una directory di temi di schema 2 (`<id>/theme.toml`, opzionali `appearance.json` e `effects.conf`, `overrides/`, lo sfondo) che si installa sotto `$XDG_DATA_HOME/argvus/themes/`. Il profilo contiene aspetto, geometria del layout, effetti visivi, font e presentazione del pannello. Preferenze audio, tastiera, rete e altre preferenze non visive restano fuori dai temi. Usa l'ambito `appearance` sopra per spostare l'intera configurazione dell'aspetto, e l'ambito `desktop` per un backup completo della configurazione.

Lo schema attuale raggruppa le preferenze sotto `appearance`, `layout`, `effects`, `fonts`, `control_panel`, `default_apps`, `keyboard_shortcuts`, `hyprland`, `displays`, `power`, `session`, `calendar` e `removable_devices`. `default_apps` è l'unica fonte persistente per le applicazioni preferite, e `keyboard_shortcuts` memorizza chiavi stabili in inglese, indipendenti dalla localizzazione, con valori stringa o `null` per le associazioni disabilitate.

Il Centro di Controllo legge i valori canonici dell'aspetto quando apre una pagina Aspetto e scrive le modifiche tramite `argvus-config`. I file di stato nativi esistenti rimangono proiezioni per gli script di aspetto e per le installazioni più vecchie.

Il Pannello di Controllo utilizza `control_panel.cards` per l'ordine delle schede e la visibilità, e `control_panel.enabled` per l'interruttore principale del pannello. I suoi file `control-panel/cards.json` e `state/control-panel` sono conservati come proiezioni di compatibilità. Le modifiche apportate nel Pannello di Controllo fanno quindi parte dell'ambito di esportazione `desktop`.

Le preferenze dei font utilizzano `fonts.targets` per la famiglia, lo stile e la dimensione della barra delle applicazioni, della telemetria, del Pannello di Controllo, del sistema, delle applicazioni, del terminale e degli obiettivi del browser. `fonts.rendering` memorizza le preferenze di antialiasing, hinting, ordine dei subpixel e DPI. `fonts.conf`, la regola di fontconfig, le impostazioni del desktop e i blocchi di font gestiti all'interno dei fogli di stile di Waybar sono proiezioni: il Centro di Controllo persiste `/fonts/targets/*` tramite `argvus-config`, e il proiettore riscrive solo il suo blocco delimitato.

Le preferenze di Hyprland utilizzano `keyboard_shortcuts` per le scorciatoie, `hyprland.input` per le impostazioni di mouse e touchpad, e `hyprland.keyboard` per layout XKB, variante e opzioni. Un ID di binding del manifesto si mappa su uno `config_key` stabile (per esempio `window.close` → `close_window` e `window.drag_mouse` → `drag_window__floating_window_only`). Le preferenze del display utilizzano `displays.monitors`, `displays.workspaces` e `displays.primary_monitor`. I file TOML nativi e i file Lua generati sotto `data/generated/` rimangono proiezioni consumate da Hyprland.

Ogni interruttore `effects` è impostato di default su attivo (animazioni, sfocatura, telemetria dei widget e ogni superficie trasparente), anche su macchine virtuali e sessioni a basso consumo; l'ombra della finestra (`shadow_enabled`) e il wobble (`wobble_enabled`) sono impostati di default su spento. Le preferenze degli effetti utilizzano `effects` per l'interruttore principale delle animazioni (`animations`, applicato a `hyprland.lua` dal proiettato `hypr/state.lua`, indipendentemente dall'interruttore Effetti, che è `blur_global_enabled` più il `transparency_<surface>_enabled` di `taskbar`, `control-panel`, `widget-telemetry`, `terminal` e `launchers`, e risulta attivo quando uno qualsiasi di essi lo è; `argvus effects-toggle animations|effects` li cambia), l'interruttore globale della sfocatura (`blur_global_enabled`), i parametri di sfocatura di Hyprland (`blur_size`, `blur_passes`, `blur_brightness`, `blur_noise`, `blur_contrast`, `blur_vibrancy`, `blur_vibrancy_darkness`), l'ombra della finestra (`shadow_enabled`, `shadow_sharp`, `shadow_color`, `shadow_range`, `shadow_render_power`, `shadow_offset_x`, `shadow_offset_y`, `shadow_scale`), i valori delle Animazioni (`animation_<leaf>_speed`, `_curve`, `_style`, `_popin` e `_enabled` per ogni animazione, più `animation_bezier_x1` fino a `animation_bezier_y2` e `animation_spring_mass`, `animation_spring_stiffness` e `animation_spring_dampening` per le curve personalizzate) e i valori di trasparenza per superficie. Gli effetti sono di proprietà del tema, non della variante: Sticky e Spaced utilizzano la stessa proiezione degli effetti `<theme>.conf` mentre le loro proiezioni di layout differiscono. Le superfici attualmente includono `taskbar`, `control-panel` e `widget-telemetry`; i valori di trasparenza sono percentuali da `0` a `100` (le superfici del Centro di Controllo e del launcher di default sono `15`, le altre `50`), i parametri di sfocatura sono numeri entro gli intervalli dichiarati in `schema.json` e gli stati abilitati sono booleani. I valori delle Animazioni sono validati anche da `argvus-config`: le velocità sono numeri da `1` a `10` (decimali consentiti), le percentuali pop-in sono numeri interi da `1` a `100`, le curve devono essere nomi dalla pagina Animazioni, gli stili devono essere nomi dalla pagina Animazioni e accettati dalla foglia stessa dell'animazione (ad esempio `popin` viene rifiutato per `animation_workspace_in_style`, `gnomed` è accettato solo per l'apertura e la chiusura della finestra e le varianti di scorrimento vengono rifiutate sulla finestra e sulle ante del livello), i punti Y di Bézier rimangono tra `-1` e `2`, l'intervallo Hyprland 0.56.2 analizza e gli altri parametri della curva rimangono entro gli intervalli documentati. Le foglie `layers` e `switching` non esistono più, quindi le configurazioni salvate prima della loro rimozione perdono le chiavi `animation_layers_enabled`, `animation_layers_speed`, `animation_layers_curve` e `animation_switching_*` quando vengono caricate. Gli stili esterni al set corrente vengono eliminati e i punti Y di Bézier esterni da `-1` a `2` vengono bloccati, quindi le impostazioni predefinite del pacchetto si applicano nuovamente a qualunque cosa sia stata rimossa.

Le preferenze di alimentazione utilizzano `power.screen_off_minutes`, `power.lock_minutes`, `power.keep_awake` e le policy del coperchio/pulsante di alimentazione. I moduli proiettati sono `data/generated/hypr/hypridle.conf` e il marcatore `data/power/keep-awake`; `.keep-awake` è solo un input di migrazione. `session.language` è ritirato: i valori esistenti sono ignorati, le nuove scritture sono rifiutate e nessun file di compatibilità linguistica viene proiettato. Impostare `LANG` nell'utente `locale.conf` invece. Le preferenze del calendario utilizzano `calendar` e sono proiettate sul calendario della barra delle applicazioni utente TOML. Le preferenze dei dispositivi rimovibili utilizzano `removable_devices`; l'applicazione dei dispositivi rimovibili tratta questa sezione come un override dell'utente sui valori predefiniti inclusi nel pacchetto.

`migrate` importa una volta i vecchi file root e le sezioni, preservando un valore canonico esistente. I vecchi `defaults.json`, `/defaults`, `/hyprland/keybindings`, `keybindings.toml`, i file con punto, `generated/` e le directory dei componenti vengono spostati sotto `data/`; i ricaricamenti ripetuti non ricreano il vecchio layout root. Selezionare di nuovo il tema corrente è idempotente: se lo stato canonico e le proiezioni sono già in accordo, il piano di proiezione segnala sezioni non modificate e la sessione salta lo splash e il riavvio dei servizi.

L'alimentazione segue lo stesso contratto di proiezione: il Control Center scrive `/power` in `config.json`, `argvus config project` rende `data/hypr/hypridle.conf` in modo atomico, e solo allora può `argvus sessionctl` riavviare `argvus-hypridle.service`. Rimuovere quel file è recuperabile con `argvus config project`; un valore di `0` omette il timeout corrispondente e significa Mai.

L'aspetto della schermata di blocco viene proiettato da Rust dal tema canonico, accento, effetti, sfondo e stato dei font in `data/hypr/hyprlock.conf`. Il punto di ingresso storico `hyprlock-theme.sh` è solo per compatibilità e delega a `argvus config lock apply`; `--invalidate` invalida solo la cache dello sfondo sfocato derivato.

Le modifiche del timeout di alimentazione utilizzano `/power/lock_minutes` e `/power/screen_off_minutes` come loro fonte di verità. `argvus config project` riutilizza il renderer Hypridle esistente, e il punto di ingresso `idle-timeout.sh` è solo un adattatore di compatibilità; il riavvio del servizio rimane delegato a `argvus sessionctl`.

L'aspetto delle notifiche è derivato dal tema canonico, accento e effetti in `data/notifications/dunstrc`. Il file storico delle notifiche `theme.sh` espone solo funzioni di compatibilità che chiamano `argvus config notifications`; non analizza né modifica la configurazione di Dunst.

Gli adattatori orientati all'hardware, come la scoperta del display e il controllo Bluetooth, sono ora importati dal dominio display di Rust: `nwg-displays` è un importatore esterno, mentre `/displays` e `data/generated/hypr/monitors.lua` rimangono rispettivamente lo stato canonico e derivato. Gli strumenti display modificano il layout con `argvus config monitor-set`, `monitor-primary`, `monitor-workspaces` e `monitor-remove`; i comandi display sono `argvus config display status|list-modes|set|apply|apply-nwg|migrate-nwg|nwg-import|nwg-apply`. Il file Hyprland è proiettato da `/displays`; nessuno script shell lo scrive più. La selezione del wallpaper, la modalità del gruppo di utilità della taskbar e la ricerca delle app predefinite utilizzano anche le API `argvus-config`; i loro punti di ingresso storici in Shell sono wrapper UI o di compatibilità. Keep Awake è una preferenza `/power/keep_awake` persistente proiettata sul marker `data/power/keep-awake`, mentre lock-DPMS è uno stato di runtime della sessione sotto
`$XDG_CACHE_HOME/argvus/lock-dpms`. Il layout della tastiera è ora gestito dal Rust `argvus-keyboard-layout` helper: l'ordine configurato proviene dalla proiezione di input generata di Hyprland o dal fallback della localizzazione, mentre il layout attivo resta lo stato di runtime della sessione scoperto da Hyprland. Il punto di ingresso del ciclo e il demone del layout della tastiera sono solo wrapper di compatibilità; il servizio utente esegue direttamente il demone Rust e `argvus sessionctl keyboard-layout cycle` è il comando stabile per i chiamanti.

Lo stato di esecuzione di Bluetooth segue il confine dell'adattatore esterno ma non fa parte di `argvus-config`: `argvus bluetooth` gestisce l'analisi tipizzata e la validazione intorno a `bluetoothctl`, i tentativi limitati, i controlli dei servizi di sistema e lo stato RFKill. `argvus bluetoothctl` è il punto di ingresso. L'alimentazione dell'adattatore, i dispositivi abbinati e i dispositivi connessi rimangono nello stato di esecuzione di BlueZ piuttosto che nello stato JSON canonico; la validazione del Bluetooth hardware-attivo dipende dall'ambiente.
## Identità di runtime e riconciliazione

Usa `argvus config status --verbose` per ispezionare il binario esatto, l'identità di build, la root di configurazione, la root generata, la root degli asset, la fonte del manifesto e la generazione attiva. Riporta gli stati di proiezione `synchronized`, `stale`, `incomplete`, `legacy` e `pending-transaction`.

`argvus config --version` segnala la versione del pacchetto, la revisione della build quando fornita dall'ambiente di build e il profilo. I test di sviluppo devono inserire esplicitamente il loro binario e le risorse; non devono utilizzare silenziosamente `argvus config` o `~/.config/argvus`. `apply` normalizza in sicurezza gli identificatori combinati legacy come `argvus-dark-spaced` in valori separati di tema e variante prima di ricostruire le proiezioni. L'identità delle risorse pacchettizzate non è registrata in un manifesto in questa release.

## Luce notturna

Apri **Desktop & Input > Displays > Night Light** oppure esegui `argvus control-center displays night-light`. Configura temperatura, attivazione manuale e pianificazioni locali fisse tramite `argvus config display night-light`. Vedi [Night Light](/docs/user-guide/night-light/) per comandi, stato e requisiti di esecuzione.

### Impostazioni Wobble (Hyprland 0.57+)

`effects.wobble_enabled`, `wobble_stiffness`, `wobble_damping`, `wobble_mass`, `wobble_intensity` e `wobble_mesh` proiettano direttamente in `hypr/state.lua`. Usa `argvus effects-toggle wobble-settings get` per leggerli, oppure `argvus effects-toggle wobble-settings set enabled=enabled stiffness=100 damping=10 mass=1 intensity=1 mesh=8` per salvare e richiedere un ricaricamento. `enabled=disabled` disattiva l'effetto.

Il Centro di Controllo condivide la bozza delle Animazioni: `animations-settings get` restituisce anche le voci `wobble_*`, e `animations-settings set` accetta questi nomi completi insieme ai suoi argomenti di animazione esistenti. Questo conserva l'intera bozza in una patch validata e richiede un ricaricamento. Per esempio:

```sh
argvus effects-toggle animations-settings set enabled=enabled wobble_enabled=true wobble_intensity=0 wobble_mesh=8
```
Il `enabled` globale controlla le animazioni; `wobble_enabled` è la sua preferenza indipendente. Disabilitare le animazioni globali sopprime il wobble in Lua senza cancellare quella preferenza. I valori non validi rifiutano l'intera patch. Le chiavi mancanti ricevono i valori predefiniti di ARGVUS senza sostituire le scelte esistenti. Vedi [Appearance](/docs/user-guide/appearance/#window-wobble) per limiti e valori predefiniti.

### Scorciatoie da tastiera create dall'utente

`/keyboard_shortcuts` mantiene i suoi valori stringa-o-nullo esistenti per le azioni ufficiali del manifesto. Le scorciatoie da tastiera globali personalizzate utilizzano un oggetto `/hyprland/custom_shortcuts` separato, memorizzato in `config/hyprland.json`. I suoi ID stabili `custom_` sono indipendenti dalle descrizioni e dalle località. Ogni record ha `description`, `keys`, `enabled` e un `action` tipizzato:

```json
{
  "custom_editor": {
    "description": "Open my editor",
    "keys": "SUPER + CTRL + E",
    "enabled": true,
    "action": { "type": "command", "command": "editor --new-window" }
  },
  "custom_float": {
    "description": "Toggle floating",
    "keys": "SUPER + CTRL + F",
    "enabled": false,
    "action": {
      "type": "dispatcher",
      "dispatcher": "window.float",
      "arguments": { "action": "toggle" }
    }
  }
}
```
L'esempio è il valore di `/hyprland/custom_shortcuts`, non un documento di configurazione completo. I nomi dei dispatcher provengono dalle fabbriche Lua dispatcher supportate; gli argomenti sono un oggetto JSON opzionale o `null`, mai codice valutato. Le scorciatoie personalizzate non hanno sotto-mappa, binding del mouse o flag avanzati. La proiezione scrive `data/generated/hypr/custom-shortcuts.lua` come tabella dati escape, includendo una tabella vuota quando l'ultimo record viene rimosso. Sia la proiezione completa che `project --only keyboard_shortcuts` la generano. Hyprland registra le voci abilitate con `hl.bind`; eventuali fallimenti nella registrazione del file e individuali sono contenuti e registrati.

Il Centro di Controllo crea/modifica record e disabilita conflitti confermati utilizzando una transazione `argvus config patch`. Salva prima di applicare tramite il servizio di configurazione esistente. Un'applicazione fallita non annulla un salvataggio riuscito. Il ripristino dei valori predefiniti ufficiali preserva i record personalizzati e viene rifiutato se colliderebbe con un collegamento personalizzato attivo. Le descrizioni degli utenti rimangono testo letterale in tutte le localizzazioni e nel foglio di riferimento generato.

## Modalità finestra globale

Aprire **Centro di Controllo → Desktop e Input → Hyprland → Finestre → Modalità** e selezionare **Finestre a Piastrelle (Predefinito)** o **Finestre Fluttuanti**. Lo stesso selettore è disponibile dall'icona della modalità finestra nel gruppo di utilità della barra delle applicazioni.

La preferenza si applica a ogni monitor e spazio di lavoro e viene ricordata dopo il logout. Floating converte le finestre delle applicazioni esistenti e le nuove finestre aperte. Tile ripristina solo le finestre che questa politica ha convertito; le finestre di dialogo e altre finestre che erano già in modalità flottante rimangono flottanti. Una finestra fissata esplicitamente mentre Floating è attivo rimane anch'essa flottante per preservare la sua fissazione. Il layout a tessere viene ricalcolato; il suo esatto albero delle divisioni precedente non viene ripristinato.

`SUPER + SHIFT + Space` mantiene il suo normale comportamento per finestra in Tile. In Floating non riporta una finestra alla modalità a tessere. I pannelli, la barra delle applicazioni e le notifiche sono superfici layer-shell e non sono interessati. La modalità a schermo intero e la massimizzazione continuano a usare il comportamento normale di Hyprland.

La preferenza canonica è `/hyprland/window_mode` in `$XDG_CONFIG_HOME/argvus/config/hyprland.json`. La sua proiezione generata è `data/generated/hypr/window-mode.lua`; non modificarla. Il registro di ripristino per finestra si trova sotto `$XDG_RUNTIME_DIR/argvus`, limitato all'istanza del compositor, e sopravvive ai ricaricamenti della configurazione. Non viene riutilizzato in una nuova sessione.

Se l'applicazione è in attesa o fallisce, l'interfaccia distingue questo dalla preferenza salvata. Seleziona nuovamente l'opzione per riprovare. Senza un compositor, la preferenza viene salvata per il prossimo accesso.

```sh
argvus config window-mode get
argvus config window-mode status --json
argvus config window-mode set floating
argvus config window-mode set tile
argvus config window-mode toggle
```
## Pulsanti della finestra e barra dell'applicazione

La modalità flottante aggiunge una barra del titolo ARGVUS con **Riduci a icona**, **Ingrandisci / Ripristina** e **Chiudi** alle normali finestre delle applicazioni. Fare doppio clic sulla barra del titolo attiva o disattiva l'ingrandimento. La decorazione aggiuntiva è nascosta in Tile e a schermo intero. Le applicazioni che disegnano i propri controlli possono visualizzare entrambe le barre; i controlli uniformi ARGVUS sono intenzionali.

Una seconda Waybar appare su ogni monitor, sul bordo opposto rispetto alla barra delle applicazioni principale. Elenca solo le finestre appartenenti allo spazio di lavoro corrente di quel monitor, incluse le finestre minimizzate di quello spazio di lavoro. Uno spazio di lavoro speciale aperto ha la precedenza. Le finestre fissate rimangono visibili; le schede dei gruppi nascoste non creano voci aggiuntive, e un gruppo minimizzato ha una voce di recupero. Le schede all'interno di un'applicazione non vengono elencate separatamente. Le voci minimizzate mantengono la loro associazione originale con monitor/spazio di lavoro. Clicca su una finestra inattiva per attivarla, su una finestra attiva per minimizzarla, o su una finestra minimizzata per ripristinarla; il suo menu contestuale offre le opzioni ripristina, minimizza, massimizza/ripristina e chiudi. Le icone delle applicazioni sconosciute vengono sostituite da un simbolo di finestra. Attivare o ripristinare una voce mantiene la posizione del puntatore. La barra secondaria utilizza il CSS primario effettivo (colori, font, bordi e arrotondamenti), con la sua altezza e margini speculari sul bordo opposto.

`SUPER + Alt + Up` posiziona la barra principale in alto e la barra della finestra in basso. `SUPER + Alt + Down` fa il contrario. Ripetere una direzione lascia le posizioni invariate. Tile ha solo la barra principale. Tornare a Tile ripristina tutte le finestre minimizzate prima di rimuovere la barra della finestra e poi conserva la politica originale a piastrelle/galleggiante.

La minimizzazione utilizza lo spazio di lavoro riservato `special:argvus-minimized`. Non utilizzare manualmente quello spazio di lavoro. Il processo `argvus windows service` registra l'identità, lo spazio di lavoro/monitor originale, la geometria, lo stato a schermo intero/massimizzazione e lo stato di pinning in un registro runtime specifico per istanza del compositore sotto `$XDG_RUNTIME_DIR/argvus`. I record sopravvivono a un riavvio del servizio o a un ricaricamento della configurazione, ma non attraversano le sessioni del compositore. La chiusura di una finestra rimuove il suo record. Se un monitor scompare, il ripristino utilizza un'uscita disponibile. Una barra di recupero mancante o guasta impedisce nuove minimizzazioni; un ripristino fallito mantiene il record e la barra di recupero.

La sessione possiede `argvus-windows.service` e `argvus-window-bar.service`. Quest'ultimo mantiene il suo figlio Waybar inattivo in Tile. Il plugin hyprbars è fornito da `argvus-hyprland`, costruito per l'ABI di Hyprland 0.57.0 su x86_64; non è compilato nella directory home dell'utente. Ricompilare il plugin con un aggiornamento dell'ABI del compositore.Comandi: `argvus windows list --json`, `watch --json`, `position top|bottom`, `restore-all` e `minimize|restore|activate|maximize|close ADDRESS`. Gli indirizzi delle finestre provengono dalla lista. `position top --persist` memorizza/proietta la preferenza senza riavviare la barra principale. I dettagli degli errori di configurazione e runtime vengono scritti nel registro; i fallimenti delle azioni visibili mostrano anche una notifica tradotta.

In **Spaziato + Fluttuante**, le finestre rispettano i margini esterni e le aree riservate da entrambe le barre. I confini includono il bordo e il titolo, che si trova all'interno dello stesso bordo della finestra. La massimizzazione tramite ARGVUS controlla riempie quest'area con margini; il ripristino recupera la geometria precedente, anche dopo il riavvio di un servizio. Lo schermo intero utilizza comunque l'intero output. Le finestre fluttuanti possono sovrapporsi; gli spazi interni nella disposizione a piastrelle non le separano.

I margini interni riservati dalle barre non vengono conteggiati due volte nei gap esterni della finestra. Con la spaziatura simmetrica, la distanza visibile tra finestra e barra è uguale sopra e sotto, anche dopo aver invertito le barre.

`SUPER + Alt + Space` alterna la modalità globale Piastrella/Fluttuante, utilizzando la stessa preferenza della barra delle applicazioni e del Centro di Controllo. Il comando equivalente è `argvus config window-mode toggle`. Le invocazioni concorrenti vengono serializzate; l'alternanza utilizza la preferenza salvata anche quando l'applicazione è in sospeso. La scorciatoia può essere cambiata o disabilitata nelle Scorciatoie da tastiera; `SUPER + Shift + Space` mantiene la sua azione per finestra singola.
