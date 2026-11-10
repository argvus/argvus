---
title: Emballage
description: Construire et inspecter les paquets ARGVUS Arch.
---
Chaque composant possède son emballage Arch dans son dépôt. Inspectez `packaging/archlinux/`, les fonctions du paquet et la liste des fichiers installés avant de documenter un contenu. Les valeurs par défaut du paquet sous `/usr/share/argvus` et la configuration de l'administrateur sous `/etc/argvus` ne sont pas interchangeables.

Pour la vérification à l'exécution, comparez le contenu du paquet avec `pacman -Ql <package>` et inspectez séparément les surcharges actives des utilisateurs.

Les constructions locales conservent les objets Cargo dans `build/cargo-target`, en dehors de l'arborescence source du paquet, afin qu'un `make build` répété ne recompile que ce qui a changé. `make clean` supprime ce répertoire, et la construction suivante commence à froid.

Ne déduisez pas le contenu à l'exécution uniquement à partir du nom du dépôt. Un paquet peut contenir des binaires, des services utilisateur, des valeurs par défaut `/usr/share/argvus`, la configuration administrateur `/etc/argvus` ou des ressources. Confirmez le contenu réel avec `PKGBUILD` et `pacman -Ql` du dépôt après installation.
