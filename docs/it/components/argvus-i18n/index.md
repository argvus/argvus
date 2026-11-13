---
title: Internazionalizzazione ARGVUS
description: Cataloghi di lingue, selezione e supporto Unicode.
---

# ARGVUS Internazionalizzazione

ARGVUS navi 18 lingue di interfaccia: Arabo (`ar`), Bengali (`bn-BD`), tedesco (`de-DE`),
Inglese (inglese)`en-US`), spagnolo (`es-ES`), francese (`fr-FR`), Hindi (`hi-IN`), indonesiano
(`id-ID`), italiano (`it-IT`), giapponese (`ja-JP`), coreano (`ko-KR`), portoghese brasiliano
(`pt-BR`), portoghese europeo (`pt-PT`), russo (`ru-RU`), turco (`tr-TR`), Urdu
(`ur-PK`), Vietnamita (`vi-VN`) e cinese semplificato (`zh-CN`). L'inglese è
catalogo canonico e fallback. Le traduzioni sono state prodotte con AI assistenza e
correzioni tecniche; la revisione indipendente madrelingua non è stata certificata.

## Seleziona una lingua

Centro di controllo aperto > Preferenze > Lingua e regione > Lingua. La pagina scopre
cataloghi installati e mostra i loro nomi nativi. Salva `ARGVUS_LANGUAGE` in
`$XDG_CONFIG_HOME/locale.conf` (default) `~/.config/locale.conf`), indipendentemente da
`LANG`; generare un sistema locale non è richiesto. Portoghese brasiliano (Italia)`pt-BR`) e
Portogallo europeo (`pt-PT`) sono scelte separate.
La preferenza non modifica `/etc/locale.conf` o regionale `LANG` impostazione.
I commenti e le impostazioni regionali di categoria sono conservati. Il pannello di controllo è riavviato
dopo il salvataggio; avviare una nuova sessione per applicare la lingua durante i componenti di sessione.
Gli utenti non configurati utilizzano l'inglese. I controlli separati delle preferenze locali
`LANG`, formattazione regionale e applicazioni di terze parti. Il saluto di login
prima che la preferenza dell'utente è disponibile e mantiene il proprio account di servizio
lingua, inglese per impostazione predefinita.
Varianti regionali utilizzano il catalogo installato per la loro lingua quando un esatto
il catalogo è assente; per esempio `es_MX.UTF-8` usi `es-ES`. `zh-Hans` selezioni
Cinese semplificato. Cinese tradizionale (`zh-Hant`♪ `zh-TW`♪ `zh-HK`♪ `zh-MO`) è
non silenziosamente mappato a cinese semplificato e rientra in inglese.

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
