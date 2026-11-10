---
title: Calendrier
description: Utilisez le calendrier de la barre des tâches ARGVUS.
---
`argvus-taskbar-calendar` est le popup du calendrier GTK4 ouvert depuis le module de date de Waybar ou avec :

```sh
argvus --calendar
argvus taskbar-calendar toggle
argvus taskbar-calendar status
```
Il prend en charge les événements locaux, les rappels et l'import/export ICS. Son service utilisateur est `argvus-taskbar-calendar.service`. Utilisez `argvus taskbar-calendar --help` pour les options prises en charge par la version installée.

Le calendrier suit le thème ARGVUS actif, y compris les variantes Sticky et Spaced. Les changements de thème sont appliqués via le cache utilisateur généré ; utilisez `argvus taskbar-calendar reload` si un popup en cours d'exécution nécessite un rafraîchissement explicite.

Lorsqu'il est ouvert depuis le module de date de la barre des tâches, le popup est ancré au bord de la barre : il s'ouvre en dessous d'une barre des tâches en haut et au-dessus d'une barre des tâches en bas, aligné au bord droit du module de date. Les lancements directs sans géométrie de la barre des tâches utilisent le pointeur ou le moniteur comme solution de repli.

## Commandes

```bash
argvus taskbar-calendar                 open the calendar
argvus taskbar-calendar toggle          show or hide it (the Waybar date module uses this)
argvus taskbar-calendar show | hide
argvus taskbar-calendar config          open the system configuration for editing (sudo)
argvus taskbar-calendar config-path     print the configuration path
argvus taskbar-calendar reload
argvus taskbar-calendar import file.ics
argvus taskbar-calendar export --output calendar.ics
argvus taskbar-calendar sync
argvus taskbar-calendar status
argvus taskbar-calendar service
```
## Événements et rappels

Les événements sont désactivés par défaut. Activez-les dans la fenêtre pop-up sous **Paramètres > Événements** ;
le choix est conservé dans le répertoire d'état et active également et démarre `argvus-taskbar-calendar.service`, qui affiche les rappels. Les désactiver arrête le service.

- **+ ÉVÉNEMENT** (ou `n`) ouvre l'éditeur pour le jour sélectionné. Les événements peuvent être créés et modifiés à partir d'aujourd'hui ; un événement d'un jour passé peut toujours être ouvert et supprimé.
- Un nouvel événement commence à 09:00 et dure la longueur par défaut définie dans les Paramètres. Déplacer l'heure de début conserve la durée ; une heure de fin antérieure à l'heure de début signifie que l'événement se termine le jour suivant.
- Chaque événement affiche une notification avec un son à son début, quel que soit le contenu des cases **Rappel** et **Répétition**. Un événement enregistré après son début (un événement sur toute la journée créé au cours de la journée, par exemple) notifie dans les 30 secondes.
- **Rappel** ajoute une notification anticipée, à l'heure choisie avant le début (au début si cette heure est déjà passée).
- Pour un événement sur toute la journée, le champ horaire devient **NOTIFIER À** : l'événement couvre toute la journée et notifie à cette heure (00:00 notifie au début de la journée). L'heure est conservée lorsque l'événement est rouvert.
- Pour les événements sur toute la journée, **Répéter** notifie à nouveau à l'intervalle choisi après cette heure.
- Modifier un événement sans déplacer son début ne répète pas les rappels déjà affichés.
- Les notifications suivent la langue du calendrier (Paramètres > Langue).
- Un jour affiche un point (`•`) lorsqu'un événement a lieu ce jour-là, y compris chaque jour d'un événement d'une nuit ou de plusieurs jours. Les jours suivent le fuseau horaire local.
- Les événements locaux sont supprimés automatiquement 10 minutes après leur fin. Si un événement est déjà terminé lorsqu'il est enregistré, il reste disponible au moins 10 minutes après l'enregistrement.

Les fichiers importés `.ics` conservent leurs dates, fuseaux horaires (`TZID`) et alarmes (`VALARM`). Les événements importés notifient également à leur début, en plus de leurs propres alarmes. Les événements récurrents (`RRULE`) sont stockés et exportés, mais seule leur première occurrence est affichée et rappelée. Après une mise à jour du paquet, le service de rappel se termine et systemd lance la nouvelle version dans les 30 secondes. `sync` ne rapporte que les calendriers distants configurés ; la synchronisation CalDAV n’est pas encore implémentée.

## Chemins

- Configuration : `/etc/argvus/taskbar/calendar/config.toml` (à l'échelle du système ; une valeur par défaut est fournie avec le package).
- Styles : `/etc/argvus/taskbar/calendar/style.css` et `theme.css` ; les couleurs du thème ARGVUS actif proviennent du thème généré via le cache ci-dessous.
- Données : `$XDG_DATA_HOME/argvus-taskbar-calendar/` (ou `~/.local/share/argvus-taskbar-calendar/`).
- État : `$XDG_STATE_HOME/argvus/taskbar/calendar/` (ou `~/.local/state/...`) ; le basculement des événements y est conservé.
- Cache : `$XDG_CACHE_HOME/argvus-taskbar-calendar/` (ou `~/.cache/argvus-taskbar-calendar/`).

La couleur de surbrillance est lue depuis `~/.config/argvus/data/.accent-color` et appliquée au moment de l'exécution
par-dessus les couleurs du thème.

La sélection automatique de la langue suit l'utilisateur `locale.conf`, l'anglais étant la valeur par défaut. Les préférences explicites de langue du calendrier continuent de primer sur la sélection automatique.

La barre des tâches du calendrier, ARGVUS Focus, ARGVUS Clock et le Panneau de configuration sont mutuellement exclusifs : l'ouverture de l'un ferme les autres.

Les échecs de lecture du calendrier sont affichés séparément d'un agenda vide. L'ouverture de la base de données, le chargement des événements quotidiens et le chargement des indicateurs mensuels ont des messages distincts. Sélectionnez à nouveau un jour pour réessayer. Un rafraîchissement échoué conserve uniquement les données précédemment chargées pour le même jour ou mois ; les résultats valides de l'autre requête restent disponibles.

## Catalogues de langues partagés

L'interface utilise les catalogues partagés `argvus-i18n` pour les dix langues. Les textes sont stockés dans `resources/locales/<locale>/taskbar-calendar.json` dans l'espace de travail principal et installés par `argvus`. L'anglais est la langue de repli ; les nouvelles langues ne nécessitent plus de dictionnaire bilingue local.
