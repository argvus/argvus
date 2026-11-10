---
title: Projets
description: Le sélecteur de projet et la configuration des projets, via les projets argvus.
---
`argvus projects` répertorie et ouvre vos projets. C'est une commande en cours du package `argvus`, et `SUPER + O` ouvre le sélecteur.

## Commandes

```text
argvus projects                    pick a project (SUPER + O)
argvus projects open SELECTOR      open a project by position, full path or unique name
argvus projects add [--root] DIR   add a project, or a root holding projects
argvus projects remove DIR         remove a project or root
argvus projects list               print configured roots and paths
argvus projects pick [PROMPT]      print the path picked from the list, for other tools
```
## Qu'est-ce qu'un projet

Un projet est un répertoire répertorié dans la configuration (`/projects/paths`), ou un sous-répertoire d'une racine répertoriée dans `/projects/roots`. Au maximum neuf projets sont listés : les numéros des raccourcis `SUPER + CTRL + N`. Ouvrir un projet concentre le workspace Hyprland correspondant. La première fois qu'un projet s'ouvre, ARGVUS démarre un terminal et l'éditeur défini dans `/projects/editor` à l'intérieur.

La configuration est lue et écrite via `argvus config`, donc le Centre de Contrôle et le shell voient la même liste.

## Workspaces de projet et la barre des tâches

Les projets utilisent des espaces de travail Hyprland ordinaires nommés, et non des espaces de travail spéciaux. `SUPER + O` montre le numéro du projet, le répertoire et le raccourci direct `SUPER + CTRL + N`. Les deux points d'entrée se concentrent sur le même espace de travail. Seuls les espaces de travail ouverts apparaissent dans la barre des tâches, avec une étiquette `Project: name`, un état actif et une info-bulle contenant le chemin complet et le raccourci. Cliquez sur le bouton de l'espace de travail pour y revenir. La barre des tâches suit le moniteur de l'espace de travail.

Le chemin du répertoire canonique détermine l'identité stable d'un espace de travail ; changer l'ordre de la liste modifie le raccourci, pas l'espace de travail. Les projets ayant le même nom de répertoire restent séparés, et les liens symboliques vers le même répertoire sont dédupliqués. Le lancement du terminal et de l'éditeur cible explicitement l'espace de travail du projet. Les éditeurs qui réutilisent un processus/fenêtre existant peuvent conserver le comportement de l'espace de travail propre à cette application.

Les espaces de travail ouverts avant la mise à jour conservent leurs fenêtres existantes et leurs noms natifs. L'ouverture d'un projet après la mise à jour utilise sa nouvelle identité dérivée du chemin ; fermez les anciennes fenêtres de projet lorsque cela est pratique. Aucune association ou déplacement automatique n'est tenté car un ancien nom n'identifie pas le répertoire de manière fiable. En cas de noms de répertoires en double, utilisez le sélecteur, un numéro ou le chemin canonique complet plutôt qu'un nom ambigu.

L'ouverture d'un projet régénère `$XDG_RUNTIME_DIR/argvus/project-workspaces.json` de manière atomique à partir de la liste de projets configurée et des traductions. Il s'agit des métadonnées de session pour les étiquettes et infobulles des espaces de travail, et non de la configuration ; ne l'éditez pas. Rouvrez un projet après avoir modifié la liste ou la langue pour actualiser ses étiquettes. Mettez à jour à la fois `argvus` et `argvus-waybar`, ainsi que la configuration `argvus-taskbar`, pour cette intégration.
