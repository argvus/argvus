---
title: Capteurs de température
description: Aperçu de la température en direct pris en charge par JSON lm_sensors.
---
`argvus sensors` ouvre le panneau Ratatui en cours d'exécution utilisé par le popup de température CPU de la barre des tâches. Il exécute `sensors -j`, regroupe les relevés par les noms de puce et les étiquettes rapportés par le matériel, et se met à jour toutes les deux secondes. La surveillance GPU est gérée par `argvus system-monitor --gpu`.

Les relevés utilisent Celsius. Le panneau affiche `max`, `crit`, et l'état d'alarme lorsque lm_sensors les fournit. Un indicateur n'est affiché que lorsqu'une limite positive est disponible. Les `lm_sensors` manquants, les échecs de commande, le JSON invalide et les relevés vides restent visibles en tant qu'états récupérables ; le sondage reprend au cycle suivant.

Appuyez sur `q` ou `Esc` pour fermer. La barre des tâches ouvre le panneau dans Kitty via `argvus tui-terminal` ; son titre d'onglet inférieur localisé montre l'indication de fermeture. Cette commande possède les détails de température du CPU ; l'icône de température GPU ouvre le moniteur btop uniquement GPU. Le package de la barre des tâches dépend déjà de `lm_sensors`, donc aucun package ou dépendance supplémentaire n'est nécessaire.
