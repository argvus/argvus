---
title: Temi ufficiali
description: Il catalogo ufficiale dei temi ARGVUS, i colori di evidenziazione predefiniti e come selezionare un tema e la sua modalità Sticky o Spaced.
---
Questa pagina elenca i temi ufficiali disponibili come pacchetti di temi e spiega come selezionarne uno. Per installarli, vedi [Installing themes](/docs/components/argvus-themes/installing-themes/).

## Seleziona un tema

1. Apri **Centro di Controllo → Aspetto → Temi**.
2. Scegli **Temi Ufficiali**.
3. Scegli **Scuro** o **Chiaro**.
4. Scegli la famiglia di temi. Il tema viene applicato non appena lo selezioni.

Il tema selezionato viene applicato all'intero desktop contemporaneamente: la barra delle applicazioni in esecuzione, i pannelli, le notifiche, le finestre e lo sfondo cambiano colore sul posto, senza schermata di caricamento e senza riavviare. ARGVUS mantiene la selezione dello sfondo a meno che il tema non venga applicato con la sua associazione di sfondo.

Puoi anche premere `SUPER + SHIFT + T` per aprire il selettore dei temi dalla tastiera. Usa i tasti freccia per muoverti tra le categorie, `Enter` per scegliere e `Esc` per chiudere.

## Fisso e Spaziato

Ogni tema funziona in due modalità. La modalità è indipendente dal tema: cambiare il tema mantiene la modalità corrente e cambiare la modalità mantiene il tema corrente.

| Modalità | Spazi tra le finestre | Margine esterno | Arrotondamento finestre | Margine taskbar e shell |
| --- | ---: | ---: | ---: | ---: |
| Sticky | `2` | `0` | `0` | vicino al bordo |
| Distanziato | `10` | `18` | `4` | `18` |

Per cambiare modalità, apri **Centro di Controllo → Aspetto → Modalità** e scegli **Sticky** o **Spaced**. Le tue regolazioni manuali di questi valori di layout rimangono attive fino alla prossima selezione esplicita del tema.

## Temi scuri

| Tema | Pacchetto | Evidenziazione predefinita |
| --- | --- | --- |
| Uno scuro | `argvus-theme-one-dark` | `#61AFEF` |
| Dracula | `argvus-theme-dracula` | `#BD93F9` |
| Argento Scuro | `argvus-theme-silver-dark` | `#595959` |
| Ardesia scura | `argvus-theme-slate-dark` | `#7391A5` |
| Universo | `argvus-theme-universe` | `#EEEEEE` |
| Gruvbox Alto Scuro | `argvus-theme-gruvbox-high-dark` | `#D79921` |
| Gruvbox Scuro | `argvus-theme-gruvbox-dark` | `#D4BE98` |
| Pino Rosato | `argvus-theme-rose-pine` | `#C4A7E7` |
| Notte di Tokio | `argvus-theme-tokyo-night` | `#7AA2F7` |
| Solitudine | `argvus-theme-solitude` | `#798186` |
| Tramonto | `argvus-theme-sunset` | `#E2BE8A` |
| Matrice | `argvus-theme-matrix` | `#82FB9C` |
| Monokai scuro | `argvus-theme-monokai-dark` | `#78DCE8` |
| Signora Nebbia | `argvus-theme-lady-mist` | `#354B78` |

## Temi chiari

| Tema | Pacchetto | Evidenziazione predefinita |
| --- | --- | --- |
| GitHub Chiaro | `argvus-theme-github-light` | `#0969DA` |
| Solarized Chiaro | `argvus-theme-solarized-light` | `#268BD2` |
| Nord Chiaro | `argvus-theme-nord-light` | `#5E81AC` |
| One Chiaro | `argvus-theme-one-light` | `#4078F2` |
| Everforest Chiaro | `argvus-theme-everforest-light` | `#3A94C5` |
| Catppuccin Latte | `argvus-theme-catppuccin-latte` | `#1E66F5` |
| Gruvbox Chiaro | `argvus-theme-gruvbox-light` | `#458588` |
| Acquerello | `argvus-theme-watercolor` | `#1A639C` |
| Obsidian Chiaro | `argvus-theme-obsidian-light` | `#334A78` |

## Temi integrati

**ARGVUS Scuro** e **ARGVUS Chiaro** sono forniti da `argvus-appearance` e sono sempre disponibili sotto **Temi Ufficiali**. ARGVUS Scuro utilizza l'evidenziazione predefinita `#3590BD`; ARGVUS Chiaro utilizza `#181818`.

## Cosa controlla un tema

Un tema definisce la tavolozza e il trattamento della superficie utilizzati dai componenti del desktop, il colore di evidenziazione predefinito, l'associazione dello sfondo predefinito e i valori predefiniti di trasparenza e sfocatura. I componenti del desktop leggono questo stato anziché memorizzare una copia propria, quindi una modifica del tema raggiunge insieme il compositore, la barra delle applicazioni, il Pannello di controllo, la schermata di blocco, le notifiche e le applicazioni.

Il colore di evidenziazione predefinito è solo il punto di partenza. Puoi sostituirlo con qualsiasi colore a sei cifre, vedi [Personalizzare il colore di evidenziazione](/docs/components/argvus-themes/highlight-color/).
