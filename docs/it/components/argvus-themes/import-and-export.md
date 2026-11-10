---
title: Importazione ed esportazione di temi
description: Esporta il tema attivo come tema personale, importa un archivio del tema, applicalo e cancellalo.
---
Esportazione e importazione funzionano con i **temi**, nello stesso formato dei temi ufficiali. L'esportazione salva il tema attivo, con il colore di evidenziazione e lo sfondo, come un nuovo tema personale. L'importazione installa un archivio di tema tra i tuoi temi. Entrambi si trovano in **Centro di Controllo → Aspetto → Temi**, e i tuoi temi sono elencati in **Temi Personalizzati**.

## Cosa contiene un tema esportato

L'archivio, `argvus-theme-<id>.tar.gz`, contiene una directory del tema, strutturata come un tema ufficiale:

```text
<id>/theme.toml        the manifest (schema 2): name, category, palette, wallpaper
<id>/appearance.json  your visual settings, including layout, effects, fonts and panel presentation
<id>/effects.conf     per-surface effect values for the active theme, when present
<id>/overrides/...     the overrides of the theme it was made from, if any
<id>/wallpaper.<ext>   the wallpaper on screen when you exported
```
Il manifesto si basa sul tema attivo, con il tuo nome, un nuovo ID, il tuo colore di evidenziazione come accento della palette e lo sfondo accanto ad esso. Il profilo di aspetto conserva le impostazioni visive attualmente applicate in ARGVUS, inclusi spaziatura delle finestre, bordi, margini della barra delle applicazioni, presentazione del pannello, colori, effetti e caratteri. Lo sfondo personalizzato e il colore di accento sono memorizzati con il tema. Applicare qualsiasi tema riapplica il suo profilo completo (o i valori predefiniti di ARGVUS quando non esiste un profilo), sostituendo le modifiche visive indipendenti effettuate dall’ultima applicazione. Passare a Sticky o Spaced riapplica sempre la geometria delle finestre e della barra delle applicazioni di quel modo; salva la geometria personale in un tema personalizzato esportato per mantenerla disponibile quando quel tema viene applicato. Le impostazioni non visive come audio, tastiera e preferenze di rete non fanno parte di un tema.

L'aspetto applicato persiste attraverso i riavvii e i ricaricamenti della sessione. La preparazione della sessione ricostruisce i file necessari dalla configurazione attuale senza ripristinare la geometria del tema originale.

Le impostazioni modificate dopo l'applicazione di un tema sono temporanee durante il successivo cambiamento di tema o modalità. Esporta il tema da **Centro di Controllo → Aspetto → Temi → Esporta tema** per preservare la configurazione visiva attuale e ripristinarla in seguito applicando quel tema esportato.

I tuoi temi sono installati in `~/.local/share/argvus/themes/<id>/`. Estrarre un archivio lì installa anche il tema.

## Esporta il tema attivo

1. Apri **Centro di Controllo → Aspetto → Temi**.
2. Scegli **Esporta tema**, oppure premi `e` su una pagina Temi.
3. Digita un nome per il tema. Il campo mostra il file che sarà scritto.
4. Premi `Enter`.

L'id è creato dal nome: lettere minuscole, cifre e trattini. Quando l'id è già utilizzato da un tema installato o da un archivio nella tua directory home, vengono aggiunti `-2`, `-3` e così via. L'archivio viene scritto nella tua directory home, ad esempio `~/argvus-theme-my-theme.tar.gz`. ARGVUS controlla il tema prima di scriverlo: il manifesto deve caricarsi e ogni componente deve essere renderizzato in entrambe le modalità.

## Importare un tema

1. Copia l'archivio `.tar.gz` nella tua directory home (`~`). ARGVUS elenca gli archivi presenti lì che contengono un tema.
2. Apri **Centro di controllo → Aspetto → Temi → Importa tema**, oppure premi `i` in una pagina Temi.
3. Seleziona l'archivio e premi `Enter`.

ARGVUS controlla l'archivio prima di installarlo. Deve contenere una singola directory con un `theme.toml`, solo con percorsi relativi (nessun `..`, nessun link), al massimo 128 voci e un limite di dimensione per file; l'id deve corrispondere alla directory, lo sfondo deve essere nell'archivio e ogni componente deve funzionare in entrambe le modalità. Un profilo di aspetto incluso deve usare sezioni supportate e JSON valido. Se un controllo fallisce, nulla viene installato.

Il tema viene aggiunto a **Temi Personalizzati**. L'importazione non lo applica: scegli tu quando farlo.

### Collisioni di nomi

- Se uno dei tuoi temi ha lo stesso id, ARGVUS chiede **Sostituire il tuo tema con lo stesso ID?**. Scegli **Sostituisci** per sostituirlo, oppure premi `Esc` per mantenere il tuo. Il tema sostituito viene conservato come backup fino a quando il nuovo non è installato.
- Un archivio con l'id di un tema ufficiale viene rifiutato. Esportalo nuovamente con un altro nome.

## Applica un tema personalizzato

1. Apri **Centro di Controllo → Aspetto → Temi → Temi Personalizzati**.
2. Seleziona il tema e premi `Enter`.

Viene applicata la configurazione completa esportata del tema, inclusi sfondo, spaziatura, bordi, colore d'accento, effetti, font e presentazione del pannello. I valori visivi sostituiscono le impostazioni correnti. ARGVUS quindi riavvia la barra delle applicazioni, il servizio di ricarica di Hyprland, lo sfondo e il Pannello di Controllo affinché ogni superficie utilizzi il tema personalizzato.

## Eliminare un tema personalizzato

1. Apri **Centro di Controllo → Aspetto → Temi → Temi Personalizzati**.
2. Seleziona il tema e premi `d`.
3. Conferma con `y`, oppure premi `n` o `Esc` per annullare.

La directory del tema viene rimossa da `~/.local/share/argvus/themes`. Se era il tema attivo, ARGVUS passa prima a **ARGVUS Dark**.

## Profili delle versioni precedenti

Le versioni precedenti esportavano *profili dei temi* (`profile.tar.gz`), una copia delle impostazioni di aspetto. La prima volta che si apre una pagina dei Temi, ogni profilo in `~/.local/share/argvus/themes/<name>/` viene convertito in un tema creato a partire dal suo tema di base, con il colore di evidenziazione e lo sfondo. Il profilo originale e la vecchia lista `custom-themes.json` vengono spostati in `~/.config/argvus/data/backups/theme-profiles-<seconds>/`, e la barra di stato segnala la conversione. Un profilo il cui tema di base non è installato viene solo spostato nella copia di backup.

## Condividere un tema

Un tema esportato è un archivio normale. Copialo su un altro computer che utilizza ARGVUS e importalo lì.

## Risoluzione dei problemi

- **L'archivio non è elencato.** Deve trovarsi direttamente nella tua directory principale, terminare con `.tar.gz` e contenere `<id>/theme.toml`.
- **L'importazione è rifiutata.** La riga di stato indica il motivo: un tema ufficiale con lo stesso ID, uno sfondo mancante, un manifesto che non si carica o un componente che non viene renderizzato.
- **L'esportazione è rifiutata.** Il nome deve contenere almeno una lettera o un numero e il tema attivo deve essere installato.
