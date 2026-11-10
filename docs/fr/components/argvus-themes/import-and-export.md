---
title: Importer et exporter des thèmes
description: Exporter le thème actif en tant que thème personnel, importer une archive de thème, l'appliquer et la supprimer.
---
L'exportation et l'importation fonctionnent avec des **thèmes**, dans le même format que les thèmes officiels. L'exportation enregistre le thème actif, avec votre couleur de surlignage et votre fond d'écran, comme un nouveau thème à vous. L'importation installe une archive de thème parmi vos thèmes. Les deux se trouvent dans **Centre de Contrôle → Apparence → Thèmes**, et vos thèmes sont listés dans **Thèmes Personnalisés**.

## Ce que contient un thème exporté

L'archive, `argvus-theme-<id>.tar.gz`, contient un répertoire de thème, structuré comme un thème officiel :

```text
<id>/theme.toml        the manifest (schema 2): name, category, palette, wallpaper
<id>/appearance.json  your visual settings, including layout, effects, fonts and panel presentation
<id>/effects.conf     per-surface effect values for the active theme, when present
<id>/overrides/...     the overrides of the theme it was made from, if any
<id>/wallpaper.<ext>   the wallpaper on screen when you exported
```
Le manifeste est basé sur le thème actif, avec votre nom, un nouvel identifiant, votre couleur de surlignage comme accent de la palette et le fond d'écran à côté. Le profil d'apparence préserve les paramètres visuels actuellement appliqués dans ARGVUS, y compris l'espacement des fenêtres, les bordures, les marges de la barre des tâches, la présentation des panneaux, les couleurs, les effets et les polices. Le fond d'écran personnalisé et la couleur d'accent sont stockés avec le thème. L'application de n'importe quel thème réapplique son profil complet (ou les paramètres par défaut d'ARGVUS lorsqu'aucun profil n'existe), remplaçant les modifications visuelles indépendantes effectuées depuis la dernière application. Le basculement entre Sticky ou Spaced réapplique toujours la géométrie des fenêtres et de la barre des tâches de ce mode ; sauvegardez votre géométrie personnelle dans un thème personnalisé exporté pour la rendre disponible lorsque ce thème est appliqué. Les paramètres non-visuels tels que les préférences audio, clavier et réseau ne font pas partie d'un thème.

L'apparence appliquée persiste à travers les redémarrages et rechargements de session. La préparation de la session reconstruit les fichiers nécessaires à partir de la configuration actuelle sans restaurer la géométrie du thème d'origine.

Les paramètres modifiés après l'application d'un thème sont temporaires jusqu'au prochain changement de thème ou de mode. Exportez le thème depuis **Centre de Contrôle → Apparence → Thèmes → Exporter le thème** pour préserver la configuration visuelle actuelle et la restaurer plus tard en appliquant ce thème exporté.

Vos thèmes sont installés dans `~/.local/share/argvus/themes/<id>/`. Extraire une archive à cet endroit installe également le thème.

## Exporter le thème actif

1. Ouvrez **Centre de Contrôle → Apparence → Thèmes**.
2. Choisissez **Exporter le thème**, ou appuyez sur `e` sur une page Thèmes.
3. Tapez un nom pour le thème. Le champ montre le fichier qui sera écrit.
4. Appuyez sur `Enter`.

L'identifiant est créé à partir du nom : lettres minuscules, chiffres et traits d'union. Lorsque l'identifiant est déjà utilisé par un thème installé ou par une archive dans votre répertoire personnel, `-2`, `-3`, et ainsi de suite sont ajoutés. L'archive est écrite dans votre répertoire personnel, par exemple `~/argvus-theme-my-theme.tar.gz`. ARGVUS vérifie le thème avant de l'écrire : le manifeste doit se charger et chaque composant doit s'afficher dans les deux modes.

## Importer un thème

1. Copiez l'archive `.tar.gz` dans votre répertoire personnel (`~`). ARGVUS liste les archives présentes contenant un thème.
2. Ouvrez **Centre de contrôle → Apparence → Thèmes → Importer un thème**, ou appuyez sur `i` sur une page Thèmes.
3. Sélectionnez l'archive et appuyez sur `Enter`.

ARGVUS vérifie l'archive avant de l'installer. Elle doit contenir un seul répertoire avec un `theme.toml`, uniquement avec des chemins relatifs (pas de `..`, pas de liens), au maximum 128 entrées et une limite de taille par fichier ; l'identifiant doit correspondre au répertoire, le fond d'écran doit être dans l'archive et chaque composant doit s'afficher dans les deux modes. Un profil d'apparence inclus doit utiliser des sections prises en charge et un JSON valide. Si une vérification échoue, rien n'est installé.

Le thème est ajouté à **Thèmes personnalisés**. L'importation ne l'applique pas : vous choisissez quand.

### Conflits de nom

- Si l'un de vos thèmes a le même identifiant, ARGVUS demande **Remplacer votre thème avec le même ID ?**. Choisissez **Remplacer** pour le remplacer, ou appuyez sur `Esc` pour conserver le vôtre. Le thème remplacé est conservé comme sauvegarde jusqu'à ce que le nouveau soit installé.
- Une archive avec l'identifiant d'un thème officiel est refusée. Exportez-la à nouveau sous un autre nom.

## Appliquer un thème personnalisé

1. Ouvrez **Centre de Contrôle → Apparence → Thèmes → Thèmes Personnalisés**.
2. Sélectionnez le thème et appuyez sur `Enter`.

La configuration complète du thème exportée est appliquée, y compris son fond d'écran, l'espacement, les bordures, la couleur d'accent, les effets, les polices et la présentation du panneau. Ses valeurs visuelles remplacent les paramètres actuels. ARGVUS redémarre ensuite la barre des tâches, le service de rechargement Hyprland, le fond d'écran et le panneau de contrôle afin que chaque surface utilise le thème personnalisé.

## Supprimer un thème personnalisé

1. Ouvrez **Centre de Contrôle → Apparence → Thèmes → Thèmes Personnalisés**.
2. Sélectionnez le thème et appuyez sur `d`.
3. Confirmez avec `y`, ou appuyez sur `n` ou `Esc` pour annuler.

Le répertoire du thème est supprimé de `~/.local/share/argvus/themes`. S'il s'agissait du thème actif, ARGVUS passe d'abord à **ARGVUS Dark**.

## Profils des versions précédentes

Les versions précédentes exportaient des *profils de thèmes* (`profile.tar.gz`), une copie des paramètres d'apparence. La première fois qu'une page Thèmes s'ouvre, chaque profil dans `~/.local/share/argvus/themes/<name>/` est converti en un thème créé à partir de son thème de base, avec sa couleur de surlignage et son fond d'écran. Le profil original, ainsi que l'ancienne liste `custom-themes.json`, sont déplacés vers `~/.config/argvus/data/backups/theme-profiles-<seconds>/`, et la ligne de statut indique la conversion. Un profil dont le thème de base n'est pas installé est uniquement déplacé vers la sauvegarde.

## Partager un thème

Un thème exporté est une archive normale. Copiez-le sur un autre ordinateur qui exécute ARGVUS et importez-le là-bas.

## Dépannage

- **L’archive n’est pas listée.** Elle doit se trouver directement dans votre répertoire personnel, se terminer par `.tar.gz` et contenir `<id>/theme.toml`.
- **L’importation est refusée.** La ligne de statut indique pourquoi : un thème officiel avec le même identifiant, un fond d’écran manquant, un manifeste qui ne se charge pas ou un composant qui ne rend pas.
- **L’exportation est refusée.** Le nom doit contenir au moins une lettre ou un chiffre, et le thème actif doit être installé.
