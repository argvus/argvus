---
title: Variabili d'ambiente
description: Contratti ambientali utilizzati dai componenti della sessione ARGVUS.
---
La sessione stabilisce l'integrazione dell'ambiente XDG e Wayland e può esportare variabili di compatibilità con la virtualizzazione quando rilevate. `ARGVUS_CONFIG_HOME`, `ARGVUS_SYSTEM_CONFIG` e `ARGVUS_BOOTSTRAP` sono utilizzati dagli helper della sessione e del dispatcher per risolvere le radici di configurazione.

Tratta le variabili d'ambiente come contratti di runtime: verifica lo `--help` dell'helper installato o la sua fonte prima di aggiungere una sovrascrittura persistente.

Lo stato derivato vive sotto la radice dello stato piuttosto che sotto la radice della configurazione. `XDG_STATE_HOME` (normalmente `~/.local/state`) riceve `argvus/config-projection.json`, il manifest di proiezione `argvus-config` ricostruisce e `argvus sessionctl reload` applica, più `argvus/session.log`. Nessuno dei due è una fonte di verità.

## Ambiente linguistico

L'unica preferenza generale della lingua è `LANG` in `$XDG_CONFIG_HOME/locale.conf` (predefinito `~/.config/locale.conf`). Senza un valore valido, ARGVUS seleziona l'inglese (`en-US`), anche quando `/etc/locale.conf`, PAM o il gestore di accesso forniscono il portoghese. Valori come `pt_BR.UTF-8` si normalizzano in `pt-BR` per i cataloghi.

La sessione applica questa preferenza prima di avviare i componenti del desktop e importare gli ambienti di attivazione di systemd user e D-Bus. Sostituisce `LANG`, `LC_MESSAGES`, `LANGUAGE` ereditati e cancella `LC_ALL`; le impostazioni regionali `LC_*` nello stesso file vengono mantenute. L'inglese usa `en_US.UTF-8` quando viene generato, altrimenti `C.UTF-8`. Nessun file di preferenze viene creato automaticamente. Disconnettersi e accedere nuovamente dopo aver cambiato la lingua.

`session.language`, `argvus/data/language` e `argvus/data/internal/language` sono dati legacy ignorati. Non vengono né importati né rigenerati; i file esistenti sono preservati. `argvus config set` e `patch` rifiutano nuove scritture verso `session.language` e indirizzano i chiamanti verso `locale.conf`.

Per la diagnosi, confrontare `argvus i18n locale`, `cat "${XDG_CONFIG_HOME:-$HOME/.config}/locale.conf"` e `systemctl --user show-environment`. `localectl status` descrive il sistema, non il linguaggio ARGVUS dell'utente. `ARGVUS_I18N_DIR` seleziona una directory del catalogo per lo sviluppo, non il linguaggio.
