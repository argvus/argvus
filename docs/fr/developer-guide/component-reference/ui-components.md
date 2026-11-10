---
title: Composants d'interface utilisateur
description: Packages d'interface utilisateur ARGVUS.
---
Les packages UI incluent `argvus-control-panel`, `argvus-taskbar` et `argvus-launcher`, ainsi que les sous-commandes `argvus` `argvus control-center` (assistant dans `argvus`), `argvus taskbar-calendar`, `argvus widget-telemetry`, `argvus terminal`, `argvus system-monitor`, `argvus removable-devices`, `argvus focus` et `argvus clock`. Le calendrier, Focus et l'horloge utilisent le crate partagé `argvus-taskbar-popup` pour le positionnement et la fermeture du layer-shell ; Focus expose également son lecteur de statut Waybar via la route `argvus focus status` en cours d'exécution.

Leur comportement côté utilisateur est documenté par fonctionnalité ; cette page enregistre la propriété des packages.
