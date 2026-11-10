---
title: Services
description: Services systemd et utilisateurs ARGVUS.
---
La cible principale de l'utilisateur est `argvus-session.target`. Les services utilisateur courants incluent `argvus-taskbar.service`, `argvus-control-panel.service`, `argvus-widget-telemetry.service`, `argvus-hyprland.service`, `argvus-dunst.service`, `argvus-wallpaper.service`, `argvus-hypridle.service`, les services de presse-papiers et `argvus-session-loading.service`.

Le pare-feu est un service système nommé `argvus-firewall.service`. Le paquet calendrier installe `argvus-taskbar-calendar.service` en tant qu'unité utilisateur, et `argvus-clock.service` (désactivé par défaut) démarre ARGVUS Clock en mode caché après la connexion lorsque **Démarrer avec la session** est activé.

Inspectez les unités actives avec :

```sh
systemctl --user status argvus-session.target
systemctl --user list-units 'argvus-*.service'
```
## Unités appartenant à la session

Le package de session installe ces unités utilisateur sous `/usr/lib/systemd/user` (la charge utile source se trouve sous l'arborescence de configuration partagée du package de session) :

| Unité | Rôle |
| --- | --- |
| `argvus-session.target` | Limite de durée de vie pour la session graphique ARGVUS. |
| `argvus-session-prepare.service` | Préparation au démarrage en une seule fois et configuration générée. |
| `argvus-control-panel.service` | Panneau de contrôle Quickshell. |
| `argvus-taskbar.service` | Barre des tâches principale ARGVUS. |
| `argvus-widget-telemetry.service` | Surface optionnelle de télémétrie/informations système. |
| `argvus-hyprland.service` | Unité ponctuelle à la demande qui exécute `hyprctl reload`. |
| `argvus-config.service` | Projette la configuration canonique (`argvus config project`); le rechargement exécute `argvus sessionctl reload-config-transaction`. |
| `argvus-session-reload.service` | Rechargement de connexion (`argvus sessionctl reload-desktop --login`) : recharge Hyprland uniquement lorsque la projection l'a affecté. |
| `argvus-ssh-agent.service` | Agent SSH pour la session. |
| `argvus-secret-service.service` | Démarre le service secret via `argvus sessionctl start-secret-service`. |
| `argvus-wallpaper.service` | Backend de fond d'écran actif. |
| `argvus-dunst.service` | Démon de notifications Dunst. |
| `argvus-hypridle.service` | Politique d'inactivité et de verrouillage. |
| `argvus-clipboard-text.service` / `argvus-clipboard-image.service` | Observateurs de l'historique du presse-papiers. |
| `argvus-keyboard-layout.service` | Notifications de disposition du clavier. |
| `argvus-polkit.service` | Intégration de l'agent PolicyKit. |
| `argvus-session-loading.service` | Superposition de chargement de session depuis le transfert du compositeur jusqu'au démarrage des composants du bureau. |

`argvus-blueman-applet.service` est également expédié par le package de session. Certaines unités sont optionnelles ou démarrées de manière conditionnelle ; elles ne sont pas toutes actives à chaque session.

Pour inspecter la définition effective d'une unité plutôt que seulement son code source :

```sh
systemctl --user cat argvus-session.target
systemctl --user status argvus-session.target
journalctl --user -u argvus-session.target -u 'argvus-*.service'
```
Le `argvus-firewall.service` au niveau du système appartient à `argvus-firewall`. Le package de calendrier expédie séparément `argvus-taskbar-calendar.service` en tant qu'unité utilisateur. `argvus-clock.service` est une unité utilisateur du package `argvus`, `WantedBy=graphical-session.target`, activée à partir des paramètres de l'horloge ARGVUS.

## Commandes de gestion de session

La commande `argvus sessionctl` gère le cycle de vie de la session et peut redémarrer des composants spécifiques sans se déconnecter :

### Cycle de vie de la session

```sh
argvus sessionctl start               # Start a new session
argvus sessionctl stop                # Stop the current session
argvus sessionctl restart             # Restart the entire session
argvus sessionctl reload              # Reload configuration and active services
argvus sessionctl status              # Show session status
argvus sessionctl logs                # Display session logs
```
### Redémarrages spécifiques aux composants

Redémarrez les services individuels sans affecter l'ensemble de la session :

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
### Synchronisation de l'environnement

Importer les variables d'environnement dans la session :

```sh
argvus sessionctl import-environment
```
Cela synchronise les variables Wayland, Hyprland, Qt, curseur et DBus entre le shell et les services utilisateur systemd.

## Référence des unités de service

Des unités utilisateur supplémentaires peuvent être gérées directement avec `systemctl --user` :

```sh
systemctl --user status argvus-session.target
systemctl --user start argvus-session.target
systemctl --user stop argvus-session.target
systemctl --user restart <unit>

# View logs for a specific service
journalctl --user -u argvus-wallpaper.service -b -f
```
Les journaux pour le démarrage initial de la session sont stockés à :

```
${XDG_STATE_HOME:-$HOME/.local/state}/argvus/session.log
```
