---
title: Conti
description: Profili utente locali, nomi visualizzati, avatar e appartenenza ai gruppi, tramite account argvus.
---
`argvus accounts` elenca e modifica gli account utente locali del desktop ARGVUS. È un eseguibile privato del pacchetto `argvus`, installato in `/usr/lib/argvus/commands/accounts`. Il comando `argvus` lo avvia, e solo le operazioni che modificano altri account o dati di sistema vengono eseguite come root.

## Comandi

```text
argvus accounts list [--all]                   list users (--all includes system accounts)
argvus accounts show USER                      show account details
argvus accounts name USER "DISPLAY NAME"       change another user's display name
argvus accounts avatar USER IMAGE              set another user's avatar
argvus accounts avatar USER --remove           remove another user's avatar
argvus accounts groups USER                    list groups
argvus accounts groups USER --add G --remove G change groups (authorization required)
argvus accounts passwd USER                    change a password (reads three lines from stdin)
argvus accounts self                           show your own account
argvus accounts self name "DISPLAY NAME"       change your own display name
argvus accounts self avatar IMAGE              set your avatar
argvus accounts self avatar --remove           remove your avatar
argvus accounts self groups                    list your groups
argvus accounts --verbose ...                  print diagnostic information to stderr
```
## Nomi visualizzati e avatar

Il nome visualizzato è memorizzato nel campo GECOS standard. Il nome utente Unix non viene mai cambiato.

Un avatar è un'immagine in formato PNG, JPEG o WebP, fino a 20 MiB. L'immagine viene ridimensionata a 256×256 pixel e salvata come PNG con modalità `0644` in `~/.face`, la posizione freedesktop letta dal greeter e dalla maggior parte dei display manager. Il tuo avatar personale viene scritto all'interno della tua home directory e non richiede autorizzazione.

## Autorizzazione

Solo `list`, `show`, `self`, `self groups` e il tuo avatar personale funzionano senza autorizzazione. Ogni altra modifica richiede l'autorizzazione dell'agente polkit del desktop. La regola ARGVUS (`/usr/share/polkit-1/rules.d/50-argvus-accounts.rules`) decide chi deve autenticarsi:

- un utente in un gruppo amministratore (`wheel`, il default di Arch, o `sudo`) conferma con la propria password;
- qualsiasi altro utente conferma con la password di root.

| Operazione | Senza autorizzazione | Con autorizzazione |
| --- | --- | --- |
| Elenca utenti, mostra un account, elenca gruppi | ✓ | ✓ |
| Modifica il proprio avatar | ✓ | ✓ |
| Modifica il proprio nome visualizzato | — | ✓ |
| Cambia la propria password | — | ✓ (dopo aver verificato la password corrente) |
| Cambia il nome, l'avatar, i gruppi o la password di un altro utente | — | ✓ |

La modifica della propria password verifica prima la password corrente. Una password errata viene rifiutata prima che appaia la richiesta di autorizzazione.

Non è necessario eseguire il comando con `sudo`. Il comando richiede l'autorizzazione a polkit solo quando è necessario.

## Limiti di sicurezza

- `passwd` legge la password corrente, quella nuova e la conferma dalla standard input, una per riga, e non le prende mai come argomenti. La riga della password corrente può essere vuota quando un amministratore reimposta la password di un altro utente. Esempio:

  ```sh
  printf '%s\n%s\n%s\n' "$OLD" "$NEW" "$NEW" | argvus accounts passwd ghost
  ```

  Interattivamente, digita le tre righe e premi Invio dopo ciascuna. L'input viene mostrato sul
  terminale mentre lo digiti.
- La modifica di un account con UID inferiore a 1000 (e `nobody`) è rifiutata, così come il blocco, la cancellazione
  e la scadenza della sua password. Il tuo account non può essere bloccato o cancellato.
- Rinominare o eliminare un gruppo con GID inferiore a 1000, o un gruppo amministratore (`wheel`, `sudo`), è
  rifiutato; i membri possono comunque essere modificati, tranne che tu non puoi rimuovere te stesso da un
  gruppo amministratore.
- L'appartenenza da amministratore è il gruppo `wheel` o `sudo`. Il Centro di Controllo aggiunge nuovi
  amministratori a `wheel` quando il sistema lo possiede, e a `sudo` altrimenti. La rimozione della propria
  appartenenza da amministratore tramite una modifica è rifiutata.

## Esempi

```sh
# Set your display name and avatar after the first login
argvus accounts self name "William Canin"
argvus accounts self avatar ~/Pictures/profile.png

# Check your groups, then add another user to the wheel group (authorization required)
argvus accounts self groups
argvus accounts groups ghost --add wheel

# Administrator: show a user's details and reset that user's name
argvus accounts show ghost
argvus accounts name ghost "Ghost Account"
```
## Risoluzione dei problemi

- `pkexec was not found`: installare `polkit`. Il pacchetto `argvus` dipende da esso.
- Il prompt di autorizzazione non appare: verificare che un agente di autenticazione polkit sia in esecuzione nella vostra sessione. Il pacchetto `argvus` dipende da `hyprpolkitagent`, che ne fornisce uno.
- Errori `unix_chkpwd` durante il cambio della password: il pacchetto `pam` fornisce l'assistente.
- Eseguire `argvus accounts --verbose ...` per stampare i passaggi del comando su stderr.

Per note per gli sviluppatori sul confine dei privilegi, vedere `docs/internal/argvus-accounts/`.

Quando un amministratore blocca o rimuove l'account configurato per il login automatico, il fornitore dell'account disabilita prima la sua voce greetd per il login automatico. Se questa pulizia fallisce, l'operazione sull'account viene rifiutata. Se in seguito il blocco o la rimozione falliscono, il login automatico rimane disabilitato. Il login automatico per un altro account viene preservato; sbloccare un account non abilita il login automatico. Le sessioni esistenti non vengono chiuse e greetd non viene riavviato.
