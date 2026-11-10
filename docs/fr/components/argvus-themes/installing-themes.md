---
title: Installation de thèmes
description: Installez les paquets du thème officiel ARGVUS avec pacman.
---
Les thèmes officiels sont publiés sous forme de packages Arch Linux. Chaque thème est son propre package nommé `argvus-theme-<THEME_NAME>`.

## Où obtenir les thèmes

La source de chaque thème officiel se trouve dans l'organisation de projet ARGVUS :

- Chaque thème est un dépôt nommé `argvus-theme-<THEME_NAME>` (par exemple `argvus-theme-dracula`), construit comme un package portant le même nom.

Les packages de thèmes dépendent de `argvus-appearance`. Installez ARGVUS d'abord, comme décrit dans [Installation](/docs/getting-started/installation/), afin que le composant d'apparence soit présent.

## Installer un seul thème

Pour installer tous les thèmes officiels en une seule fois, installez le métapackage `argvus-themes` :

```bash
sudo pacman -S argvus-themes
```
Cela dépend de tous les packages `argvus-theme-*` publiés. ARGVUS Dark et ARGVUS Light sont déjà expédiés par `argvus-appearance`.

Installez un thème avec son nom de package :

```bash
sudo pacman -S argvus-theme-<THEME_NAME>
```
Par exemple :

```bash
sudo pacman -S argvus-theme-dracula
sudo pacman -S argvus-theme-rose-pine
```
Le paquet installe le manifeste du thème, `/usr/share/argvus/themes/<theme-id>/theme.toml` (son nom, sa catégorie, son fond d'écran et sa palette, avec le `overrides/` facultatif), et le fond d'écran du thème sous `/usr/share/backgrounds/argvus/`. Les fichiers de chaque composant sont générés à partir de la palette lorsque le thème est appliqué. Il n’écrit pas dans votre répertoire personnel et ne change pas votre thème actuel. Sélectionnez-le ensuite dans le Centre de contrôle, voir [Thèmes officiels](/docs/components/argvus-themes/official-themes/).

## Installer plusieurs thèmes

Il n’existe pas de méta-paquet qui installe tous les thèmes. Installez les thèmes que vous souhaitez en une seule commande :

```bash
sudo pacman -S argvus-theme-dracula argvus-theme-sunset
```
## Packages disponibles

Les packages ci-dessous sont les packages de thèmes officiels publiés aujourd'hui.

**Thèmes sombres**

| Package | Thème |
| --- | --- |
| `argvus-theme-one-dark` | One Dark |
| `argvus-theme-dracula` | Dracula |
| `argvus-theme-silver-dark` | Silver Dark |
| `argvus-theme-slate-dark` | Slate Dark |
| `argvus-theme-universe` | Universe |
| `argvus-theme-gruvbox-high-dark` | Gruvbox High Dark |
| `argvus-theme-gruvbox-dark` | Gruvbox Dark |
| `argvus-theme-rose-pine` | Rosé Pine |
| `argvus-theme-tokyo-night` | Tokyo Night |
| `argvus-theme-solitude` | Solitude |
| `argvus-theme-sunset` | Sunset |
| `argvus-theme-matrix` | Matrix |
| `argvus-theme-monokai-dark` | Monokai Dark |

**Thèmes clairs**

| Package | Thème |
| --- | --- |
| `argvus-theme-github-light` | GitHub Clair |
| `argvus-theme-solarized-light` | Solarized Clair |
| `argvus-theme-nord-light` | Nord Clair |
| `argvus-theme-one-light` | One Clair |
| `argvus-theme-everforest-light` | Everforest Clair |
| `argvus-theme-catppuccin-latte` | Catppuccin Latte |
| `argvus-theme-gruvbox-light` | Gruvbox Clair |
| `argvus-theme-watercolor` | Aquarelle |

Les thèmes intégrés **ARGVUS Dark** et **ARGVUS Light** n'ont pas besoin d'un paquet séparé ; ils sont fournis avec `argvus-appearance`.

## Vérifier ce qui est installé

Liste des paquets de thèmes installés avec :

```bash
pacman -Qs argvus-theme
```
Ouvrez le Centre de Contrôle et allez à **Apparence → Thèmes → Thèmes officiels** pour voir les thèmes actuellement disponibles dans le sélecteur. Un thème installé mais absent de cette liste est un problème d’emballage ; consultez la section de dépannage du guide [Themes](/docs/components/argvus-themes/).

## Ajouter votre propre thème

Un thème est un manifeste, donc un thème personnel n’a pas besoin de paquet. Créez `~/.local/share/argvus/themes/<id>/theme.toml`, où le nom du répertoire est l’ID du thème (lettres minuscules, chiffres et tirets simples) :

```toml
schema = 2
id = "mist"
name = "Mist"
category = "dark"
wallpaper = "/usr/share/backgrounds/argvus/argvus-dark.jxl"

[palette]
bg = "#101418"
fg = "#D8DEE9"
accent = "#88C0D0"
```
Seuls `bg`, `fg` et `accent` sont requis : toutes les autres couleurs en découlent par défaut, et le manifeste en définit une seulement pour la modifier. Vérifiez-le, puis appliquez-le comme n'importe quel autre thème :

```bash
argvus theme check ~/.local/share/argvus/themes/mist
argvus theme apply mist
```
`argvus theme list` affiche vos thèmes à côté des thèmes officiels (un thème officiel avec le même ID l'emporte). Vos thèmes apparaissent dans le menu des thèmes (`SUPER + Shift + T`) ; l'écran de bienvenue avant connexion ne peut pas lire votre répertoire personnel, donc il conserve le dernier thème officiel que vous avez appliqué pendant qu'un de vos thèmes est actif.

Les copies par thème que les anciennes versions d'ARGVUS écrivaient sous `~/.config/argvus/data/<app>/themes/` ne sont plus lues. Le premier changement de thème après la mise à jour les déplace vers `~/.config/argvus/data/backups/theme-copies-<time>/` ; pour conserver un fichier que vous avez modifié, mettez-le dans un thème personnel sous `overrides/` et nommez-le dans la table `[overrides]` du manifeste.

## Supprimer un thème

Supprimez un seul paquet de thème avec :

```bash
sudo pacman -R argvus-theme-<THEME_NAME>
```
La suppression d’un package de thème ne supprime pas vos profils importés. Si le thème supprimé est celui actuellement appliqué, sélectionnez un autre thème depuis le Centre de Contrôle avant de le supprimer.
