---
title: Apparence
description: Configurer le langage visuel partagé par les composants ARGVUS.
---
Le système d'apparence applique un état de thème unique à Hyprland, GTK, Qt, Waybar, Quickshell, Rofi, Dunst, Hyprlock et aux applications ARGVUS.

## Familles de thèmes

Les vingt-trois familles ci-dessous sont définies par la charge utile du thème actuel. La colonne des couleurs indique la couleur par défaut de la bordure active/de l'accentuation ; les utilisateurs peuvent choisir une autre couleur RGB valide.

| Famille | Réinitialiser la couleur de surbrillance | Arrière-plan de base | Modes |
| --- | --- | --- | --- |
| One Dark | `#61AFEF` | `#282C34` | Collant, Espacé |
| Dracula | `#BD93F9` | `#282A36` | Collant, Espacé |
| ARGVUS Dark | `#3590bd` | `#111316` | Collant, Espacé |
| Dark Silver | `#595959` | `#111316` | Collant, Espacé |
| Dark Slate | `#7391a5` | `#2f3541` | Collant, Espacé |
| Dark Universe | `#eeeeee` | `#000000` | Collant, Espacé |
| ARGVUS Dark Gruvbox High | `#D79921` | `#282828` | Collant, Espacé |
| ARGVUS Dark Gruvbox | `#D4BE98` | `#282828` | Collant, Espacé |
| Lumière ARGVUS | `#181818` | `#f7f7f7` | Collant, espacé |
| Lumière GitHub | `#0969DA` | `#FFFFFF` | Collant, espacé |
| Latte à la catpuccine | `#1E66F5` | `#EFF1F5` | Collant, espacé |
| ARGVUS Light Gruvbox | `#458588` | `#FBF1C7` | Collant, espacé |
| Pin Rosé | `#C4A7E7` | `#191724` | Collant, espacé |
| Tokyo-Nuit | `#7AA2F7` | `#1A1B26` | Collant, espacé |
| Solitude | `#798186` | `#101315` | Collant, espacé |
| Coucher de soleil | `#E2BE8A` | `#0F0F0F` | Collant, espacé |
| Matrice | `#82FB9C` | `#0B0C16` | Collant, espacé |
| Monokai Sombre | `#78DCE8` | `#2D2A2E` | Collant, espacé |
| Dame Brume | `#354B78` | `#131313` | Collant, espacé |
| Une lumière | `#4078F2` | `#FAFAFA` | Collant, espacé |
| Lumière de la forêt éternelle | `#3A94C5` | `#FDF6E3` | Collant, espacé |
| Lumière solarisée | `#268BD2` | `#FDF6E3` | Collant, espacé |
| Lumière Nord | `#5E81AC` | `#ECEFF4` | Collant, espacé |
| Aquarelle | `#1A639C` | `#E9E3D3` | Collant, espacé |

La couleur affichée dans le tableau est la valeur écrite dans `config.json` lorsque cette famille est sélectionnée ou lorsque **Réinitialiser sur le thème par défaut** est utilisé. Un accent RGB à six chiffres choisi manuellement s'applique jusqu'au prochain changement de thème, qui le remplace par le thème par défaut sélectionné.

**Collant** est la disposition compacte : les fenêtres en mosaïque utilisent de petits espaces internes, les espaces extérieurs sont nuls et la barre des tâches est proche du bord de l'écran. **Espacé** ajoute de plus grands espaces, des coins arrondis, des ombres et des marges autour de la barre des tâches et des surfaces du shell.

Les valeurs de disposition `float` existantes et les identifiants de thème `-float` sont reconnus comme des noms hérités pour Espacé. Les nouvelles valeurs de disposition et les identifiants de thème résolus utilisent `spaced` et `-spaced`.

Le mode change la géométrie et le traitement de surface ; il ne crée pas une sixième famille de couleurs. Voir [Themes](/docs/user-guide/appearance/) pour la sélection et [Desktop](/docs/user-guide/desktop/) pour le placement de la barre des tâches et des fenêtres.

- [Thèmes](/docs/user-guide/appearance/)
- [Papier peint](/docs/argvus-appearance/wallpapers/)
- [Polices et icônes](/docs/argvus-appearance/fonts-and-icons/)
- [Effets](/docs/argvus-appearance/effects/)

Le Centre de Contrôle expose les paramètres accessibles à l'utilisateur. L'implémentation est partagée par `argvus-appearance`, `argvus-session`, les composants du shell et les paquets d'assets. Voir [architecture de l'apparence](/docs/developer-guide/subsystems/appearance-and-themes/).

## Oscillation des fenêtres

Ouvrez **Bureau & Entrée → Hyprland → Animations → Fenêtres** et utilisez les
contrôles **Oscillation**. Recherchez `wobble`, ou ouvrez directement la page :

```sh
argvus control-center hyprland animations windows
```
Le balancement est désactivé par défaut. Ses valeurs par défaut ARGVUS sont rigidité `100`, amortissement `10`, masse `1`, intensité `1` et résolution du maillage `8`. La rigidité et la masse acceptent `0.0001–1000`, l'amortissement `0–1000`, l'intensité `0–2`, et le maillage un entier `2–32`. Une résolution de maillage plus élevée augmente le coût de rendu. Utilisez les touches fléchées pour ajuster les valeurs ou Entrée pour les saisir. **Restaurer les valeurs par défaut du balancement** modifie le brouillon ; **Appliquer** enregistre toutes les modifications d'animation en attente et recharge Hyprland une fois, sans redémarrer la session. Revenir dans Animations conserve le brouillon partagé.

Le commutateur global Animations supprime le balancement tout en conservant ses paramètres individuels. Les valeurs canoniques se trouvent sous `effects.wobble_*` dans l'état ARGVUS ; `hypr/state.lua` est généré et ne doit pas être modifié manuellement.

Cela nécessite Hyprland **0.57.0 ou une version plus récente**, exigé par la dépendance de paquet de `argvus-hyprland`. La version 0.56 n'est pas prise en charge pour cette configuration. Le maximum d'intensité suit la [source publiée 0.57.0 ](https://github.com/hyprwm/Hyprland/blob/v0.57.0/src/config/values/ConfigValues.cpp), qui la limite à `2`, même si une documentation plus récente peut indiquer `3`.
