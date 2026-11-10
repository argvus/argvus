---
title: Aspetto
description: Configura il linguaggio visivo condiviso dai componenti ARGVUS.
---
Il sistema di aspetto applica uno stato tema a Hyprland, GTK, Qt, Waybar, Quickshell, Rofi, Dunst, Hyprlock e alle applicazioni ARGVUS.

## Famiglie di temi

Le ventitré famiglie qui sotto sono definite dal payload del tema corrente. La colonna colore è il colore attivo del bordo/accento predefinito della famiglia; gli utenti possono scegliere un altro accent RGB valido.

| Famiglia | Resetta colore evidenza | Sfondo base | Modalità |
| --- | --- | --- | --- |
| One Dark | `#61AFEF` | `#282C34` | Sticky, Spaced |
| Dracula | `#BD93F9` | `#282A36` | Sticky, Spaced |
| ARGVUS Dark | `#3590bd` | `#111316` | Sticky, Spaced |
| Dark Silver | `#595959` | `#111316` | Sticky, Spaced |
| Dark Slate | `#7391a5` | `#2f3541` | Sticky, Spaced |
| Dark Universe | `#eeeeee` | `#000000` | Sticky, Spaced |
| ARGVUS Dark Gruvbox High | `#D79921` | `#282828` | Sticky, Spaced |
| ARGVUS Dark Gruvbox | `#D4BE98` | `#282828` | Sticky, Spaced |
| ARGVUS Luce | `#181818` | `#f7f7f7` | Appiccicoso, Distanziato |
| GitHub Luce | `#0969DA` | `#FFFFFF` | Appiccicoso, Distanziato |
| Catppuccin Latte | `#1E66F5` | `#EFF1F5` | Appiccicoso, Distanziato |
| ARGVUS Gruvbox leggero | `#458588` | `#FBF1C7` | Appiccicoso, Distanziato |
| Pino Rosato | `#C4A7E7` | `#191724` | Appiccicoso, Distanziato |
| Tokyo-Notte | `#7AA2F7` | `#1A1B26` | Appiccicoso, Distanziato |
| Solitudine | `#798186` | `#101315` | Appiccicoso, Distanziato |
| Tramonto | `#E2BE8A` | `#0F0F0F` | Appiccicoso, Distanziato |
| Matrice | `#82FB9C` | `#0B0C16` | Appiccicoso, Distanziato |
| Monokai scuro | `#78DCE8` | `#2D2A2E` | Appiccicoso, Distanziato |
| Signora Nebbia | `#354B78` | `#131313` | Appiccicoso, Distanziato |
| Una luce | `#4078F2` | `#FAFAFA` | Appiccicoso, Distanziato |
| Luce dell'Everforest | `#3A94C5` | `#FDF6E3` | Appiccicoso, Distanziato |
| Luce Solarizzata | `#268BD2` | `#FDF6E3` | Appiccicoso, Distanziato |
| Nord Luce | `#5E81AC` | `#ECEFF4` | Appiccicoso, Distanziato |
| Acquerello | `#1A639C` | `#E9E3D3` | Appiccicoso, Distanziato |

Il colore mostrato nella tabella è il valore scritto in `config.json` quando quella famiglia è selezionata o quando viene usato **Reimposta ai valori predefiniti del tema**. Un accento RGB a sei cifre scelto manualmente rimane applicato fino al prossimo cambio tema, che lo sostituisce con il valore predefinito del tema selezionato.

**Sticky** è il layout compatto: le finestre a mosaico utilizzano piccoli spazi interni, i margini esterni sono zero e la barra delle applicazioni è vicina al bordo dello schermo. **Spaced** aggiunge spazi maggiori, angoli arrotondati, ombre e margini intorno alla barra delle applicazioni e alle superfici della shell.

I valori di layout esistenti di `float` e gli identificatori di tema `-float` sono riconosciuti come nomi legacy per Spaced. Nuovi valori di layout e identificatori di tema risolti utilizzano `spaced` e `-spaced`.

La modalità cambia la geometria e il trattamento delle superfici; non crea una sesta famiglia di colori. Vedi [Themes](/docs/user-guide/appearance/) per la selezione] e [Desktop](/docs/user-guide/desktop/) per il posizionamento della barra delle applicazioni e delle finestre].

- [Temi](/docs/user-guide/appearance/)
- [Sfondi](/docs/argvus-appearance/wallpapers/)
- [Caratteri e icone](/docs/argvus-appearance/fonts-and-icons/)
- [Effetti](/docs/argvus-appearance/effects/)

Il Centro di Controllo espone le impostazioni visibili all'utente. L'implementazione è condivisa da `argvus-appearance`, `argvus-session`, dai componenti della shell e dai pacchetti di risorse. Vedi [architettura dell'aspetto](/docs/developer-guide/subsystems/appearance-and-themes/).

## Oscillazione delle finestre

Apri **Desktop e Input → Hyprland → Animazioni → Finestre** e utilizza i controlli **Oscillazione**. Cerca `wobble`, oppure apri direttamente la pagina:

```sh
argvus control-center hyprland animations windows
```
Il Wobble è disabilitato di default. I suoi valori predefiniti ARGVUS sono stiffness `100`, damping `10`, mass `1`, intensity `1` e mesh resolution `8`. Stiffness e mass accettano `0.0001–1000`, damping `0–1000`, intensity `0–2` e mesh un intero `2–32`. Una risoluzione della mesh più alta aumenta il costo di rendering. Usa i tasti freccia per regolare i valori o Invio per digitarli. **Ripristina valori predefiniti di Wobble** modifica il draft; **Applica** salva tutte le modifiche di animazione in sospeso e ricarica Hyprland una volta, senza riavviare la sessione. Tornare indietro in Animations mantiene il draft condiviso.

L'interruttore globale Animations sopprime il Wobble mantenendo le sue impostazioni individuali. I valori canonici risiedono sotto `effects.wobble_*` nello stato ARGVUS; `hypr/state.lua` viene generato e non deve essere modificato manualmente.

Questo richiede Hyprland **0.57.0 o più recente**, imposto dalla dipendenza del pacchetto di `argvus-hyprland`. La versione 0.56 non è supportata per questa configurazione. Il massimo dell'intensità segue il [codice sorgente rilasciato 0.57.0 ](https://github.com/hyprwm/Hyprland/blob/v0.57.0/src/config/values/ConfigValues.cpp), che lo limita a `2`, anche se documentazioni più recenti possono elencare `3`.
