---
title: Horloge ARGVUS
description: Horloge, horloges mondiales, chronomètre, minuteur et alarmes avec son.
---
ARGVUS Clock est une application GTK 4 avec l'heure actuelle et les outils temporels : horloges mondiales, un chronomètre, un compte à rebours et des alarmes. Ouvrez-la depuis le lanceur, en cliquant sur l'heure dans la barre des tâches, avec **SUPER + Maj + C**, ou avec :

```sh
argvus clock                     # open the popup near the pointer
argvus clock --toggle            # hide the popup if it is visible, show it otherwise
argvus clock --toggle --x 1600 --y 30  # open next to a taskbar click position
argvus clock --page alarms       # open on a page: clock, world, stopwatch, timer, alarms
argvus clock --background        # run without a window, to keep alarms active
```
L'horloge ARGVUS s'ouvre sous forme de popup aligné avec la barre des tâches, avec un écart de 4 pixels. La barre des tâches transmet les coordonnées du clic racine afin que le popup utilise le moniteur cliqué ; le lancer sans coordonnées utilise la position du pointeur. Si le pointeur reste en dehors du popup pendant 5 secondes, celui-ci se ferme automatiquement ; déplacer le pointeur à l'intérieur du popup annule le compte à rebours. **Échap** sur un onglet principal le cache toujours immédiatement, et **Échap** sur une page complète revient à l'onglet précédent. `argvus --clock` est un alias. Cliquer sur la date dans la barre des tâches ouvre toujours le calendrier ; l'horloge ARGVUS n'affiche pas les événements.

ARGVUS Focus, ARGVUS Clock et le Panneau de configuration partagent le même bord de l'écran. L'ouverture de l'horloge ferme d'abord Focus et le Panneau de configuration ; ouvrir l'une de ces surfaces ferme l'horloge.

## Pages

- **Horloge** : l'heure en grands chiffres (24 heures par défaut, 12 heures dans les paramètres, secondes optionnelles), la date localisée, le fuseau horaire local et son décalage UTC, et la prochaine alarme.
- **Monde** : villes ajoutées en recherchant une ville ou un fuseau horaire IANA (les accents et la casse sont ignorés). Chacune affiche son heure, la différence avec l'heure locale et si c’est hier, aujourd’hui ou demain là-bas. L’heure d’été provient de la base de données des fuseaux horaires IANA ; rien n’est calculé avec des décalages fixes. Les villes peuvent être réordonnées et supprimées.
- **Chronomètre** : démarrer, mettre en pause, reprendre, réinitialiser et tours (temps au tour et total).
- **Minuteur** : heures, minutes et secondes, ou les présélections 5, 10, 15, 25, 30 et 60 minutes. Pause, reprendre, redémarrer et annuler. À la fin, il sonne comme une alarme. Ce minuteur est indépendant des sessions ARGVUS Focus.
- **Alertes** : créer, modifier, activer et désactiver, et supprimer des alertes. Chacune a une heure, un nom facultatif, les jours de la semaine où elle se répète (aucun signifie qu'elle sonne une seule fois), un son et une durée de rappel.

Clavier : **Ctrl+1** à **Ctrl+5** pour changer de page, **Ctrl+N** pour créer une alerte, **Ctrl+,** pour ouvrir les paramètres, **Échap** pour revenir en arrière, **Ctrl+W** pour fermer la fenêtre et **Ctrl+Q** pour quitter.

## Son d'alerte

Les alertes jouent un vrai son jusqu'à ce que vous y répondiez :

- Sons : **Carillon** (par défaut) et **Balise**, expédiés dans `/usr/share/argvus/sounds/`.  
- **Volume de l'alarme** (10 à 100, par défaut 80) et **Écouter le son de l'alarme** se trouvent dans les paramètres ; l'éditeur dispose d'un bouton de test pour le son choisi.  
- Une alarme qui sonne affiche une carte en haut de l'écran, au-dessus des autres fenêtres, et une notification, toutes deux avec **Snooze** et **Ignorer**. Avec la carte en focus, **Entrée** ignore et **S** reporte.  
- Le rappel sonne à nouveau après la durée de snooze de l'alarme (1 à 60 minutes, par défaut 10).  
- Une alarme qui n'est pas répondue s'arrête après **Arrêter de sonner après** (5, 10, 15 ou 30 minutes, par défaut 10) et laisse une notification.  
- Deux alarmes qui sonnent en même temps sont toutes deux listées ; la seconde n’éteint jamais la première.  
- Si le son ne peut pas être joué, la carte et la fenêtre l’indiquent avec l’erreur, et l’erreur est enregistrée. ARGVUS Clock ne signale jamais un son qui n’a pas été joué.

La lecture utilise le support multimédia de GTK (GStreamer), donc elle fonctionne avec PipeWire, PulseAudio ou ALSA. Le paquet dépend de `gst-plugins-base` (Ogg Vorbis) et `gst-plugins-good` (sortie audio).

## Quand les alarmes sonnent

Les alarmes ne sonnent que lorsque ARGVUS Clock fonctionne dans votre session :

- Fermer la fenêtre permet à ARGVUS Clock de continuer à fonctionner lorsque les alarmes, un minuteur en cours ou un chronomètre en cours en ont besoin (**Continuer à fonctionner lorsque la fenêtre est fermée**, activé par défaut). S'il n'y a rien en attente, il se quitte.
- **Démarrer avec la session** active l'unité utilisateur `argvus-clock.service`, qui exécute `argvus clock --background` après la connexion. Elle est désactivée par défaut.
- Les alarmes ne peuvent pas sonner lorsque vous êtes déconnecté, que l'ordinateur est éteint ou en veille.
- Une alarme détectée jusqu'à 5 minutes en retard sonne toujours. Plus tard que cela (l'horloge ne fonctionnait pas, ou le système était en veille ou éteint), elle ne sonne pas : elle est signalée comme **manquée** dans une notification et sur la page des Alarmes. Une alarme récurrente passe au jour suivant ; une alarme unique se désactive.
- Chaque occurrence est enregistrée comme traitée avant qu'elle ne sonne, donc rouvrir ARGVUS Clock ne fait jamais sonner la même alarme deux fois, et une seule instance fonctionne par session.
- Les alarmes récurrentes sont calculées en heure locale aux jours choisis, donc elles conservent leur heure malgré les changements d'heure d'été. Une heure sautée lors du passage au printemps sonne à la première minute après le trou ; une heure qui se produit deux fois sonne la première fois.
- Un minuteur continue de compter pendant une suspension. S'il s'est terminé pendant que le système était en veille, il sonne au réveil et indique le retard. Après la fermeture d'ARGVUS Clock, un minuteur ou chronomètre en cours revient en pause.

## Chemins

| Chemin | Contenu |
| --- | --- |
| `$XDG_CONFIG_HOME/argvus/data/clock/settings.json` | Paramètres et horloges mondiales |
| `$XDG_DATA_HOME/argvus/clock/alarms.json` | Alarmes |
| `$XDG_STATE_HOME/argvus/clock/timers.json` | Minuterie et point de contrôle du chronomètre |
| `/usr/lib/systemd/user/argvus-clock.service` | Démarrer avec la session |
| `/usr/share/argvus/sounds/` | Sons emballés |

Les fichiers sont écrits de manière atomique ; un fichier qui ne peut pas être lu est renommé en `<name>.corrupt-<time>` et des valeurs par défaut sont utilisées. Les couleurs suivent le thème ARGVUS actif et la couleur de surlignage (consommateur de thème `time`). Les chaînes proviennent des domaines de traduction `clock` et `time`.

Le calendrier de la barre des tâches, ARGVUS Focus, ARGVUS Horloge et le Panneau de configuration sont mutuellement exclusifs : ouvrir l’un ferme les autres.
