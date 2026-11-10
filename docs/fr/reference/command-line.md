---
title: Ligne de commande
description: La commande argvus, ses sous-commandes, les indicateurs conservés pour les raccourcis clavier, et où chacun s'exécute.
---
`/usr/bin/argvus` est le point d'entrée public d'ARGVUS. C'est un programme Rust construit à partir du crate `argvus-cli`. Chaque sous-commande s'exécute soit à l'intérieur de ce programme, soit exécute un exécutable d'assistance depuis le répertoire commands, soit exécute un script packagé.

Exécutez `argvus --help` pour la liste installée avec votre version. Les sous-commandes sont écrites sans le préfixe `--`. Les formes `--` sont conservées uniquement lorsqu'un raccourci clavier ou une entrée de bureau les utilise.

## Où chaque commande s'exécute

- **Dans le processus :** la commande est une bibliothèque de l'espace de travail et s'exécute à l'intérieur de `argvus`. Aucun exécutable séparé n'existe. Ce sont `config`, `theme`, `i18n`, `keys`, `gpg`, `projects`, `terminal`, `system-monitor`, `network`, `bluetooth`, `focus status`, `widget-telemetry`, `tui-terminal`, `sensors` et `setup`.
- **Helper :** un exécutable distinct dans `/usr/lib/argvus/commands/`, installé par le package `argvus`. Il s'agit de `control-center` (application de terminal), `argvus-du` (lancé avec `argvus du`) et des applications GTK 4 `taskbar-calendar`, `removable-devices`, `focus` et `clock`.
- **Exécutable privé :** un exécutable qu'un autre package installe dans `/usr/lib/argvus/commands/`. `argvus` le démarre uniquement. Il s'agit notamment de `session`, `start`, `tty`, `sessionctl`, `launcher`, `snippets`, `ssh`, `git`, `app-profiles-yazi-keymap`, `notifications`, `displayctl`, `firewall`, `accounts`, `taskbar-actions` et les points d'entrée `greeter-*`.
- **Script :** un script shell packagé exécuté avec `/bin/sh`. Ce sont les commandes du script Hyprland répertoriées ci-dessous.

`ARGVUS_COMMANDS_DIR` modifie le répertoire des commandes, pour les tests et les vérifications `DESTDIR`. Un exécutable privé qui n'y est pas trouvé est recherché sur `PATH`.

## Sous-commandes

| Commande | Fonctionne | Remarques |
| --- | --- | --- |
| `argvus config ...` | En cours | Modèle de configuration et outils. Atteint également par `theme-switch` et `effects-toggle`. |
| `argvus theme ...` | En cours | `apply ID` est le commutateur de thème (le même que `theme-switch`). Constructions de thèmes : `list` les thèmes installés, `check DIRECTORY` un thème par rapport aux modèles installés, `render ID [--variant sticky\|spaced] [--accent #RRGGBB] [--output DIRECTORY]`. Voir [Apparence et thèmes](/docs/developer-guide/subsystems/appearance-and-themes/). |
| `argvus i18n ...` | En cours | Traductions : `get DOMAIN KEY [name=value...]`, `locale`, `list`, `validate [LOCALE]`. |
| `argvus keys ...` | En cours | Gestionnaire de clés SSH. `gpg` est un alias pour les sous-commandes GPG. |
| `argvus gpg ...` | En cours | Clés GPG : `list`, `status`, `generate`, `export-public`, `export-private`, `import`, `import-private`, `delete`, `edit`, `keyserver`. |
| `argvus projects ...` | En cours | Sélecteur de projet. |
| `argvus terminal ...` | En cours | Le profil de terminal ARGVUS. |
| `argvus system-monitor ...` | En cours | Le moniteur système btop ; `--gpu` sélectionne uniquement les boîtiers GPU, et `--panel [--cpu|--mem|--gpu]` l'intègre dans le shell TUI ARGVUS. |
| `argvus du` | Assistant (application terminal) | Ouvre l'analyseur d'utilisation du disque ARGVUS dans une fenêtre Kitty dédiée et exécute ncdu. |
| `argvus network ...` | En cours | Statut et contrôles de NetworkManager (`networkctl` est conservé comme alias). |
| `argvus bluetooth ...` | En cours | Statut et contrôles Bluetooth (`bluetoothctl` est conservé comme alias). |
| `argvus widget-telemetry block ...` | En cours | Le bloc de télémétrie de Waybar. C'est le chemin principal de la barre. |
| `argvus widget-telemetry toggle ...` | En cours | Active ou désactive les blocs de télémétrie, et leur ordre. |
| `argvus game-snake` | Exécutable privé | Le jeu Snake pour le terminal, du projet indépendant `argvus-game-snake`, un package optionnel. En dehors d'un terminal, il en ouvre un. |
| `argvus control-center [page]` | Assistant (application terminal) | Ouvre le Centre de Contrôle, ou met le focus sur la fenêtre déjà ouverte. |
| `argvus welcome [--if-needed]` | Dispatcheur → Centre de Contrôle | Ouvre la page d'accueil intégrée dans le Centre de Contrôle. `--if-needed` s'ouvre une fois, sauf si les enregistrements de progression par utilisateur indiquent que l'accueil a déjà été affiché, complété ou rejeté. |
| `argvus taskbar-calendar [args]` | Application GTK 4 | Par défaut sur `toggle`. |
| `argvus removable-devices [args]` | Application GTK 4 | Par défaut sur `menu`. |
| `argvus focus [--toggle] [--start\|--pause] [--x X --y Y] [--bar-bottom PX --bar-right PX]` | Application GTK 4 | Popup de focus ARGVUS. Une instance par session. |
| `argvus focus status --waybar [--follow]` | En cours de traitement | Émet l'indicateur de barre des tâches Focus en tant que JSON Waybar ; `--follow` diffuse les changements sans charger GTK. |
| `argvus clock [--toggle] [--background] [--page PAGE] [--x X --y Y] [--bar-bottom PX --bar-right PX]` | Application GTK 4 | Popup de l'horloge ARGVUS. Une instance par session ; `--background` maintient les alarmes actives sans popup. |
| `argvus loading-theme ...` | `splash` emballé à `/usr/lib/argvus/loading-theme/splash` | La superposition des rechargements directs de session ; un changement de thème ne l'utilise pas. |
| `argvus session`, `start`, `tty`, `sessionctl` | Exécutables privés | Points d'entrée de session et contrôleur de session. |
| `argvus greeter`, `greeter-session`, `greeter-tui`, `greeter-setup` | Exécutables privés | Points d'entrée du greeter. |
| `argvus accounts`, `firewall` | Exécutables privés | Outils de comptes et de pare-feu. |
| `argvus launcher`, `snippets`, `ssh`, `git` | Exécutables privés | Outils de lancement et de sélection. |
| `argvus tui-terminal` | En processus | Enveloppe de terminal pour applications TUI. |
| `argvus sensors` | En processus | Aperçu en direct de la température depuis `sensors -j`. |
| `app-profiles-yazi-keymap` | Exécutable privé | Assistant de mappage des touches. |
| `argvus notifications`, `displayctl`, `taskbar-actions` | Exécutables privés | Notifications, affichage et actions sur la barre des tâches. |
| `argvus setup [options]` | En processus | Copie les valeurs par défaut packagées dans la configuration utilisateur. Voir ci-dessous. |

Les arguments sont transmis tels quels, y compris les espaces et les valeurs vides. L'entrée standard, la sortie, l'erreur, les signaux et le statut de sortie appartiennent à la commande qui s'exécute. Un nom inconnu échoue avec un indice pour exécuter `argvus --help`. Une commande qui s'exécute en processus refuse un argument qui n'est pas en UTF-8 valide (statut de sortie 2) au lieu de transmettre une valeur altérée ; les commandes qui exécutent un programme séparé reçoivent les octets inchangés.

## Options et alias

Ces noms existent pour que les raccourcis clavier, les entrées de bureau et les scripts plus anciens continuent de fonctionner. Préférez la forme de sous-commande dans le nouveau code.

| Option ou alias | Équivalent |
| --- | --- |
| `--control-center`, `--about`, `--default-apps` | `control-center` (avec les pages `about` et `apps`) |
| page `--displays`, `--monitors` | `control-center displays` |
| `--network-interfaces` | `control-center network interfaces` |
| `--system-monitor`, `--terminal`, `--game-snake` | `system-monitor`, `terminal`, `game-snake` |
| `--calendar`, `--removable-devices`, `--storage` | `taskbar-calendar`, `removable-devices` |
| `--focus`, `--clock` | `focus`, `clock` |
| `--spf`, `--superfile` | le gestionnaire de fichiers (`spf`) |
| `--yazi`, `--yazy` | le navigateur de fichiers (`yazi`) |
| `--setup` | `setup` |
| `--start-desktop` | démarre `tty` (aucun argument n'est transmis) |
| `--version`, `-V`, `version` | tirages `ARGVUS <version>` |
| `help`, `--help`, `-h` | imprime l'utilisation |
| `networkctl`, `bluetoothctl` | `network`, `bluetooth` |
| `widget-telemetry-toggle`, `widget-telemetry-block` | `widget-telemetry toggle`, `widget-telemetry block` |
| `theme-switch` | `config theme-apply` |
| `effects-toggle` | `config effects` |

Le centre de contrôle accepte les anciens indicateurs de page comme premier argument et les mappe aux noms de page. Exemples : `--apps` devient `apps`, `--keyboard-layout` devient `locale` et `--hostname` devient `system`.

## Commandes du script Hyprland

Ces scripts emballés sont exécutés et utilisés par la configuration de Hyprland :

`cheatsheets-hypr`, `cheatsheets-launcher`, `scratchpad`, `hypr-screenshot`, `spaces-switch`, `emoji-picker`, `toggle-sidebar`, `weather-location`, `layout-mode-menu`, `brightness-switch`, `toggle-mode`, `idle-timeout`, `keep-awake`, `hypr-power-menu`.

Un script dans la configuration de l'utilisateur remplace celui emballé. L'ordre de recherche est le même que `_config_path` dans `hyprland.lua` : la copie de la configuration de l'utilisateur, puis la copie des données ARGVUS, puis la copie d'état, puis la copie générée, et seulement ensuite le fichier emballé sous `/usr/share/argvus`.

## Exemples

```sh
argvus control-center displays
argvus taskbar-calendar toggle
argvus widget-telemetry block header system
argvus widget-telemetry toggle status
argvus i18n get widget-telemetry label.device
argvus config get /appearance/theme
argvus network status
argvus bluetooth status
argvus gpg list
```
## Configuration

`argvus setup` copie les paramètres par défaut fournis avec `/usr/share/argvus/<app>` dans `~/.config/argvus/data/<app>`, afin que vous puissiez les personnaliser. Il ne modifie jamais les fichiers fournis.

```sh
argvus setup --copy <app>          # copy one packaged app
argvus setup --copy-all            # copy every packaged app directory
argvus setup --copy <app> --force  # back up the existing destination first
argvus setup --copy <app> --dry-run
```
Une destination qui existe déjà est signalée et laissée telle quelle, sauf si `--force` est fourni. Avec `--force`, le répertoire existant est renommé en `<name>.bak-<timestamp>` avant la copie.

L'application `argvus` est spéciale : elle copie le contenu de son répertoire empaqueté dans la racine de configuration. Comme la racine de configuration existe déjà lorsque l'installation s'exécute, `--copy argvus` nécessite `--force` dans le code actuel.

L'installation valide chaque option avant de modifier quoi que ce soit. Les noms d'application invalides (vides, commençant par `.` ou contenant `/`) sont rejetés.

## Session et rechargement

`SUPER + SHIFT + R` exécute `argvus sessionctl reload`. Le contrôleur de session applique la configuration générée et redémarre uniquement les composants qu'il possède. Il ne copie pas `/usr/share/argvus` dans le répertoire personnel et ne remplace pas un fichier JSON malformé.

## Lumière nocturne

Ouvrez **Bureau & Saisie > Affichages > Lumière nocturne** ou exécutez `argvus control-center displays night-light`. Configurez la température, l'activation manuelle et les horaires locaux fixes via `argvus config display night-light`. Consultez [Lumière nocturne](/docs/user-guide/night-light/) pour les commandes, l'état et les exigences d'exécution.

`argvus tui-terminal --frame TITLE FOOTER COMMAND [ARG...]` exécute une commande interactive à l'intérieur du cadre de terminal partagé ARGVUS. Ctrl+E ferme le cadre ; son appelant fournit le pied de page localisé. Utilisez ceci à l'intérieur d'un terminal, ou comme la commande passée à `argvus tui-terminal --term kitty -- ...`.

`argvus tui-terminal --term kitty --profile ai-agents` utilise une configuration Kitty dédiée avec des onglets cachés et uniquement des raccourcis pour le presse-papiers. L'appelant fournit la commande et le titre ; le placement de la fenêtre est géré par la règle Hyprland pour la classe `argvus-ai`.
