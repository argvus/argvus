---
title: Cycle de vie d'exécution
description: Démarrage de session et propriété du service utilisateur.
---
`argvus-session` importe l'environnement, démarre Hyprland via `argvus start` et lance `argvus-session.target` après que le compositeur soit prêt. La cible intègre les services de shell, notification, fond d'écran, inactivité et presse-papiers. `argvus sessionctl` coordonne le démarrage, l'arrêt, le rechargement, le redémarrage, le statut et les journaux.

La session arrête la cible lorsque Hyprland se termine. Les projets individuels restent responsables de leurs propres binaires et configurations.

## Démarrage

Le point d'entrée graphique est `/usr/share/wayland-sessions/argvus.desktop`, qui invoque `argvus session`. La session importe l'environnement de configuration ARGVUS et XDG, le synchronise avec `systemd --user` et l'activation D-Bus, et délègue le démarrage du compositeur à `argvus start`.

`argvus start` valide la configuration de Hyprland, applique le repli packagé lorsqu'un fichier généré par défaut est trouvé ou qu'une configuration utilisateur est invalide, détecte la compatibilité de la virtualisation et lance Hyprland. Le pont de disponibilité du compositeur appelle ensuite `argvus sessionctl ready`, qui démarre la cible ARGVUS avec l'environnement Wayland et Hyprland actuel.

La cible utilise une unité de préparation ponctuelle avant de démarrer les composants de longue durée. La préparation applique la configuration générée, l'état du thème et de l'accent, l'état du moniteur, l'intégration des applications et l'environnement de session. Les services persistants sont détenus par `systemd --user`, et non par des processus de fond ad hoc.

`argvus start` met en file d'attente `argvus-config.service` (`--no-block`) juste avant de lancer Hyprland : la projection n'a pas besoin de compositeur, donc elle chevauche le démarrage du compositeur. `argvus sessionctl ready` démarre ensuite la cible, qui constate que l'unité est active ou attend son travail, de sorte que la configuration canonique est encore projetée avant que la préparation ne s'exécute. Une fois la cible démarrée, `sessionctl ready` attend jusqu'à 1,5 seconde que le fond d'écran (une surface de couche d'arrière-plan) et la barre des tâches soient mappés avant de retirer la superposition de chargement, car un service simple est considéré comme démarré avant de dessiner ; une surface dont l'unité ne fonctionne pas n'est pas attendue, et l'attente est enregistrée comme `session.desktop-surfaces`, avec `session.wallpaper-surface` et `session.taskbar-surface` pour le temps que chaque surface a pris. Une défaillance fatale de projection ou de démarrage de cible est enregistrée et arrête la superposition de chargement indépendante, empêchant un spinner apparemment sans fin. Optionnel les consommateurs visuels doivent résoudre un contournement ou signaler un échec dégradé sans remplacer la cible de session.

### Coût de connexion

Une connexion sans changement de configuration ne répète aucun travail coûteux :

- La projection exécute le changement de thème et l'adaptateur d'effets uniquement lorsque les sections apparence ou effets ont changé, ou lorsque leurs sorties réelles (les copies d'utilisateur matérialisées et `data/state/effects/<theme>.conf`) sont manquantes.
- La préparation applique le thème de démarrage et l'accent par-dessus uniquement lorsque le tampon dans `$XDG_CACHE_HOME/argvus/session/theme-startup.stamp` ne correspond plus aux fichiers de thème emballés et au marqueur d'accent. Le tampon crée une empreinte des contenus de fichiers, donc réinstaller des fichiers identiques ne l'invalide pas. Des entrées de thème modifiées ou un nouvel accent l'invalident une fois. La migration d'un tampon basé sur un horodatage plus ancien le reconstruit également une fois. Lorsque le thème est réappliqué, la préparation recharge également Hyprland, qui avait démarré avec les fichiers précédents.
- Les paramètres GTK (`gsettings`) ne sont écrits que lorsque le thème de démarrage a été réappliqué. Entre les connexions, dconf les conserve, donc une connexion stable ne les exécute pas à nouveau.
- `argvus-dunst.service` est `Type=dbus` : il est considéré comme démarré une fois que Dunst possède `org.freedesktop.Notifications`, donc la cible de session n'attend pas que l'état de pause soit restauré.
- La préparation exécute `spaces-switch.sh` et `borders-switch.sh` (`--apply-static`) côte à côte : ils écrivent des fichiers disjoints (les marges JSONC et la géométrie effective des barres, et les CSS des barres et le thème Rofi).
- La projection réécrit un fichier généré uniquement lorsque son contenu a changé. Les fichiers inchangés sont laissés tels quels, donc une connexion stable ne les fsync pas à nouveau.
- L'étape d'affichage de la préparation (`argvus config display session-prepare`) ne s'applique qu'au DPMS et aux sorties désactivées : elle ne projette pas à nouveau et ne recharge pas le compositeur.
- `argvus-session-reload.service` exécute `argvus sessionctl reload-desktop --login`, qui ne recharge Hyprland que lorsque le manifeste de projection `hyprland` parmi les consommateurs concernés. Les composants du bureau n’attendent pas.

Le spinner de session (l’assistant de transfert en mode Wayland) est mappé depuis le transfert compositeur jusqu’à ce que `argvus sessionctl ready` ait commencé la cible, donc un seul spinner couvre tout le démarrage. Il n’a pas de `PartOf=`, car recréer la cible à un nouveau login ne doit pas la supprimer ; `sessionctl` l’arrête explicitement, et `RuntimeMaxSec=30` la termine si aucun arrêt n’arrive.

Chaque étape de préparation, de rechargement et d’adaptateur de projection ajoute sa durée au journal de session comme `<epoch-ms> timing <step> <N>ms` ; voir [session startup troubleshooting](/docs/user-guide/troubleshooting/session-startup/).

La résolution des chemins, la préparation de la session et le plan de rechargement de Hyprland sont gérés par Rust. `argvus config session-prepare` et `argvus config session-reload-config` exécutent les étapes de démarrage et de rechargement ; l'ancien script `hypr-init.sh` a été supprimé.

## Services et arrêt

La cible possède généralement `argvus-control-panel.service`, `argvus-taskbar.service`, `argvus-widget-telemetry.service` lorsqu'ils sont activés, `argvus-wallpaper.service`, `argvus-dunst.service`, `argvus-hypridle.service`, des surveillants de presse-papier, des notifications de disposition du clavier et l'intégration PolicyKit. L'overlay de chargement de la session est lancé par `argvus start`, et non par la cible. Une défaillance dans un service optionnel ne remplace pas toute la session.

Les services nécessitant une connexion client Wayland, y compris le Panneau de Contrôle et le gestionnaire d'inactivité, attendent qu'un runtime Hyprland utilisable soit disponible avant de lancer leur client. Cela évite de démarrer Quickshell ou hypridle avec une socket obsolète pendant la transition du compositeur ; si Hyprland est remplacé, systemd peut relancer le composant après que le nouveau runtime est disponible.

Un changement de thème sérialise son commit, sa construction, sa projection et le rechargement des consommateurs comme une seule opération de cycle de vie sous le verrou du thème. Il ne recharge pas `argvus-config.service` ni ne redémarre les services : la projection qu'il exécute écrit seulement des fichiers et laisse le thème changer pendant que le verrou est actif, et les consommateurs en cours se rechargent sur place (voir [Apparence et thèmes](/docs/developer-guide/subsystems/appearance-and-themes/)).

Lorsque Hyprland se ferme, `argvus start` arrête `argvus-session.target` et les services qui y sont liés. Cela empêche les processus de la barre des tâches, de l'environnement de bureau, des notifications, de l'inactivité et du presse-papiers de survivre à la déconnexion.

## Rechargement

```sh
argvus sessionctl reload
```

Reload réimporte l'environnement graphique, puis **applique** une génération validée lorsqu'une commande de domaine précédente telle que `apply-theme`, `accent`, `set`/`patch` ou un changement de bloc de widget-télémétrie a déjà projeté le changement. Dans ce cas, il restaure les consommateurs listés dans le plan du manifeste et n'appelle pas `argvus config project` à nouveau, car reprojeter reconstruirait un arbre identique et supprimerait le plan. La projection, la migration et `ensure` ne sont effectuées que lorsqu'aucun plan en attente valide n'existe — au démarrage de la session, ou après que `argvus config recover` doit restaurer le document précédent. Une opération sans effet propre est autorisée uniquement lorsque les projections et le runtime géré sont tous deux à jour. Si l'état canonique n'a pas changé mais qu'une socket de compositeur, un service ou un consommateur est obsolète, reload le réconcilie au lieu de retourner prématurément. La propagation normale couvre Hyprland, la barre des tâches, la télémétrie des widgets, le panneau de configuration, les notifications, le fond d'écran et l'inactivité. selon le manifeste de projection, tandis que les consommateurs optionnels inchangés restent intacts. Chaque fichier sous `data/generated/` est écrit par `argvus-config` à ce stade, donc la session ne réaffiche jamais elle-même un thème. Le rechargement exécute ensuite `theme-adapters.sh`, qui réconcilie les consommateurs qui ne peuvent pas lire le document canonique ou la projection publiée par eux-mêmes. Ces adaptateurs ne lisent que la génération validée : ils écrivent leurs propres fichiers et caches de consommateurs, et ils n'écrivent jamais le document canonique ou `data/generated/`, donc une seule action utilisateur coûte toujours une mutation, une projection, une publication et un rechargement. Utilisez `argvus sessionctl restart <component>` uniquement lorsqu’un service possédé unique nécessite un redémarrage explicitement isolé. Inspectez la [référence des services ](/docs/reference/services/) pour les noms exacts des unités.

Les opérations de projection et de rechargement sont sérialisées avec un verrou d'exécution utilisateur. Pour une trace temporelle temporaire, définissez `ARGVUS_SESSION_PERF=1` avant d'invoquer le rechargement et inspectez `~/.local/state/argvus/session.log` ; la trace enregistre les limites des étapes sans modifier le comportement normal du rechargement.
