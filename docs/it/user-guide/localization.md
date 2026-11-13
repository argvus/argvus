---
title: Localizzazione
description: Usa ARGVUS in inglese o portoghese.
---

`argvus-i18n` fornisce i cataloghi condivisi, i `argvus i18n validate` comando e QML modulo `org.argvus.i18n`. ARGVUS navi 18 lingue di interfaccia: arabo, Bengali, tedesco, inglese, spagnolo, francese, hindi, indonesiano, italiano, giapponese, coreano, portoghese brasiliano, portoghese europeo, russo, turco, urdu, vietnamita e cinese semplificato.

Nativo ARGVUS applicazioni e componenti shell consumano i cataloghi condivisi. Una stringa tradotta può quindi essere utilizzata da più di un repository. Arabo e Urdu usano il layout di destra a sinistra nel supportato GTK♪ Quickshell e interfacce terminali.

Utilizzare **Centro di controllo → Preferenze → Lingua & Regione → Lingua** per selezionare il ARGVUS linguaggio di interfaccia. Questo scrive `ARGVUS_LANGUAGE` a `$XDG_CONFIG_HOME/locale.conf` (normalmente) `~/.config/locale.conf`). Per esempio, `ARGVUS_LANGUAGE=pt-PT` seleziona portoghese europeo. Portoghese brasiliano (Italia)`pt-BR`) è un'opzione separata. Il catalogo della lingua è installato con ARGVUS, così non dipende da `locale-gen`.

Il Pannello di controllo si riavvia automaticamente dopo le modifiche della lingua dell'interfaccia. Avviare una nuova sessione per applicarla ai componenti di sessione. L'impostazione separata **Regional Locale** scrive `LANG` e controlla formati regionali e applicazioni di terze parti. Richiede un sistema generato locale; abilitare uno in `/etc/locale.gen` piste `locale-gen` con l'autorizzazione dell'amministratore. Il layout della tastiera, la variante e la tastiera della console sono impostazioni separate.

## Quando le variabili ambientali superano il cambiamento

ARGVUS applicazioni leggere `ARGVUS_LANGUAGE` dall'utente `locale.conf`. Se è assente, risolvono la regionale `LANG` valore a un catalogo installato. Le variabili locali inerite non sovrascrivono un esplicito `ARGVUS_LANGUAGE` preferenza. Le applicazioni di terze parti continuano a utilizzare il sistema locale regionale.

Ispezionare i valori effettivi con:

```sh
systemctl --user show-environment | rg '^(LANG|LC_|ARGVUS)='
locale
printf '%s\n' "$LANG"
localectl status
```

Controlla che `~/.config/locale.conf` contiene una valida `ARGVUS_LANGUAGE=<catalog>` valore. Per modificare la formattazione regionale o la lingua di applicazione di terze parti, utilizzare **Regional Locale** e selezionare un locale generato. Il Pannello di controllo è riavviato dopo le modifiche di preferenza; disconnettersi e se altri componenti di sessione mostrano ancora la vecchia lingua di interfaccia. Vedere [variabili di ambiente](/docs/reference/environment-variables/) per i contratti tecnici e la precedenza.
