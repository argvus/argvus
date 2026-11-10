---
title: Localizzazione
description: Usa ARGVUS in inglese o portoghese.
---
`argvus-i18n` fornisce i cataloghi condivisi, il comando `argvus i18n validate` e il modulo QML `org.argvus.i18n`. ARGVUS attualmente distribuisce i cataloghi in inglese e portoghese.

Le applicazioni native e i componenti della shell utilizzano i cataloghi condivisi. Una stringa tradotta può quindi essere utilizzata da più di un repository.

Usa **Centro di Controllo → Lingua e Regione** per configurare la lingua, la locale regionale, le localizzazioni di sistema generate, il fuso orario, data/ora e tastiera. Gli attuali cataloghi ARGVUS forniscono inglese e portoghese; questo è separato dall'elenco delle localizzazioni di sistema che il sistema operativo ha generato.

Cambiare la lingua di visualizzazione salva la preferenza ARGVUS e riavvia automaticamente `argvus-control-panel.service`, così il Pannello di Controllo riflette immediatamente la modifica. Altre applicazioni e superfici della shell potrebbero richiedere un ricaricamento della sessione o una nuova sessione. Generare o modificare le voci `/etc/locale.gen` richiede autorizzazione ed esegue `locale-gen`; selezionare una localizzazione che non è generata viene rifiutato. Layout della tastiera, variante e mappatura dei tasti della console sono impostazioni separate e non devono essere confusi con la lingua dell'interfaccia.

## Quando le variabili d'ambiente sovrascrivono la modifica

Le applicazioni ARGVUS rilevano la lingua dal file di lingua ARGVUS persistente per prime, poi da `LC_ALL`, `LC_MESSAGES` e `LANG`. Le variabili d'ambiente locali possono quindi far sembrare che il Centro di Controllo ignori una modifica della lingua, specialmente quando un servizio utente o una sessione grafica mantiene un ambiente più vecchio.

Ispeziona i valori effettivi con:

```sh
systemctl --user show-environment | rg '^(LANG|LC_|ARGVUS)='
locale
echo $LANG
localectl status
```
Se `~/.config/locale.conf` esiste e definisce `LANG`, `LC_ALL` o `LC_MESSAGES`, rinominalo temporaneamente in modo che non reintroduca più quei valori. Conserva un backup invece di eliminarlo:

```sh
mv ~/.config/locale.conf ~/.config/locale.conf.backup
```
Applica di nuovo la lingua in **Centro di Controllo → Lingua e Regione**. Il Pannello di Controllo viene riavviato automaticamente; se altri componenti mostrano ancora la lingua precedente, disconnettiti e riconnettiti affinché la sessione grafica e `systemd --user` ricevano il nuovo ambiente. Ripeti i comandi di diagnostica successivamente. Consulta [Variabili d'ambiente](/docs/reference/environment-variables/) per i contratti tecnici e la precedenza.
