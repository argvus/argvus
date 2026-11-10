---
title: Posizioni dei file
description: File e directory ARGVUS installati importanti.
---

- `/usr/bin/argvus*` — comandi pubblici.
- `/usr/share/argvus/hyprland/` — configurazione e strumenti di supporto di Hyprland.
- `/usr/share/argvus/control-panel/` — QML del Pannello di Controllo e script.
- `/usr/share/argvus/taskbar/` — configurazione e script della barra delle applicazioni.
- `/usr/share/argvus/lock/` — configurazione di Hyprlock.
- `/usr/share/argvus/portal/` — file di integrazione del portale.
- `/usr/share/backgrounds/argvus/` — risorse per sfondi.
- `/etc/argvus/` — configurazione del fornitore di sistema.
- `/etc/argvus/greeter.toml` — configurazione del greeter.
- `/etc/argvus/removable-devices/config.json` — impostazioni predefinite di sistema per archivi rimovibili.
- `$XDG_CONFIG_HOME/argvus/config.json` — preferenze portatili canoniche.
- `$XDG_CONFIG_HOME/argvus/data/` — dati dei componenti gestiti, backup, blocchi e output generato.
- `$XDG_CONFIG_HOME/waybar/argvus-taskbar.{jsonc,css}` — sostituzioni opzionali complete della barra delle applicazioni native Waybar.
- `$XDG_CONFIG_HOME/argvus/data/taskbar/argvus-taskbar.{jsonc,css}` — gestite copie della barra delle applicazioni ARGVUS. 
- `$XDG_CONFIG_HOME/argvus/data/generated/waybar/` — profili Waybar generati, inclusi `argvus-taskbar.css` e `argvus-widget-telemetry.{jsonc,css}`; questi hanno priorità rispetto alle copie sopra indicate. 
- `$XDG_CONFIG_HOME/argvus/data/generated/waybar/argvus-widget-telemetry.{jsonc,css}` — anche duplicato sotto `data/waybar/`, che viene sostituito con il default incluso nel pacchetto in caso di modifica dell'aspetto e successivamente ha i blocchi `ARGVUS_TELEMETRY_*` e dei font riscritti. 
- `$XDG_CONFIG_HOME/argvus/data/generated/yazi/` — albero di configurazione Yazi proiettato, con un fallback consapevole delle varianti per i temi che non includono una versione Yazi. 
- `$XDG_CONFIG_HOME/argvus/data/generated/superfile/` — set di temi Superfile proiettato. 
- `$XDG_CONFIG_HOME/argvus/data/generated/terminal/` — profili terminale proiettati.
- `$XDG_CONFIG_HOME/argvus/data/generated/qt6ct/` — palette Qt6ct prevista.  
- `$XDG_CONFIG_HOME/argvus/data/generated/gtk/` — file del tema GTK previsti.  
- `$XDG_CONFIG_HOME/argvus/data/rofi/` — copie utente della configurazione di Rofi, dei temi e della modalità. ARGVUS crea ogni copia quando modifica per la prima volta il file; i file non modificati vengono letti da `/usr/share/argvus/launcher/config/`.  
- `$XDG_CONFIG_HOME/argvus/data/generated/removable-devices/theme.css` — foglio di stile per dispositivi rimovibili previsto.  
- `$XDG_CONFIG_HOME/argvus/data/focus/settings.json`, `$XDG_DATA_HOME/argvus/focus/history.json`, `$XDG_STATE_HOME/argvus/focus/session.json` — impostazioni di ARGVUS Focus, cronologia della sessione e checkpoint della sessione corrente.  
- `$XDG_RUNTIME_DIR/argvus/focus/status.json` — stato effimero di ARGVUS Focus utilizzato dalla barra delle applicazioni; rimosso quando Focus termina.
- `$XDG_CONFIG_HOME/argvus/data/clock/settings.json`, `$XDG_DATA_HOME/argvus/clock/alarms.json`, `$XDG_STATE_HOME/argvus/clock/timers.json` — Impostazioni dell'orologio ARGVUS e orologi mondiali, sveglie e punto di controllo timer/cronometro. 
- `$XDG_CONFIG_HOME/argvus/data/generated/theme/current/time/theme.css` — colori di ARGVUS Focus e ARGVUS Clock (consumatore del tema `time`). 
- `/usr/share/argvus/sounds/` — suoni di ARGVUS Focus e ARGVUS Clock (`argvus-chime.oga`, `argvus-beacon.oga`, `argvus-bell.oga`). 
- `$XDG_CONFIG_HOME/argvus/data/generated/theme/` — build dei temi (`builds/<id>-<variant>-<hash>/`), scritte da `argvus theme render` e da ogni cambio di tema e accento, e il collegamento simbolico `current` alla build del tema attivo. 
- `$XDG_CONFIG_HOME/argvus/data/hypr/` — override Lua di Hyprland dell'utente e proiezioni native di Hyprland.
- `$XDG_CONFIG_HOME/hypr/hyprland.lua` — configurazione nativa completa opzionale di Hyprland selezionata da `argvus start`; non è un overlay incrementale.  
- `$XDG_STATE_HOME/argvus/config-projection.json` — manifesto di proiezione: generazione, hash della configurazione effettiva, sezioni modificate e lavori runtime in sospeso. Letto da `argvus sessionctl reload`; mai una fonte di verità.  
- `$XDG_STATE_HOME/argvus/widget-telemetry-blocks` — preferenze del blocco di telemetria legacy. Solo input per la migrazione; i valori canonici risiedono in `config.json` e questo file non viene più scritto.  
- `$XDG_STATE_HOME/argvus/session.log` — registro della sessione grafica (normalmente `~/.local/state/argvus/session.log`).  
- `/usr/share/wayland-sessions/argvus.desktop` — ingresso della sessione grafica.  
- `/usr/share/plymouth/themes/argvus/` — risorse del boot splash confezionate.  

I payload esatti sono specifici del pacchetto; vedere il riferimento dei componenti per sviluppatori e il manifesto del pacchetto pertinente.
