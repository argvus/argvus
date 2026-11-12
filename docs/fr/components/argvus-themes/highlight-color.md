---
title: Personnalisation de la couleur de surbrillance
description: Changez la couleur de surbrillance ARGVUS indépendamment du thème sélectionné, et revenez au paramètre par défaut du thème.
---
La **couleur de surbrillance** est l'accent utilisé pour les bordures de mise au point, la sélection et les états actifs sur le bureau. Chaque thème a une couleur de surbrillance par défaut, mais vous pouvez la remplacer par n'importe quelle couleur sans changer le thème.

## Changer la couleur de surbrillance

1. Ouvrez **Centre de contrôle → Apparence → Couleur de surbrillance**.
2. Choisissez **Modifier la couleur de surbrillance**. La valeur actuelle est affichée à côté de l'option.
3. Tapez une couleur RGB à six chiffres, avec ou sans le préfixe `#`. Par exemple, `3590BD` et `#3590BD` sont tous deux acceptés.
4. Appuyez sur `Enter` pour appliquer.

Vous pouvez également coller une couleur dans le champ. ARGVUS n'accepte le texte collé que s'il s'agit d'une valeur RGB à six chiffres valide. Les formes abrégées comme `#123` et les valeurs contenant des caractères non hexadécimaux sont rejetées avec un message d'erreur, et la couleur n'est pas modifiée.

La nouvelle couleur est appliquée immédiatement au bureau et est enregistrée dans votre configuration ARGVUS.

## Revenir à la valeur par défaut du thème

Pour revenir à la couleur définie par le thème sélectionné, ouvrez **Centre de contrôle → Apparence → Couleur de surbrillance** et choisissez **Réinitialiser par défaut du thème**.

## Comment la couleur de surbrillance interagit avec les thèmes

- Une couleur de surbrillance que vous choisissez est une modification indépendante. Elle reste lorsque vous changez de thème, donc la couleur que vous avez choisie reste la même.
- Un thème que vous appliquez après **Réinitialiser par défaut du thème** prend de nouveau le contrôle de la couleur de surbrillance et utilise sa propre valeur par défaut.
- L’exportation du thème actif enregistre la couleur de surbrillance qui est active lors de l’exportation comme accent de la palette du nouveau thème. Voir [Importer et exporter des thèmes](/docs/components/argvus-themes/import-and-export/).

## Où la valeur est stockée

La couleur de surlignage fait partie de votre configuration ARGVUS, dans `$XDG_CONFIG_HOME/argvus/config.json` (généralement `~/.config/argvus/config.json`). Ne modifiez pas les fichiers générés sous `$XDG_CONFIG_HOME/argvus/data/generated` : ARGVUS les réécrit à partir de la configuration.
