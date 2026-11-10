---
title: Matrice del pacchetto
description: Responsabilità del pacchetto ARGVUS.
---

| Pacchetto | Responsabilità |
| --- | --- |
| `argvus` | Pacchetto principale: dispatcher, comandi in esecuzione (configurazione, traduzioni, chiavi, progetti, terminale, monitor di sistema, rete, Bluetooth, telemetria, terminale TUI, configurazione), i dati condivisi e i cataloghi i18n. Ogni pacchetto componente dipende da esso. |
| `argvus` (helpers) | Helper nello stesso pacchetto: il Centro di Controllo e l'Analizzatore dell'Uso del Disco (applicazioni terminale), oltre alle applicazioni GTK 4 per il calendario della barra delle applicazioni, dispositivi rimovibili, overlay di caricamento, ARGVUS Focus e ARGVUS Clock. |
| `argvus` (greeter) | Saluto di accesso per greetd, fornito nello stesso pacchetto e costruito da `crates/argvus-greeter`. |
| `argvus-desktop` | Profilo desktop e applicazioni quotidiane richiesto dal pacchetto principale `argvus`. Non dipende da `argvus`, quindi la dipendenza rimane unidirezionale. |
| `argvus-devtools`, `argvus-pentest` | Metapacchetti opzionali indipendenti per strumenti di sviluppo e auditing di sicurezza autorizzato. Il pacchetto principale non installa questi profili. |
| `argvus-session` | Punti di ingresso della sessione e ciclo di vita dell'utente. |
| `argvus-hyprland` | Configurazione di Hyprland. |
| `argvus-control-panel` | Pannello di controllo Quickshell. |
| `argvus-taskbar`, `argvus-waybar` | Configurazione di Waybar e binario patchato. |
| `argvus-branding` | Risorse di logo e marchio testuale. |
| `argvus-appearance`, `argvus-wallpapers`, `argvus-fonts`, `argvus-icons` | Stato dell'aspetto e risorse visive. |
| `argvus-display`, `argvus-power`, `argvus-notifications` | Provider del desktop. |
| `argvus-lock`, `argvus-boot-splash` | Schermata di blocco e splash di avvio. |
| `argvus-firewall` | Firewall, con i propri privilegi. |
| `argvus-launcher` | Configurazione del launcher e punti di ingresso. |
| `argvus-app-profiles` | Profili solo dati per le applicazioni terminali incluse. |
| `argvus-portal` | Predefiniti XDG portal e Wayland/DBus. |
| `argvus-theme-*` | Un pacchetto opzionale per ciascuna famiglia di temi ufficiale. |
| `argvus-game-snake`, `argvus-games` | Opzionale. Snake è un progetto indipendente; `argvus-games` raggruppa i giochi opzionali. |

## Pacchetti sostituiti

`argvus` dichiara `replaces`, `conflicts` e `provides` per i pacchetti che ha assorbito, quindi un aggiornamento li rimuove e i loro file non rimangono orfani:

`argvus-config`, `argvus-i18n` (con `argvus-language-en-us` e `argvus-language-pt-br`), `argvus-keys`, `argvus-projects`, `argvus-terminal`, `argvus-system-monitor`, `argvus-network`, `argvus-widget-telemetry` e `argvus-du`. `argvus` sostituisce anche `argvus-control-center`, `argvus-taskbar-calendar`, `argvus-loading-theme`, `argvus-removable-devices` e i precedenti pacchetti divisi `argvus-gtk` e `argvus-greeter`, che continua comunque a fornire in modo che i pacchetti dipendenti continuino a risolversi. Il gestore account (`argvus accounts`) è costruito da `crates/argvus-accounts` e installato come `/usr/lib/argvus/commands/accounts`, con la sua regola polkit. L'Analizzatore di Utilizzo Disco è costruito da `crates/argvus-du` e installato come `/usr/lib/argvus/commands/argvus-du`.

`/usr/bin/argvus-terminal` rimane installato come launcher di compatibilità per `argvus terminal`. Il terminale predefinito di Hyprland, il catalogo del Centro di Controllo e le applicazioni predefinite salvate continuano a nominare quel comando.

I pacchetti rimossi `argvus-pwvucontrol` e `argvus-snappy-switcher` non sono forniti da nessun pacchetto.

Non è richiesto un ordine di installazione tra i pacchetti componenti ARGVUS: `argvus` è il loro pacchetto principale, e ogni componente dipende da esso. L'installazione di `argvus` installa anche il profilo `argvus-desktop`; `argvus-devtools` e `argvus-pentest` rimangono opzionali. La presenza di un pacchetto in questa tabella non significa che possieda l'intera funzionalità: per esempio, la barra delle applicazioni combina `argvus-waybar`, `argvus-taskbar`, `argvus-session` e i provider di dominio.
