---
title: Machines virtuelles
description: Exécutez ARGVUS avec du matériel graphique virtuel.
---
`argvus start` détecte la virtualisation et exporte les paramètres de compatibilité pour les sessions affectées, y compris les rendus logiciels et les modes de secours pour le curseur/les modificateurs lorsque requis par l'environnement détecté.

Si une machine virtuelle échoue toujours à rendre l'affichage, inspectez l'environnement de la session et les journaux avant d'ajouter une substitution. Voir [dépannage graphique](/docs/user-guide/troubleshooting/graphics-and-virtual-machines/).

Lorsque la virtualisation est détectée, `argvus start` exporte des variables de compatibilité pour la session, y compris les rendus logiciels, le curseur logiciel et les reprises de modificateurs. Cela fait du chemin de connexion normal la première chose à essayer ; ne pas ajouter de substitution permanente simplement parce que le bureau s'exécute dans une VM.

Si la détection n'est pas suffisante, utilisez une substitution utilisateur Hyprland comme solution de secours diagnostique :

```sh
mkdir -p ~/.config/argvus/data/hypr
nano ~/.config/argvus/data/hypr/user.lua
```
Ajoutez ceci uniquement lorsque la pile graphique de la VM l'exige :

```lua
hl.env("LIBGL_ALWAYS_SOFTWARE", "1")
```
Déconnectez-vous et reconnectez-vous après avoir modifié l'environnement de la session. Confirmez la valeur active avec `echo $LIBGL_ALWAYS_SOFTWARE` ; le rendu logiciel peut rendre le bureau utilisable mais peut réduire les performances.
