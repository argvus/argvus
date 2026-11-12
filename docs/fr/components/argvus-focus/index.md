---
title: Focus ARGVUS
description: Pomodoro et sessions de concentration avec pauses, notifications et un historique des sessions local.
---
ARGVUS Focus est une application GTK 4 pour travailler lors de sessions de concentration séparées par des pauses. Elle s'ouvre sous forme de fenêtre contextuelle à côté de la barre des tâches. Ouvrez-la depuis le lanceur, en cliquant sur son indicateur dans la barre des tâches, avec **SUPER + Maj + F**, ou avec :

```sh
argvus focus              # open the popup near the pointer
argvus focus --toggle     # hide the popup if it is visible, show it otherwise
argvus focus --toggle --x 1600 --y 30  # open next to a taskbar click position
argvus focus --start      # start or resume the current phase without opening the window
argvus focus --pause      # pause the running phase
argvus focus status --waybar         # print one Waybar JSON status and exit
argvus focus status --waybar --follow  # stream status changes for Waybar
```
La barre des tâches transmet les coordonnées du clic racine afin que le popup utilise l'écran cliqué et s'aligne avec la barre ; un lancement sans coordonnées utilise la position du pointeur. Si le pointeur reste en dehors du popup pendant 5 secondes, celui-ci se ferme automatiquement ; déplacer le pointeur à l'intérieur du popup annule ce compte à rebours. **Échap** sur la page principale le masque toujours immédiatement, et **Échap** sur une sous-page revient à la page précédente. L'indicateur de la barre des tâches n'affiche que son icône lorsqu'il est inactif et le `MM:SS` restant pendant une phase, avec la couleur de la phase ; une phase en pause conserve son temps restant et ajoute un état en pause. La commande `status` s'exécute à l'intérieur de `argvus` sans démarrer GTK. `argvus --focus` est un alias. Une seule instance de Focus s'exécute par session : un deuxième lancement transmet ses options à celle déjà en cours.

ARGVUS Focus, ARGVUS Clock et le Panneau de configuration partagent le même bord de l'écran. Ouvrir Focus ferme d'abord Clock et le Panneau de configuration ; ouvrir l'une de ces surfaces ferme Focus.

## Séances

- **Mode Pomodoro** alterne périodes de concentration et courtes pauses et termine chaque cycle par une longue pause. Les durées par défaut sont de 25, 5 et 15 minutes, avec une longue pause après 4 sessions de concentration.
- **Mode session unique** exécute une seule session de concentration de sa propre durée (45 minutes par défaut) sans pauses.
- La fenêtre affiche la phase (concentration, courte pause, longue pause, chacune dans sa propre couleur), le temps restant à l'intérieur d'un anneau de progression, la position dans le cycle et le total du jour.
- **Démarrer la concentration / Démarrer la pause**, **Pause**, **Reprendre**, **Annuler la session** (abandonne la phase et n'enregistre rien) et **Terminer maintenant** (met fin à la session de concentration prématurément et enregistre le temps réellement passé). Pendant une pause, **Passer la pause** revient à la concentration.
- Lorsque une phase se termine, la suivante attend votre action, sauf si **Démarrer les pauses automatiquement** ou **Démarrer la concentration automatiquement après une pause** est activé.

Clavier : **Espace** démarre ou met en pause, **Ctrl+,** ouvre les paramètres, **Ctrl+H** l'historique, **Échap** revient en arrière, **Ctrl+W** cache la fenêtre et **Ctrl+Q** quitte.

## Paramètres

Les préréglages (Classique 25/5/15, Étendu 50/10/30, Travail profond 90/20/30) remplissent les durées, qui restent modifiables. Les paramètres incluent également les sessions avant une longue pause, les démarrages automatiques, la notification à la fin d'une phase, et les **sons de session** : une courte sonnerie avec son propre volume et un bouton **Tester le son**. Ces sons sont séparés des sons d'alarme de [ARGVUS Clock](/docs/components/argvus-clock/).

## Historique

Chaque session de concentration terminée est enregistrée avec son début, sa fin et le temps passé ; les pauses et les sessions annulées ne le sont pas. La page d'historique montre les sessions et minutes d'aujourd'hui, les totaux et les sessions récentes. Les enregistrements de plus de 365 jours sont supprimés. **Effacer l'historique** demande confirmation.

## Fenêtre, arrière-plan et suspension

- Fermer la fenêtre pendant un cycle ne fait que la cacher : la session continue et vous notifie lorsque la phase se termine. S'il n'y a rien en cours, la fermeture quitte le programme.
- Si le système est suspendu alors qu'une phase est en cours, la phase est mise en pause au moment de la suspension, et un message indique combien de temps le système est resté en veille. Le temps de veille n'est pas compté comme temps de concentration.
- La session est sauvegardée à chaque modification et toutes les 30 secondes. Si ARGVUS Focus est fermé pendant une phase (déconnexion, plantage, **Ctrl+Q**), il reprend en pause à ce point lors de la prochaine ouverture ; il ne prétend jamais que le chronomètre a continué à tourner alors qu'il ne pouvait pas notifier.
- Les temps sont mesurés sur l'horloge de démarrage du système, donc changer l'horloge ou le fuseau horaire ne raccourcit ni n'allonge une phase.

## Chemins

| Chemin | Contenu |
| --- | --- |
| `$XDG_CONFIG_HOME/argvus/data/focus/settings.json` | Paramètres |
| `$XDG_DATA_HOME/argvus/focus/history.json` | Sessions terminées |
| `$XDG_STATE_HOME/argvus/focus/session.json` | Point de contrôle de la session actuelle |
| `/usr/share/argvus/sounds/argvus-bell.oga` | Cloche de session |

Les fichiers sont écrits de manière atomique. Un fichier qui ne peut pas être lu est renommé en `<name>.corrupt-<time>` et les valeurs par défaut sont utilisées ; un message vous indique où il a été déplacé.

Les couleurs suivent le thème actif ARGVUS et la couleur de surlignage (consommateur de thème `time`). Les chaînes proviennent des domaines de traduction `focus` et `time`.

Le calendrier de la barre des tâches, ARGVUS Focus, ARGVUS Clock et le Panneau de configuration sont mutuellement exclusifs : ouvrir l'un ferme les autres.
