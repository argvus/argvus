---
title: Ruoli del repository
description: Responsabilità dei repository di origine ARGVUS dopo la consolidazione dello spazio di lavoro.
---
I repository sono raggruppati per responsabilità piuttosto che per pagina di navigazione:

- workspace: `argvus` (dispatcher, comandi in-process, helper GTK 4, crate condivise, il greeter in `crates/argvus-greeter` e i dati dei comandi);
- sessione e integrazione: `argvus-session`, `argvus-hyprland`, `argvus-portal`, `argvus-lock`, `argvus-boot-splash`, e il greeter (`argvus-greeter`, costruito da `argvus/crates/argvus-greeter`);
- UI fuori dal workspace: Pannello di Controllo, barra delle applicazioni (`argvus-taskbar`), launcher e Waybar;
- provider: display, alimentazione, firewall, notifiche;
- risorse e packaging: aspetto, sfondi, font, icone, profili.

## Consolidato nel workspace

Questi repository sono stati uniti in `de/argvus`. La loro documentazione ora si trova sotto `docs/` del workspace, e i loro repository vengono conservati solo per la cronologia:

| Ex repository | Ora | Documentazione |
| --- | --- | --- |
| `argvus-config` | `argvus config` | [linea di comando](/docs/reference/command-line/), [file di configurazione](/docs/reference/configuration-files/) |
| `argvus-i18n` | `argvus i18n` | [Componente di internazionalizzazione](/docs/components/argvus-i18n/) |
| `argvus-keys`, `argvus-projects` | `argvus keys`, `argvus projects` | linea di comando |
| `argvus-terminal` | `argvus terminal` | [Componente Terminale](/docs/components/argvus-terminal/) |
| `argvus-system-monitor` | `argvus system-monitor` | [Componente monitor di sistema](/docs/components/argvus-system-monitor/) |
| `argvus-network` | `argvus network`, `argvus bluetooth` | [Componente di rete](/docs/components/argvus-network/) |
| `argvus-widget-telemetry` | `argvus widget-telemetry` | linea di comando |
| `argvus-control-center` | `argvus control-center` (applicazione terminale) | [Componente Centro di Controllo](/docs/components/argvus-control-center/) |
| `argvus-taskbar-calendar` | `argvus taskbar-calendar` (helper GTK 4) | [Componente calendario della barra delle applicazioni](/docs/components/argvus-taskbar-calendar/) |
| `argvus-removable-devices` | `argvus removable-devices` (helper GTK 4) | [Componente dispositivi rimovibili](/docs/components/argvus-removable-devices/) |
| `argvus-loading-theme` | `argvus loading-theme` (helper `splash`) | riga di comando |
| `argvus-tui-terminal` | `argvus tui-terminal` (crate nello spazio di lavoro) | riga di comando |
| `argvus-tui`, `argvus-theme` | Crates condivisi `crates/argvus-tui` e `crates/argvus-theme` | note per sviluppatori |
| `argvus-greeter` | Sorgente in `crates/argvus-greeter`, incluso nel pacchetto `argvus` | [Componente greeter](/docs/components/argvus-greeter/) |
| `argvus-themes` | Ritirato. Ogni tema ufficiale è un proprio pacchetto `argvus-theme-*`. | [Installazione dei temi](/docs/user-guide/appearance/) |

Le note dello sviluppatore sui componenti consolidati sono in `docs/internal/<repository>/`.

La lingua non è il confine di proprietà: ogni componente mantiene i propri helper di runtime con il proprio pacchetto. L'unica regola ferrea che attraversa i pacchetti: solo `argvus-config` scrive sotto `data/generated/`. Un componente che deve raggiungere un'applicazione che non può leggere `config.json` aggiunge un adattatore nel proprio helper e lascia che l'orchestratore lo chiami dopo il commit canonico; non scrive mai un file generato. I CLI pubblici appartengono a `/usr/bin`; gli helper interni appartengono a `/usr/lib/argvus/<component>` o nella directory dei comandi quando sono necessari a runtime; i dati indipendenti dall'architettura rimangono sotto `/usr/share/argvus/<component>`.

Vedere la [matrice dei pacchetti](/docs/reference/package-matrix/) per il confine del pacchetto installato.
