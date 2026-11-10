---
title: Analyseur d'utilisation de disque ARGVUS
description: Parcourez l'utilisation du disque de manière interactive avec ncdu dans une fenêtre Kitty thématique.
---
Ouvrez **ARGVUS Disk Usage Analyzer** depuis le lanceur d'applications ou exécutez :

```sh
argvus du
```
L'analyseur ouvre une fenêtre Kitty dédiée et centrée et intègre l'interface réelle de ncdu à l'intérieur de son cadre ARGVUS. Il démarre dans `$HOME`. Modifiez le chemin du répertoire ou choisissez **Parcourir** pour ouvrir le gestionnaire de fichiers configuré. Avec Superfile, naviguez vers un répertoire et appuyez sur **Espace** pour le sélectionner ; `q` ou `Esc` annule. Les autres gestionnaires de fichiers configurés s'ouvrent normalement, mais peuvent ne pas renvoyer un répertoire sélectionné, donc entrez ou collez le chemin dans l'analyseur.

Utilisez Tab ou les touches fléchées pour naviguer entre le répertoire, les options et les actions. Le graphique, le pourcentage, le thème sombre et le classement des répertoires en premier sont activés par défaut. Les autres options d'analyse et d'affichage peuvent être activées ou désactivées avant de sélectionner **Analyser**. Dans les Résultats, utilisez les commandes clavier de ncdu ; `q` permet de revenir au formulaire et `Esc` arrête l'analyse. Appuyez sur `d` pour supprimer le fichier ou le répertoire sélectionné. ncdu demande une confirmation ; la suppression est permanente et utilise les autorisations de l'utilisateur actuel. Aucune élévation de privilèges n'est requise.

L'interface externe suit le thème ARGVUS actif et la locale. Kitty et ncdu sont nécessaires ; le package `argvus-devtools` fournit ncdu. Si Kitty ou ncdu est manquant, l'application signale la dépendance manquante.
