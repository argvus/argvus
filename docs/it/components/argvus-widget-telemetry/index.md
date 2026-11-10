---
title: Telemetria del widget
description: I blocchi di telemetria di Waybar e il loro interruttore, tramite il widget-telemetry di argvus.
---
Il widget di telemetria è l'informazione di sistema mostrata nella barra delle applicazioni (intestazione, chiavi, sistema, CPU e GPU, memoria, archiviazione, processi, rete e il cruscotto per sviluppatori). È in-process nel pacchetto `argvus`: non ci sono collector della shell.

## Comandi

```text
argvus widget-telemetry block BLOCK ...   prints the text of one block (used by the bar)
argvus widget-telemetry toggle ...        switches the blocks on and off, and their order
```
`block` è il percorso principale della barra: viene chiamato a ogni aggiornamento. `toggle` cambia quali blocchi la barra mostra e sposta la finestra di scorrimento.

I blocchi sono `header`, `keys`, `system`, `cpu_gpu`, `memory`, `storage`, `processes`, `network` e `dev_dashboard`. La barra delle applicazioni utilizza anche le sonde `cpu-temp` e `gpu`. La sonda GPU accetta `--present` (una GPU supportata è visibile in DRM/sysfs), `--available` (una GPU e il comando btop sono disponibili per il pannello), `--available-temp` (si può leggere una temperatura attribuibile), e `--temp` (la temperatura massima valida, o `N/A`).

Le temperature della GPU sono abbinate a un dispositivo PCI della GPU. Le letture proprietarie NVIDIA utilizzano gli ID bus `nvidia-smi`/NVML; le letture NVIDIA nouveau, AMD e Intel utilizzano hwmon solo quando il collegamento del dispositivo sysfs corrisponde univocamente a quella GPU. La temperatura nella barra delle applicazioni è il valore massimo valido tra le GPU rilevate. Letture mancanti, non valide o ambigue restano non disponibili. La temperatura proprietaria NVIDIA richiede il fornitore del driver; la temperatura AMD e Intel dipende dal kernel che espone un sensore hwmon collegato. La temperatura Intel non è fornita dal pannello GPU di btop 1.4.7.

## Avvio con la sessione

L'unità utente `argvus-widget-telemetry.service` si avvia con la sessione solo quando il widget è abilitato. Il master switch è `effects.widget_telemetry_enabled` nella configurazione, che è abilitato di default, quindi un nuovo account avvia il widget al login.

Una volta che l'interruttore ha scritto `$XDG_CACHE_HOME/argvus/waybar/widget-telemetry-state`, quel file decide invece. Solo una prima riga di `enabled` avvia il widget. I vecchi file `desktop-telemetry-state` e `sysinfo-state` vengono ancora letti quando il file corrente è mancante.

Le etichette di telemetria sono compatte in tutte e dieci le lingue supportate. Gli identificatori tecnici come CPU, GPU, IP, DE, WM e Kernel mantengono il loro significato tecnico. Il tempo di attività viene letto da `/proc/uptime` e mostrato con unità abbreviate (per esempio `1d 12h 49m`); non si espande più in parole complete secondo la lingua di sistema. I nomi dei modelli hardware e altri valori del provider possono ancora influenzare la larghezza del widget.
