---
title: Questions fréquemment posées
description: Questions courantes sur ARGVUS, son architecture et son utilisation quotidienne.
---
Cette page répond aux questions courantes sur ce qu'est ARGVUS, comment il est distribué et où le configurer. Pour des procédures détaillées, suivez les liens dans chaque réponse.

## Qu'est-ce qu'ARGVUS ?

ARGVUS est un environnement de bureau modulaire pour Wayland et Arch Linux, construit autour de Hyprland. Ce n'est pas un environnement de bureau indépendant de Hyprland ; c'est une expérience de bureau assemblée sur Hyprland et adaptée à Arch Linux. ARGVUS coordonne la session graphique, les surfaces de shell, les applications de paramètres, l'apparence, les services de bureau et les outils intégrés via une collection de paquets.

C'est plus qu'une configuration Hyprland ou une collection de dotfiles personnels : elle fournit un point d'entrée de session pris en charge, une configuration et un état d'apparence partagés, ainsi qu'une propriété définie pour des composants tels que la barre des tâches, le Centre de Contrôle, les notifications, le réseau et les affichages. Voir l'[introduction](/docs/introduction/) pour un aperçu de la manière dont les éléments s'articulent.

## ARGVUS est-il gratuit ?

Oui. ARGVUS est gratuit à télécharger et à utiliser, et ses paquets de projet principaux sont distribués en tant que logiciel libre sous la [GNU General Public License version 3](https://www.gnu.org/licenses/gpl-3.0.html). Cela donne aux utilisateurs la liberté d'exécuter, d'étudier, de modifier et de redistribuer le logiciel couvert selon les termes de la licence.

ARGVUS est distribué sous forme de paquets signés Arch Linux. Le logiciel lui-même n'a pas de prix d'achat, mais le téléchargement des paquets peut tout de même entraîner des frais Internet ou d'hébergement habituels. Les composants et applications fournis par des tiers peuvent avoir leurs propres licences.

## ARGVUS est-il une distribution Linux ?

Non. ARGVUS est la couche de l'environnement de bureau installée par-dessus Arch Linux. Arch Linux fournit le système d'exploitation, le noyau, le gestionnaire de paquets et les paquets de base ; Hyprland fournit la composition Wayland ; ARGVUS coordonne la session de bureau et ses composants intégrés.

Voir [installation](/docs/getting-started/installation/) pour le modèle d'installation pris en charge et [Ce qu'ARGVUS est — et n'est pas](/docs/introduction/#what-argvus-isand-is-not) pour la distinction architecturale.

## Pourquoi un environnement de bureau au lieu d'une distribution ?

Garder ARGVUS comme environnement de bureau offre aux utilisateurs plus de contrôle sur le reste de leur système. Une distribution choisit et maintient normalement une base complète, incluant le noyau, la sélection de paquets, les dépôts, les paramètres par défaut et les politiques système. ARGVUS se concentre sur l'expérience du bureau tout en laissant ces décisions plus larges à l'utilisateur et à l'installation sous-jacente d'Arch Linux.

Ce modèle maintient également le projet modulaire : la gestion de session, la configuration de Hyprland, l'apparence, la barre des tâches, les paramètres et les providers peuvent être développés et empaquetés séparément tout en fonctionnant ensemble comme un seul bureau.

## Quels systèmes sont supportés ?

La base actuellement supportée est Arch Linux. ARGVUS est empaqueté pour Arch Linux et dépend de Wayland, Hyprland et des composants système décrits dans les [installation requirements](/docs/getting-started/installation/requirements/)].

D'autres distributions peuvent fournir certains des mêmes composants en amont, mais cela ne signifie pas que l'ensemble complet de packages ARGVUS ou l'intégration de session y soient pris en charge.

## Comment installer ARGVUS ?

Suivez le [guide d'installation](/docs/getting-started/installation/)]. Il explique les exigences, comment configurer le dépôt de packages ARGVUS signé, comment installer l'ensemble de packages et comment démarrer la première session.

ARGVUS est livré comme un ensemble coordonné de packages Arch. Le package `argvus` est la cible principale de l'installation et inclut les composants de bureau qu'il coordonne.

## Comment modifier les paramètres d'ARGVUS ?

Utilisez le **Centre de Contrôle** pour la configuration persistante du bureau, y compris les thèmes, les fonds d'écran, les effets, la disposition, les polices, le clavier, la saisie, la langue, la région et les applications par défaut. Les moyens les plus efficaces pour l'ouvrir sont le raccourci `SUPER + Alt + C`, ou le lanceur avec `SUPER + D` : tapez **ARGVUS Centre de Contrôle**. Vous pouvez également taper `argvus` pour lister les applications ARGVUS.

Comme option supplémentaire, vous pouvez l'ouvrir depuis un terminal avec :

```sh
argvus --control-center
```
Vous pouvez également ouvrir une zone spécifique avec `argvus control-center`, par exemple `argvus control-center appearance themes`. Exécutez `argvus control-center --help` pour voir les routes disponibles dans la version installée.

Utilisez **Panneau de configuration** pour l’état actuel et les actions fréquentes telles que le volume, la luminosité, le réseau, les notifications, le contrôle de l’alimentation et de la session. Ce n’est pas une seconde base de données de configuration. Voir [Où configurer les choses](/docs/user-guide/where-to-configure/)] et [Centre de contrôle](/docs/components/argvus-control-center/)] pour la distinction.

## Où sont stockées les paramètres ARGVUS ?

La configuration appartenant à l'utilisateur est stockée sous `$XDG_CONFIG_HOME/argvus` (normalement `~/.config/argvus`). La configuration logique canonique y est maintenue, tandis que les fichiers dérivés pour Hyprland, Waybar et le widget de télémétrie, GTK, Qt6ct, Quickshell, Dunst, Hyprlock, Yazi, Superfile, les terminaux, les polices, les effets, les calendriers, l'entrée, le clavier, l'alimentation et les périphériques amovibles se trouvent sous `~/.config/argvus/data/generated/`. Cet arbre est écrit uniquement par `argvus-config` et reconstruit à partir de la configuration canonique ; `argvus-appearance` valide les changements d'apparence et ne réconcilie ensuite que les consommateurs externes qui ne peuvent pas lire la configuration canonique par eux-mêmes. Rofi fait exception : `argvus launcher` emballe sa configuration et ses thèmes, et les copies éditées par l'utilisateur se trouvent sous `~/.config/argvus/data/rofi/`.

Ne modifiez pas les fichiers générés comme méthode de configuration permanente. Utilisez le Control Center ou le chemin de remplacement natif documenté pour le composant que vous souhaitez personnaliser. Voir [configuration files](/docs/reference/configuration-files/) et [configuration and state](/docs/developer-guide/architecture/configuration-and-state/).

## Que dois-je faire si une modification n'est pas appliquée ?

Confirmez d'abord que la modification a été enregistrée ou appliquée sur la page propriétaire. Vérifiez ensuite si le composant affecté nécessite un rechargement de session ou de service. Une modification validée est projetée dans les fichiers consommateurs par `argvus-config`, mais la projection de ces fichiers ne garantit pas que chaque consommateur en cours d'exécution les a rechargés. `argvus sessionctl reload` applique une génération déjà validée depuis le manifeste de projection au lieu de projeter à nouveau, et saute le rechargement graphique lorsque l'état canonique n'a pas changé.

Utilisez le [guide de dépannage](/docs/user-guide/troubleshooting/) pertinent pour l'installation, le démarrage de session, les graphiques, les écrans, Hyprland, les thèmes et les fonds d'écran. Lors de la signalisation d'un problème, incluez le composant, le paramètre modifié, la commande ou la page utilisée et la sortie de la commande d'état ou de diagnostic pertinente.

## Où puis-je en apprendre davantage ?

- [Première configuration](/docs/user-guide/getting-started/) pour le flux de première session recommandé.
- [Guide de l'utilisateur](/docs/user-guide/) pour les instructions spécifiques aux fonctionnalités.
- [Référence](/docs/reference/) pour les commandes, chemins, services et contrats de configuration.
- [Guide du développeur](/docs/developer-guide/) pour l'architecture, la propriété des paquets et les flux de travail de contribution.
