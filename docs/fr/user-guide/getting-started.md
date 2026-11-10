---
title: Première configuration
description: Un chemin de premier démarrage pratique pour configurer ARGVUS.
---
Après la première session graphique, vous n'avez pas besoin de configurer chaque partie d'ARGVUS. Commencez par les paramètres qui affectent la sensation du bureau, puis configurez le matériel et les services selon vos besoins.

## Un ordre utile

1. Lisez la [vue d'ensemble du bureau](/docs/user-guide/desktop/) pour identifier les fenêtres, les espaces de travail, la barre des tâches, le Panneau de configuration et le lanceur.
2. Ouvrez le [Centre de contrôle](/docs/components/argvus-control-center/) et choisissez un thème, un mode, un accent et un fond d'écran dans **Apparence**.
3. Ajustez [les fenêtres et la disposition](/docs/argvus-hyprland/windows-and-layout/) si les écarts, les bordures ou l'espacement de la barre des tâches par défaut ne conviennent pas à votre écran.
4. Configurez la [barre des tâches](/docs/argvus-taskbar/taskbar/) et décidez quelles cartes du Panneau de configuration vous souhaitez visibles.
5. Passez en revue les [raccourcis clavier](/docs/argvus-hyprland/keyboard-shortcuts/) et modifiez uniquement les assignations que vous utilisez réellement.
6. Configurez la [souris et le pavé tactile](/docs/argvus-hyprland/input/), les écrans, le réseau et l'alimentation depuis les pages de paramètres pertinentes.

Les quatre premières étapes sont des personnalisations optionnelles. Les étapes restantes ne sont normalement nécessaires que lorsque votre matériel, votre langue ou votre flux de travail l'exige.

## Deux endroits pour modifier les paramètres

Utilisez **Centre de contrôle** pour les paramètres qui décrivent comment ARGVUS doit être configuré : apparence, disposition, polices, saisie, raccourcis, langue, région et applications par défaut. Utilisez **Panneau de configuration** pour l'état et les actions fréquentes de la session telles que le volume, la luminosité, l'état du réseau, les notifications, l'alimentation et les contrôles de session.

Voir [Centre de contrôle vs Panneau de configuration](/docs/components/argvus-control-center/) pour la distinction complète et [Où configurer les choses](/docs/user-guide/where-to-configure/) pour un index rapide.

## Modifications, persistance et récupération

La plupart des modifications du Centre de contrôle sont appliquées pendant la session et enregistrées en tant qu'état utilisateur ARGVUS. Les changements d'apparence sont engagés dans la configuration canonique, et `argvus-config` projette les fichiers consommés par Hyprland et le shell à partir de celle-ci. Un basculement du Panneau de configuration peut être une action directe ou un changement d'état de session plutôt qu'une configuration permanente.

Lorsque qu'une page propose **Réinitialiser les paramètres par défaut**, utilisez cette action de page plutôt que de supprimer les fichiers générés. L'apparence a des actions de réinitialisation séparées pour les accents, les fonds d'écran et les valeurs de disposition ; la page des raccourcis clavier a sa propre action de restauration. Il n'existe pas de bouton de réinitialisation unique pour tous les réglages du bureau.

## Prochaines étapes

- [Apparence](/docs/user-guide/appearance/) — thèmes, fonds d'écran, effets et disposition.
- [Disposition du bureau](/docs/argvus-hyprland/windows-and-layout/) — comprendre les espaces, les bordures et l'espace du panneau.
- [Périphériques d'entrée](/docs/argvus-hyprland/input/) — contrôle de la souris et du pavé tactile.
- [Résolution des problèmes de thème et de fond d'écran](/docs/user-guide/troubleshooting/theme-and-wallpaper/) — récupération lorsque le changement visuel n'est pas celui que vous attendiez.
