---
title: Calendario
description: Usa il calendario della barra delle applicazioni ARGVUS.
---
`argvus-taskbar-calendar` è il popup del calendario GTK4 aperto dal modulo data di Waybar o con:

```sh
argvus --calendar
argvus taskbar-calendar toggle
argvus taskbar-calendar status
```
Supporta eventi locali, promemoria e import/export ICS. Il suo servizio utente è `argvus-taskbar-calendar.service`. Usa `argvus taskbar-calendar --help` per le opzioni supportate dalla versione installata.

Il calendario segue il tema ARGVUS attivo, comprese le varianti Sticky e Spaced. Le modifiche del tema sono applicate tramite la cache utente generata; usa `argvus taskbar-calendar reload` se un popup in esecuzione necessita di un aggiornamento esplicito.

Quando viene aperto dal modulo data della barra delle applicazioni, il popup è ancorato al bordo della barra: si apre sotto una barra delle applicazioni superiore e sopra una barra delle applicazioni inferiore, allineato al bordo destro del modulo data. Le aperture dirette senza geometria della barra delle applicazioni utilizzano il puntatore o il fallback del monitor.

## Comandi

```bash
argvus taskbar-calendar                 open the calendar
argvus taskbar-calendar toggle          show or hide it (the Waybar date module uses this)
argvus taskbar-calendar show | hide
argvus taskbar-calendar config          open the system configuration for editing (sudo)
argvus taskbar-calendar config-path     print the configuration path
argvus taskbar-calendar reload
argvus taskbar-calendar import file.ics
argvus taskbar-calendar export --output calendar.ics
argvus taskbar-calendar sync
argvus taskbar-calendar status
argvus taskbar-calendar service
```
## Eventi e promemoria

Gli eventi sono disattivati di default. Attivali nel popup sotto **Impostazioni > Eventi**; la scelta viene salvata nella directory di stato e abilita e avvia anche `argvus-taskbar-calendar.service`, che mostra i promemoria. Disattivarli interrompe il servizio.

- **+ EVENTO** (o `n`) apre l'editor per il giorno selezionato. Gli eventi possono essere creati e modificati a partire da oggi; un evento di un giorno passato può comunque essere aperto ed eliminato.
- Un nuovo evento inizia alle 09:00 e dura la lunghezza predefinita impostata in Impostazioni. Spostare l'orario di inizio mantiene la durata; un orario di fine precedente all'orario di inizio significa che l'evento termina il giorno successivo.
- Ogni evento mostra una notifica con un suono all'inizio, indipendentemente da cosa indicano le caselle **Promemoria** e **Ripeti**. Un evento salvato dopo il suo inizio (ad esempio un evento di tutta la giornata creato durante il giorno) notifica entro 30 secondi.
- **Promemoria** aggiunge una notifica anticipata, all'orario scelto prima dell'inizio (all'inizio quando quell'orario è già passato).
- Per un evento di tutto il giorno il campo orario diventa **NOTIFICA AL**: l'evento copre l'intera giornata e notifica a quell'orario (00:00 notifica all'inizio della giornata). L'orario viene mantenuto quando l'evento viene riaperto.
- Per gli eventi di tutto il giorno, **Ripeti** notifica di nuovo all'intervallo scelto dopo quell'orario.
- Modificare un evento senza spostarne l'inizio non ripete i promemoria già mostrati.
- Le notifiche seguono la lingua del calendario (Impostazioni > Lingua).
- Un giorno mostra un punto (`•`) quando un evento si svolge in quella data, inclusi tutti i giorni di un evento notturno o di più giorni. I giorni seguono il fuso orario locale.
- Gli eventi locali vengono rimossi automaticamente 10 minuti dopo la loro conclusione. Se un evento è già concluso quando viene salvato, rimane disponibile per almeno 10 minuti dopo il salvataggio.

I file importati `.ics` conservano le loro date, fasce orarie (`TZID`) e sveglie (`VALARM`). Gli eventi importati notificano anche al loro inizio, oltre alle proprie sveglie. Gli eventi ricorrenti (`RRULE`) vengono memorizzati ed esportati, ma viene mostrata e ricordata solo la loro prima occorrenza. Dopo un aggiornamento del pacchetto, il servizio di promemoria termina e systemd avvia la nuova versione entro 30 secondi. `sync` riporta solamente i calendari remoti configurati; la sincronizzazione CalDAV non è ancora implementata.

## Percorsi

- Configurazione: `/etc/argvus/taskbar/calendar/config.toml` (a livello di sistema; un valore predefinito è incluso con il pacchetto).
- Stili: `/etc/argvus/taskbar/calendar/style.css` e `theme.css`; i colori del tema ARGVUS attivo provengono dal tema costruito attraverso la cache sottostante.
- Dati: `$XDG_DATA_HOME/argvus-taskbar-calendar/` (o `~/.local/share/argvus-taskbar-calendar/`).
- Stato: `$XDG_STATE_HOME/argvus/taskbar/calendar/` (o `~/.local/state/...`); lì viene mantenuto il toggle degli eventi.
- Cache: `$XDG_CACHE_HOME/argvus-taskbar-calendar/` (o `~/.cache/argvus-taskbar-calendar/`).

Il colore di evidenziazione viene letto da `~/.config/argvus/data/.accent-color` e applicato in fase di esecuzione
sopra ai colori del tema.

La selezione automatica della lingua segue l'utente `locale.conf`, con l'inglese come predefinito. Le preferenze esplicite della lingua del calendario continuano a prevalere sulla selezione automatica.

La barra delle applicazioni del calendario, ARGVUS Focus, ARGVUS Clock e il Pannello di controllo sono mutuamente esclusivi: l'apertura di uno chiude gli altri.

I fallimenti di lettura del calendario vengono visualizzati separatamente da un'agenda vuota. L'apertura del database, il caricamento degli eventi giornalieri e il caricamento degli indicatori mensili hanno messaggi distinti. Seleziona nuovamente un giorno per riprovare. Un aggiornamento non riuscito mantiene i dati precedentemente caricati solo per lo stesso giorno o mese; i risultati validi dell'altra query rimangono disponibili.

## Cataloghi linguistici condivisi

L'interfaccia utilizza i cataloghi condivisi `argvus-i18n` per tutte e dieci le lingue. I testi sono memorizzati in `resources/locales/<locale>/taskbar-calendar.json` nell'area di lavoro principale e installati da `argvus`. L'inglese è il fallback; le nuove lingue non richiedono più un dizionario bilingue locale.
