---
title: Temi
description: Installa, seleziona, personalizza, importa ed esporta temi ARGVUS.
---
Un tema in ARGVUS imposta i colori, le superfici, i wallpaper predefiniti e la geometria del layout utilizzata su tutto il desktop: Hyprland, Waybar, il Pannello di Controllo, la barra delle applicazioni, la schermata di blocco, le notifiche e le applicazioni ARGVUS. Questa guida copre tutto ciò di cui hai bisogno per lavorare con i temi come utente.

## Cosa è incluso in un tema

Ogni tema ufficiale è un'identità visiva completa con due modalità:

- **Sticky** — un layout compatto con piccoli spazi tra le finestre, senza spazi esterni e una barra delle applicazioni vicino al bordo dello schermo.
- **Spaced** — un layout spazioso con spazi più grandi, angoli arrotondati e margini intorno alla barra delle applicazioni e alle superfici della shell.

Selezionare un tema non cambia la sua identità: la palette, il colore di evidenziazione predefinito e l'associazione del wallpaper rimangono gli stessi tra le modalità. La modalità è un'impostazione separata, vedi [Temi ufficiali](/docs/components/argvus-themes/official-themes/).

## Pagine in questa guida

- [Installazione dei temi](/docs/components/argvus-themes/installing-themes/) — da dove provengono i temi ufficiali e come installarne uno o tutti con `pacman`.
- [Temi ufficiali](/docs/components/argvus-themes/official-themes/) — il catalogo, il colore di evidenziazione predefinito di ciascun tema e come scegliere un tema e la sua modalità.
- [Personalizzazione del colore di evidenziazione](/docs/components/argvus-themes/highlight-color/) — cambiare il colore di accentuazione indipendentemente dal tema.
- [Importazione ed esportazione dei temi](/docs/components/argvus-themes/import-and-export/) — esportare il tema attivo, con il tuo colore di evidenziazione e sfondo, come tema personale e importarlo su un altro computer.

## Riferimento rapido

| Attività | Dove |
| --- | --- |
| Scegli un tema | **Centro di Controllo → Aspetto → Temi → Temi Ufficiali** |
| Passa da Sticky a Spaziato | **Centro di Controllo → Aspetto → Modalità** |
| Cambia il colore di evidenziazione | **Centro di Controllo → Aspetto → Colore evidenziazione** |
| Esporta il tema attivo | **Centro di Controllo → Aspetto → Temi → Esporta tema** |
| Importa un tema | **Centro di Controllo → Aspetto → Temi → Importa tema** |
| Apri il selettore di temi dalla tastiera | `SUPER + SHIFT + T` |

## Da dove provengono i temi

ARGVUS include due temi predefiniti, **ARGVUS Dark** e **ARGVUS Light**, con il pacchetto `argvus-appearance`. Ogni altro tema ufficiale è un pacchetto separato `argvus-theme-<name>`. Installare un pacchetto tema lo aggiunge al selettore di temi; non è necessario riavviare la sessione. I tuoi temi personali, esportati o importati, sono elencati separatamente come **Temi Personalizzati**.

## Suggerimenti per la tastiera

Le pagine del tema del Centro di Controllo mostrano i loro tasti nel piè di pagina:

- `↑` / `↓` — naviga.
- `Enter` — applica o apri.
- `e` — esporta il tema attivo, in una pagina Temi.
- `i` — importa un tema, in una pagina Temi.
- `d` — elimina un tema personalizzato.
- `Esc` — torna indietro.

## Risoluzione dei problemi

- **Un tema che ho installato non appare.** Conferma che il pacchetto è installato con `pacman -Q argvus-theme-<name>`, quindi riapri il Centro di Controllo. Il pacchetto deve essere installato con `argvus-appearance` presente.
- **Il colore di evidenziazione è cambiato dopo aver cambiato tema.** Un colore di evidenziazione personalizzato sopravvive ai cambi di tema. Usa **Ripristina al tema predefinito** per restituire al tema il suo colore originale.
- **Un archivio di tema non è elencato per l'importazione.** Deve essere direttamente nella tua directory principale e contenere `<id>/theme.toml`. Vedi [Importazione ed esportazione dei temi](/docs/components/argvus-themes/import-and-export/)].
