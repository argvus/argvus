---
title: Contribuer
description: Limites de contribution pour les projets et la documentation ARGVUS.
---
Tracez une fonctionnalité à travers tous les dépôts consommateurs avant de la modifier. Préservez les contrats d'état partagé, la propriété des paquets, la propriété des services et les deux catalogues de localisation.

`argvus-config` est l'unique rédacteur de `data/generated/`. Ajoutez ou modifiez un fichier consommateur là-bas, dans `crates/argvus-config/src/project.rs` et ses sous-modules sous `project/` (fichiers Hyprland dans `project/hyprland_projection.rs`), pas dans un assistant shell. Si une application consommatrice ne peut pas lire `config.json`, ajoutez un adaptateur dans le paquet propriétaire et appelez-le depuis l'orchestrateur après le commit canonique. Supprimez `data/generated/`, exécutez `argvus config apply` et confirmez que l'arbre est reconstruit octet par octet à partir de `config.json`. Validez séparément la source, le payload du paquet et le comportement en session active ; une simple compilation de la source ne prouve pas qu'une session Wayland installée utilise les nouveaux fichiers.
