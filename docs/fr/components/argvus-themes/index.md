---
title: Thèmes
description: Installer, sélectionner, personnaliser, importer et exporter des thèmes ARGVUS.
---
Un thème dans ARGVUS définit les couleurs, les surfaces, les fonds d'écran par défaut et la géométrie de la disposition utilisés sur le bureau : Hyprland, Waybar, le Panneau de Contrôle, la barre des tâches, l'écran de verrouillage, les notifications et les applications ARGVUS. Ce guide couvre tout ce dont vous avez besoin pour travailler avec les thèmes en tant qu'utilisateur.

## Ce qui est inclus dans un thème

Chaque thème officiel est une identité visuelle complète avec deux modes :

- **Sticky** — une disposition compacte avec de petits écarts entre les fenêtres, pas d'espace extérieur et une barre des tâches proche du bord de l'écran.
- **Spaced** — une disposition spacieuse avec des écarts plus grands, des coins arrondis et des marges autour de la barre des tâches et des surfaces du shell.

Sélectionner un thème ne change pas son identité : la palette, la couleur de surlignage par défaut et l'association du fond d'écran restent les mêmes entre les modes. Le mode est un paramètre séparé, voir [Thèmes officiels](/docs/components/argvus-themes/official-themes/)].

## Pages de ce guide

- [Installation des thèmes](/docs/components/argvus-themes/installing-themes/) — d'où viennent les thèmes officiels et comment en installer un ou tous avec `pacman`.
- [Thèmes officiels](/docs/components/argvus-themes/official-themes/) — le catalogue, la couleur de surlignage par défaut de chaque thème, et comment choisir un thème et son mode.
- [Personnalisation de la couleur de surlignage](/docs/components/argvus-themes/highlight-color/) — changer la couleur d’accent indépendamment du thème.
- [Importation et exportation de thèmes](/docs/components/argvus-themes/import-and-export/) — exporter le thème actif, avec votre couleur de surlignage et votre fond d’écran, en tant que thème personnel et l’importer sur une autre machine.

## Référence rapide

| Tâche | Où |
| --- | --- |
| Choisir un thème | **Centre de Contrôle → Apparence → Thèmes → Thèmes Officiels** |
| Basculer entre Collant et Espacé | **Centre de Contrôle → Apparence → Mode** |
| Changer la couleur de surlignage | **Centre de Contrôle → Apparence → Couleur de surlignage** |
| Exporter le thème actif | **Centre de Contrôle → Apparence → Thèmes → Exporter le thème** |
| Importer un thème | **Centre de Contrôle → Apparence → Thèmes → Importer le thème** |
| Ouvrir le sélecteur de thèmes depuis le clavier | `SUPER + SHIFT + T` |

## D'où viennent les thèmes

ARGVUS fournit deux thèmes intégrés, **ARGVUS Dark** et **ARGVUS Light**, avec le paquet `argvus-appearance`. Chaque autre thème officiel est un paquet `argvus-theme-<name>` séparé. L'installation d'un paquet de thème l'ajoute au sélecteur de thèmes ; aucun redémarrage de la session n'est nécessaire. Vos propres thèmes, exportés ou importés, sont listés séparément comme **Thèmes personnalisés**.

## Conseils pour le clavier

Les pages de thèmes du Centre de Contrôle affichent leurs touches dans le pied de page :

- `↑` / `↓` — naviguer.
- `Enter` — appliquer ou ouvrir.
- `e` — exporter le thème actif, sur une page Thèmes.
- `i` — importer un thème, sur une page Thèmes.
- `d` — supprimer un thème personnalisé.
- `Esc` — revenir en arrière.

## Dépannage

- **Un thème que j'ai installé n'apparaît pas.** Vérifiez que le paquet est installé avec `pacman -Q argvus-theme-<name>`, puis rouvrez le Centre de Contrôle. Le paquet doit être installé avec `argvus-appearance` présent.
- **La couleur de surlignage a changé après avoir changé de thème.** Une couleur de surlignage personnalisée survit aux changements de thème. Utilisez **Réinitialiser par défaut du thème** pour redonner au thème sa propre couleur.
- **Une archive de thème n'est pas listée pour l'importation.** Elle doit être directement dans votre répertoire personnel et contenir `<id>/theme.toml`. Voir [Importer et exporter des thèmes](/docs/components/argvus-themes/import-and-export/).
