---
title: Internazionalizzazione ARGVUS
description: Cataloghi di lingue, selezione e supporto Unicode.
---
# ARGVUS Internazionalizzazione

ARGVUS fornisce le lingue Inglese (`en-US`), Portoghese Brasiliano (`pt-BR`), Spagnolo (`es-ES`), Francese (`fr-FR`), Italiano (`it-IT`), Tedesco (`de-DE`), Russo (`ru-RU`), Cinese Semplificato (`zh-CN`), Giapponese (`ja-JP`) e Coreano (`ko-KR`). L'inglese è il catalogo canonico e di fallback. Le otto traduzioni aggiuntive sono state prodotte con assistenza di traduzione automatica e correzioni tecniche; non è stata certificata una revisione indipendente da parte di madrelingua.

## Seleziona una lingua

Apri Centro di Controllo > Preferenze > Lingua e Regione > Lingua, oppure Impostazioni > Lingua. La pagina rileva i cataloghi installati e mostra i loro nomi nativi. Salva `LANG` in `$XDG_CONFIG_HOME/locale.conf` (predefinito `~/.config/locale.conf`). Abilita prima la locale di sistema UTF-8 corrispondente in Locali di Sistema. Esempi includono `es_ES.UTF-8`, `zh_CN.UTF-8`, `ja_JP.UTF-8` e `ko_KR.UTF-8`.

La preferenza non modifica `/etc/locale.conf`. Commenti e impostazioni della categoria regionale sono preservati. Il Pannello di Controllo viene riavviato dopo il salvataggio; effettuare il logout e il login per applicare la lingua per tutta la sessione. Gli utenti non configurati utilizzano l'inglese. Le variabili d'ambiente ereditate e i file linguistici ARGVUS legacy non sostituiscono questa preferenza. Il login greeter viene eseguito prima che la preferenza dell'utente sia disponibile e mantiene la propria lingua dell'account di servizio, l'inglese per impostazione predefinita.

Le varianti regionali utilizzano il catalogo installato per la loro lingua quando manca un catalogo esatto; ad esempio `es_MX.UTF-8` utilizza `es-ES`. `zh-Hans` seleziona il cinese semplificato. Il cinese tradizionale (`zh-Hant`, `zh-TW`, `zh-HK`, `zh-MO`) non viene mappato silenziosamente al cinese semplificato e ricade sull'inglese.

## Cataloghi e contributi

La sorgente è `resources/locales/<locale>/<domain>.json` nel repository `argvus`. I cataloghi installati si trovano in `/usr/share/argvus/i18n/`. Vengono forniti nel pacchetto master `argvus`. `locale.json` fornisce l'identificatore e i nomi nativo/di visualizzazione. Non esiste un catalogo PO separato né una piattaforma di traduzione online richiesta.

Usare chiavi semantiche e preservare i nomi dei segnaposto, comandi, opzioni, percorsi e scorciatoie da tastiera. Tradurre messaggi completi invece di assemblare frasi da frammenti tradotti. Le nuove chiavi canoniche devono essere tradotte in tutte e dieci le lingue. Mantenere la terminologia coerente all'interno di ciascun dominio e rivedere le traduzioni nel contesto, specialmente per autenticazione, permessi e azioni distruttive.

```sh
argvus i18n list
argvus i18n locale
argvus i18n get common welcome name=Alice
argvus i18n validate
```
`ARGVUS_I18N_DIR` seleziona una radice alternativa del catalogo per lo sviluppo e i controlli dei pacchetti in fase di staging. Le verifiche di validazione controllano domini, chiavi, valori stringa, segnaposti, traduzioni vuote e forme plurali di interi specifiche della località. La validazione dimostra l'integrità strutturale, non la correttezza linguistica.

## Unicode e font

`argvus-fonts` richiede i font Noto CJK oltre alle famiglie Latin, simboli ed emoji. Fontconfig fornisce un fallback per cinese, giapponese e hangul senza sostituire la famiglia primaria scelta dall'utente. Applicazioni e terminali necessitano comunque che il fallback dei font sia abilitato.

Gli helper di testo condivisi di Ratatui misurano le celle del terminale, preservano i cluster di grafemi estesi durante il troncamento e avvolgono il testo largo. Gli editor di testo spostano ed eliminano interi grafemi. Le aree estremamente strette non possono visualizzare un glifo a due celle in una sola cella; in quel caso, l'helper di hard-wrap utilizza un carattere di sostituzione. Emoji e caratteri di larghezza ambigua possono ancora differire tra le implementazioni di terminale/font. La politica di larghezza del terminale dovrebbe essere conforme alla larghezza Unicode normale di Ratatui.

La selezione della lingua non installa un metodo di input né cambia il layout della tastiera. Usa un metodo di input compatibile con Wayland quando inserisci cinese, giapponese o coreano.
