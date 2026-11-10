---
title: Servizi
description: Servizi systemd e utente di ARGVUS.
---
L'utente principale target è `argvus-session.target`. I servizi più comuni per l'utente includono `argvus-taskbar.service`, `argvus-control-panel.service`, `argvus-widget-telemetry.service`, `argvus-hyprland.service`, `argvus-dunst.service`, `argvus-wallpaper.service`, `argvus-hypridle.service`, i servizi appunti e `argvus-session-loading.service`.

Il firewall è un servizio di sistema denominato `argvus-firewall.service`. Il pacchetto del calendario installa `argvus-taskbar-calendar.service` come unità utente, e `argvus-clock.service` (disabilitato di default) avvia ARGVUS Clock nascosto dopo il login quando **Avvia con la sessione** è attivo.

Ispeziona le unità attive con:

```sh
systemctl --user status argvus-session.target
systemctl --user list-units 'argvus-*.service'
```
## Unità possedute dalla sessione

Il pacchetto sessione installa queste unità utente sotto `/usr/lib/systemd/user` (il payload sorgente si trova sotto l'albero di configurazione condiviso del pacchetto sessione):

| Unità | Ruolo |
| --- | --- |
| `argvus-session.target` | Confine di durata per la sessione grafica ARGVUS. |
| `argvus-session-prepare.service` | Preparazione di avvio one-shot e configurazione generata. |
| `argvus-control-panel.service` | Pannello di controllo Quickshell. |
| `argvus-taskbar.service` | Barra delle applicazioni principale di ARGVUS. |
| `argvus-widget-telemetry.service` | Superficie opzionale di telemetria/informazioni di sistema. |
| `argvus-hyprland.service` | Unità one-shot su richiesta che esegue `hyprctl reload`. |
| `argvus-config.service` | Proietta la configurazione canonica (`argvus config project`); il reload esegue `argvus sessionctl reload-config-transaction`. |
| `argvus-session-reload.service` | Ricarica login (`argvus sessionctl reload-desktop --login`): ricarica Hyprland solo quando la proiezione lo ha interessato. |
| `argvus-ssh-agent.service` | Agente SSH per la sessione. |
| `argvus-secret-service.service` | Avvia il servizio segreto tramite `argvus sessionctl start-secret-service`. |
| `argvus-wallpaper.service` | Backend wallpaper attivo. |
| `argvus-dunst.service` | Demone di notifica Dunst. |
| `argvus-hypridle.service` | Politica di inattività e blocco. |
| `argvus-clipboard-text.service` / `argvus-clipboard-image.service` | Monitor della cronologia degli appunti. |
| `argvus-keyboard-layout.service` | Notifiche del layout della tastiera. |
| `argvus-polkit.service` | Integrazione dell'agente PolicyKit. |
| `argvus-session-loading.service` | Sovrapposizione di caricamento della sessione dal passaggio del compositor fino all'avvio dei componenti del desktop. |

`argvus-blueman-applet.service` è anche spedito tramite il pacchetto di sessione. Alcune unità sono opzionali o avviate in modo condizionale; non sono tutte attive in ogni sessione.

Per ispezionare la definizione effettiva di un'unità e non solo il suo checkout di origine:

```sh
systemctl --user cat argvus-session.target
systemctl --user status argvus-session.target
journalctl --user -u argvus-session.target -u 'argvus-*.service'
```
Il `argvus-firewall.service` a livello di sistema appartiene a `argvus-firewall`. Il pacchetto calendario invia separatamente `argvus-taskbar-calendar.service` come unità utente. `argvus-clock.service` è un'unità utente del pacchetto `argvus`, `WantedBy=graphical-session.target`, abilitata dalle impostazioni dell'orologio ARGVUS.

## Comandi di gestione della sessione

Il comando `argvus sessionctl` gestisce il ciclo di vita della sessione e può riavviare componenti specifici senza effettuare il logout:

### Ciclo di vita della sessione

```sh
argvus sessionctl start               # Start a new session
argvus sessionctl stop                # Stop the current session
argvus sessionctl restart             # Restart the entire session
argvus sessionctl reload              # Reload configuration and active services
argvus sessionctl status              # Show session status
argvus sessionctl logs                # Display session logs
```
### Riavvii specifici dei componenti

Riavvia i singoli servizi senza influenzare l'intera sessione:

```sh
# Desktop surfaces
argvus sessionctl restart waybar
argvus sessionctl restart wallpaper
argvus sessionctl restart shell

# Notifications and switcher
argvus sessionctl restart dunst

# Clipboard and input
argvus sessionctl restart clipboard keyboard-layout

# Bluetooth applet (optional)
argvus sessionctl restart blueman-applet
```
### Sincronizzazione dell'ambiente

Importa le variabili d'ambiente nella sessione:

```sh
argvus sessionctl import-environment
```
Questo sincronizza le variabili di Wayland, Hyprland, Qt, cursore e DBus tra la shell e i servizi utente di systemd.

## Riferimento alle unità di servizio

Ulteriori unità utente possono essere gestite direttamente con `systemctl --user`:

```sh
systemctl --user status argvus-session.target
systemctl --user start argvus-session.target
systemctl --user stop argvus-session.target
systemctl --user restart <unit>

# View logs for a specific service
journalctl --user -u argvus-wallpaper.service -b -f
```
I registri per il bootstrap della sessione iniziale sono memorizzati in:

```
${XDG_STATE_HOME:-$HOME/.local/state}/argvus/session.log
```
