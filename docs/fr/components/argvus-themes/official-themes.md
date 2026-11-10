---
title: Thèmes officiels
description: Le catalogue officiel des thèmes ARGVUS, les couleurs de surbrillance par défaut, et comment choisir un thème ainsi que son mode Collant ou Espacé.
---
Cette page répertorie les thèmes officiels disponibles sous forme de packages de thèmes et explique comment en sélectionner un. Pour les installer, voir [Installer des thèmes](/docs/components/argvus-themes/installing-themes/). 

## Sélectionner un thème

1. Ouvrez **Panneau de configuration → Apparence → Thèmes**.
2. Choisissez **Thèmes officiels**.
3. Choisissez **Sombre** ou **Clair**.
4. Choisissez la famille de thèmes. Le thème est appliqué dès que vous le sélectionnez.

Le thème sélectionné est appliqué à l’ensemble du bureau en une seule fois : la barre des tâches, les panneaux, les notifications, les fenêtres et le fond d’écran en cours changent de couleur sur place, sans écran de chargement et sans redémarrage. ARGVUS conserve votre sélection de fond d’écran sauf si le thème est appliqué avec son propre fond d’écran associé.

Vous pouvez également appuyer sur `SUPER + SHIFT + T` pour ouvrir le sélecteur de thèmes depuis le clavier. Utilisez les flèches pour naviguer dans les catégories, `Enter` pour choisir, et `Esc` pour fermer.

## Collant et Espacé

Chaque thème fonctionne dans deux modes. Le mode est indépendant du thème : changer le thème conserve le mode actuel, et changer le mode conserve le thème actuel.

| Mode | Espaces entre fenêtres | Espace extérieur | Arrondi des fenêtres | Marge de la barre des tâches et du shell |
| --- | ---: | ---: | ---: | ---: |
| Collant | `2` | `0` | `0` | près du bord |
| Espacé | `10` | `18` | `4` | `18` |

Pour changer le mode, ouvrez **Centre de contrôle → Apparence → Mode** et choisissez **Collant** ou **Espacé**. Vos ajustements manuels de ces valeurs de disposition restent en vigueur jusqu'à la prochaine sélection explicite de thème.

## Thèmes sombres

| Thème | Package | Surbrillance par défaut |
| --- | --- | --- |
| One Dark | `argvus-theme-one-dark` | `#61AFEF` |
| Dracula | `argvus-theme-dracula` | `#BD93F9` |
| Silver Dark | `argvus-theme-silver-dark` | `#595959` |
| Slate Dark | `argvus-theme-slate-dark` | `#7391A5` |
| Universe | `argvus-theme-universe` | `#EEEEEE` |
| Gruvbox High Dark | `argvus-theme-gruvbox-high-dark` | `#D79921` |
| Gruvbox Dark | `argvus-theme-gruvbox-dark` | `#D4BE98` |
| Rosé Pine | `argvus-theme-rose-pine` | `#C4A7E7` |
| Tokyo Night | `argvus-theme-tokyo-night` | `#7AA2F7` |
| Solitude | `argvus-theme-solitude` | `#798186` |
| Sunset | `argvus-theme-sunset` | `#E2BE8A` |
| Matrix | `argvus-theme-matrix` | `#82FB9C` |
| Monokai Dark | `argvus-theme-monokai-dark` | `#78DCE8` |
| Lady Mist | `argvus-theme-lady-mist` | `#354B78` |

## Thèmes clairs

| Thème | Package | Mise en évidence par défaut |
| --- | --- | --- |
| GitHub Clair | `argvus-theme-github-light` | `#0969DA` |
| Solarized Clair | `argvus-theme-solarized-light` | `#268BD2` |
| Nord Clair | `argvus-theme-nord-light` | `#5E81AC` |
| One Clair | `argvus-theme-one-light` | `#4078F2` |
| Everforest Clair | `argvus-theme-everforest-light` | `#3A94C5` |
| Catppuccin Latte | `argvus-theme-catppuccin-latte` | `#1E66F5` |
| Gruvbox Clair | `argvus-theme-gruvbox-light` | `#458588` |
| Aquarelle | `argvus-theme-watercolor` | `#1A639C` |
| Obsidian Clair | `argvus-theme-obsidian-light` | `#334A78` |

## Thèmes intégrés

**ARGVUS Sombre** et **ARGVUS Clair** sont fournis par `argvus-appearance` et sont toujours disponibles sous **Thèmes officiels**. ARGVUS Sombre utilise la mise en évidence par défaut `#3590BD` ; ARGVUS Clair utilise `#181818`.

## Ce qu'un thème contrôle

Un thème définit la palette et le traitement des surfaces utilisés par les composants du bureau, la couleur de surlignage par défaut, l'association de fond d'écran par défaut ainsi que les valeurs par défaut de transparence et de flou. Les composants du bureau lisent cet état plutôt que de stocker leur propre copie, de sorte qu'un changement de thème affecte le compositeur, la barre des tâches, le Panneau de configuration, l'écran de verrouillage, les notifications et les applications en même temps.

La couleur de surlignage par défaut n'est que le point de départ. Vous pouvez la remplacer par n'importe quelle couleur à six chiffres, voir [Personnalisation de la couleur de surlignage](/docs/components/argvus-themes/highlight-color/).
