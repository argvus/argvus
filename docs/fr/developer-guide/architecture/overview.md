---
title: Aperçu de l'architecture
description: Comment ARGVUS forme un environnement de bureau intégré.
---
ARGVUS possède cinq couches d'exécution :

1. points d'entrée et dispatch dans `argvus` ;
2. cycle de vie de la session dans `argvus-session` ;
3. compositor et surfaces du shell dans `argvus-hyprland`, `argvus-waybar`, `argvus-taskbar` et `argvus-control-panel` ;
4. applications utilisateur telles que le Centre de Contrôle, le calendrier et le moniteur système ;
5. fournisseurs et ressources pour l'affichage, le réseau, l'alimentation, l'apparence, les comptes et les notifications.

L'état partagé connecte ces couches. `argvus-config` contient le document canonique et est le seul à écrire dans `data/generated/`, donc une modification de thème ou d'effets régénère les fichiers utilisés par plusieurs packages en une seule transaction ; les services de session fournissent la gestion du cycle de vie sans prendre en charge l'implémentation de chaque composant.

## Limites de responsabilité

| Couche | Responsabilité principale | Exemples |
| --- | --- | --- |
| Points d’entrée et répartition | Démarrer les applications de bureau et résoudre les profils ARGVUS. | `argvus`, `argvus-app-profiles` |
| Cycle de vie de la session | Préparer l’environnement, lancer Hyprland, synchroniser l’environnement de service utilisateur et gérer l’arrêt de la session. | `argvus-session`, `argvus sessionctl` |
| Compositeur et shell | Définir le comportement des fenêtres et les surfaces de bureau visibles. | `argvus-hyprland`, `argvus-waybar`, `argvus-taskbar`, `argvus-control-panel` |
| Configuration | Posséder `config.json`, le modèle de chemin et tous les fichiers consommateurs générés. | `argvus-config` |
| Applications | Fournir les paramètres et les flux de travail de bureau centrés. | Centre de contrôle, calendrier, lanceur, moniteur système et périphériques amovibles |
| Fournisseurs et ressources | Mettre en œuvre les opérations de domaine et les ressources visuelles partagées. | affichage, réseau, alimentation, notifications, apparence, polices et fonds d’écran |

Le paquet `argvus` est la cible d'installation et le coordinateur, mais ce n'est pas un runtime monolithique. Les paquets spécialisés gardent la propriété de leurs binaires, de leur configuration et de leurs services. Le gestionnaire de session les démarre au moment approprié dans la session graphique sans absorber leur implémentation.

Cette distinction est importante lors du débogage : un échec de session appartient d'abord à `argvus-session` ; un échec de la barre des tâches appartient à `argvus-taskbar`/`argvus-waybar` ; et un échec de thème ou de fond d'écran généré appartient au chemin de configuration de l'apparence et de la session.
