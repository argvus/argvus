---
title: Calcolatrice ARGVUS
description: Base · Scientifica · Programmazione · Finanziaria · Conversione di unità
---

# Calcolatrice ARGVUS

`argvus calculator` · `SUPER + C` · launcher: **ARGVUS Calculator**

Scegliere un profilo in Impostazioni → Profilo con ↑/↓ e Invio oppure facendo clic. Profilo e opzioni vengono salvati subito e ripristinati al prossimo avvio. Ogni profilo conserva la propria espressione durante la sessione. Invio o = sostituisce l’espressione con il risultato senza selezionarlo, con il cursore alla fine.

I tassi sono percentuali per periodo; usare lo stesso periodo per tasso, numero e pagamento. I pagamenti avvengono a fine periodo. Il tasso zero è supportato. Gli interessi semplici e composti restituiscono il totale accumulato. La conversione è offline, senza valute. La cronologia salva fino a 100 voci con profilo, opzioni e campi finanziari; selezionare un risultato ripristina il contesto. Cancellare svuota anche il file salvato.

## Impostazioni

- `s Impostazioni` → **Profilo** → Base / Scientifica / Programmazione / Finanziaria / Conversione di unità.
- ↑ / ↓ · Enter · Escape / ←.
- **Copia risultato** · **Cronologia** · **Cancella cronologia**.
- **Base**: 370 × 420; **Scientifica**: 440 × 640.
- **Programmazione**: 520 × 640; **Finanziaria**: 440 × 640; **Conversione di unità**: 440 × 520.

## Base / Scientifica

`+ - * / %` · `50% = 0.5` · `1.5 = 1,5` · `2+2` → Enter → `4` → `+2` → Enter → `6`.

`sin`, `cos`, `tan`, `asin`, `acos`, `atan`, `sqrt`, `log`, `ln`, `exp`, `pi`, `e`, `^`, `(`, `)`.

**Unità angolare**: Gradi / Radianti. `-2^2 = -4`; `2^3^2 = 512`; `2*pi`.

IEEE 754 / f64 · 15 cifre significative · 4096 byte · 64 livelli di analisi.

## Programmazione

Predefinito: decimale, 64 bit con segno. L’aritmetica ritorna entro la larghezza scelta; lo scorrimento destro con segno conserva il segno. La divisione intera tronca verso zero. Divisione per zero e scorrimenti fuori da [0, bits) sono errori. Senza prefisso si usa la base scelta. Cambiare base converte l’espressione valida; larghezza o segno reinterpretano gli stessi bit.

- **Base**: 2 / 8 / 10 / 16; **Dimensione parola (bit)**: 8 / 16 / 32 / 64.
- **Con segno** / **Senza segno** · `0b1010`, `0o12`, `0xA`.
- `+ - * / %`, `AND OR XOR NOT`, `<< >>`, `& | ^ ~`, `(`, `)`.
- `9007199254740993+2 = 9007199254740995`; `0xFF AND 0b1010 = 10` (base 10).
- 8-bit: `255+1 = 0`; `NOT 0 = -1` (Con segno); `-7/2 = -3`.

## Finanziaria

Interesse semplice · Interesse composto · Rata fissa · Valore attuale · Valore futuro · Variazione percentuale.

- `P`: Capitale iniziale; `FV`: Importo futuro; `A`: Pagamento per periodo.
- `r`: Tasso per periodo (%) / 100; `n`: Numero di periodi.
- `G=(1+r)^n`; `H=(G-1)/r` (`r=0`: `H=n`).
- Interesse semplice: `P*(1+r*n)`; Interesse composto: `P*G`.
- Rata fissa: `P*G/H`; Valore attuale: `(FV+A*H)/G`; Valore futuro: `P*G+A*H`.
- Variazione percentuale: `(Importo finale - Importo originale) / Importo originale * 100`.

## Conversione di unità

Lunghezza · Massa · Temperatura · Area · Volume · Tempo · Velocità · Archiviazione dati.

**Da** / **A**: Impostazioni · `1500 m → 1.5 km` · `32 °F → 0 °C` · `K ≥ 0`.

`1 B = 8 bit`; `1 kB = 1000 B`; `1 KiB = 1024 B`; `US gal = 3.785411784 L`.

## Cronologia

100 · Profilo + `context` · `Cancella cronologia`.

`$XDG_DATA_HOME/argvus/calculator/history.json` (→ `~/.local/share/argvus/calculator/history.json`).

`$XDG_CONFIG_HOME/argvus/calculator/settings.json` (→ `~/.config/argvus/calculator/settings.json`).

`{"scientific":true}` → `{"profile":"scientific"}`; `{"scientific":false}` → `{"profile":"basic"}`.

`.desktop`: `/usr/share/applications/argvus-calculator.desktop` · `Name=ARGVUS Calculator` · `Exec=argvus calculator`.

Il testo segue Applications Font in Control Center → Appearance → Desktop Style → Fonts. Le applicazioni GTK ereditano gtk-font-name; le dimensioni di numeri e titoli restano. Il launcher legge apps_family generato e segue le modifiche.

L’intestazione unisce il badge ARGVUS nel colore di accento al nome dell’applicazione. Il piè di pagina cliccabile s Settings occupa una fascia a tutta larghezza con sfondo distinto del tema, nello stile del Control Center.
