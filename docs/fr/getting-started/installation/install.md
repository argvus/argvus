---
title: Installer ARGVUS
description: Installer, mettre à jour, supprimer et configurer ARGVUS.
---
ARGVUS est distribué en tant que paquet Arch Linux via le dépôt officiel de paquets ARGVUS.

## Avant l'installation

Activez le dépôt officiel Arch Linux `multilib` avant d'installer ARGVUS. Modifiez `/etc/pacman.conf` et décommentez à la fois la section `[multilib]` et sa ligne `Include` :

```ini
[multilib]
Include = /etc/pacman.d/mirrorlist
```
Ensuite, synchronisez les bases de données des paquets et effectuez une mise à niveau complète du système :

```sh
sudo pacman -Syu
```
L'installateur rapide configure le dépôt ARGVUS, mais n'active pas `multilib` pour vous.

## Installation rapide

Sur une installation fraîche d'Arch Linux, vous pouvez installer ARGVUS directement depuis un TTY sans installer un autre environnement de bureau au préalable.

Exécutez :

```sh
curl -fsSL https://argvus.github.io/install.sh | sh
```
L'installateur :

1. Importe et signe localement la clé de signature du dépôt ARGVUS.
2. Configure le dépôt de packages ARGVUS.
3. Met à jour la base de données des packages.
4. Installe le package `argvus` et ses dépendances.

Après l'installation, déconnectez-vous et sélectionnez **ARGVUS** depuis votre gestionnaire d'affichage.

> **Remarque :** L'installateur rapide nécessite `curl`, `sudo` et les outils standard de gestion des packages d'Arch Linux.

## Configurer uniquement le dépôt

Pour configurer le dépôt de packages ARGVUS sans installer le bureau, exécutez :

```sh
curl -fsSL https://argvus.github.io/repo-install.sh | sh
```
Installez ARGVUS manuellement ensuite :

```sh
sudo pacman -Syu argvus
```
## Installation manuelle

Si vous préférez configurer le dépôt étape par étape, utilisez les commandes suivantes.

### 1. Télécharger la clé de signature du dépôt

```sh
curl -fsSLo /tmp/argvus.gpg https://argvus.github.io/packages/arch/argvus.gpg
```
### 2. Importer la clé de signature

```sh
sudo pacman-key --add /tmp/argvus.gpg
```
### 3. Signer localement la clé

```sh
ARGVUS_KEY="$(gpg --show-keys --with-colons /tmp/argvus.gpg | grep '^pub:' | head -n1 | cut -d: -f5)"
sudo pacman-key --lsign-key "$ARGVUS_KEY"
```
### 4. Configurer le dépôt ARGVUS

```sh
curl -fsSL https://argvus.github.io/packages/arch/argvus.conf \
  | sudo tee /etc/pacman.d/argvus.conf
```
Ajoutez la configuration du dépôt à `/etc/pacman.conf` :

```sh
echo "Include = /etc/pacman.d/argvus.conf" \
  | sudo tee -a /etc/pacman.conf
```
La configuration du dépôt active les dépôts ARGVUS, y compris `argvus` et `argvus-extras`, avec `SigLevel = Required TrustedOnly`. Pacman n'accepte les paquets que lorsque leurs signatures sont valides et approuvées par le trousseau local.

### 5. Installer ARGVUS

```sh
sudo pacman -Syu argvus
```
Après l'installation, déconnectez-vous et sélectionnez **ARGVUS** depuis votre gestionnaire d'affichage.

## Mise à jour

Mettez à jour ARGVUS avec le reste d'Arch Linux :

```sh
sudo pacman -Syu
```
Pour mettre à jour uniquement le package ARGVUS :

```sh
sudo pacman -S argvus
```

> **Recommandation :** Arch Linux recommande généralement une mise à niveau complète du système avec `sudo pacman -Syu` plutôt que de mettre à jour des paquets individuels.

## Supprimer ARGVUS

Supprimez ARGVUS tout en conservant les dépendances encore nécessaires par d'autres paquets installés :

```sh
sudo pacman -R argvus
```
Supprimez ARGVUS et les dépendances qui ne sont plus nécessaires pour aucun paquet installé :

```sh
sudo pacman -Rns argvus
```
### Supprimer le dépôt ARGVUS

Si vous souhaitez également supprimer le dépôt de packages, supprimez sa configuration :

```sh
sudo rm /etc/pacman.d/argvus.conf
```
Ensuite, supprimez la ligne correspondante de `/etc/pacman.conf` :

```text
Include = /etc/pacman.d/argvus.conf
```
Vous pouvez modifier le fichier avec :

```sh
sudo nano /etc/pacman.conf
```
### Supprimer la clé de signature

Pour supprimer la clé de signature du dépôt ARGVUS du porte-clés pacman, commencez par lister les clés :

```sh
sudo pacman-key --list
```
Ensuite, supprimez la clé ARGVUS avec son ID de clé :

```sh
sudo pacman-key --delete KEY_ID
```
Remplacez `KEY_ID` par l’ID de clé de signature ARGVUS.

> **Attention :** Supprimez cette clé uniquement si vous supprimez également le dépôt ARGVUS. Ne supprimez pas les clés encore nécessaires à d’autres dépôts.

## Configuration utilisateur

L’installation du paquet n’écrit pas de configuration dans `$HOME`. ARGVUS lit les valeurs par défaut du paquet depuis `/usr/share/argvus` et fonctionne sans les copier dans votre répertoire personnel.

La configuration d’exécution suit cette priorité générale :

```text
~/.config/argvus/data/generated/<app>  ->  ~/.config/argvus/data/<component>/<app>  ->  $XDG_CONFIG_HOME/<app>  ->  /usr/share/argvus/<app>  ->  upstream defaults
```
`argvus-config` est le seul composant qui écrit `~/.config/argvus/data/generated/`, et cette projection est la couche active : une copie gérée ou native ne prend effet que tant que le fichier généré correspondant est absent.

La configuration gérée par ARGVUS se trouve sous `~/.config/argvus/`. Une substitution optionnelle `$XDG_CONFIG_HOME/<app>` reste disponible pour les applications qui prennent en charge une configuration utilisateur native.

Le répartiteur `argvus --setup` est facultatif et sert pour des substitutions manuelles explicites, pas pour la récupération. Si `~/.config/argvus` est supprimé, `SUPER + SHIFT + R` recrée le profil canonique et l'état d'exécution généré via le pipeline de rechargement normal.

```sh
argvus --setup --copy foot-tui
argvus --setup --copy kitty-tui
argvus --setup --copy-all
```
`--copy-all` est une liste blanche explicite des fichiers de profil TUI pris en charge ; elle ne clone pas `/usr/share/argvus` et ne copie pas les scripts, services, ressources, fichiers générés ou `config.json`. Exécutez `argvus --setup --help` pour voir les options actuelles. Une fois copiés, les fichiers deviennent des remplacements appartenant à l'utilisateur et les mises à jour du paquet ne les remplacent pas.

Pour remplacer une configuration copiée existante, utilisez `--force`. ARGVUS crée une sauvegarde horodatée avant de la remplacer :

```sh
argvus --setup --copy <app> --force
```
## Remplacements utilisateur Hyprland

Pour le guide complet des remplacements Hyprland, y compris la différence entre une configuration native complète et des fragments ARGVUS incrémentiels, voir [Hyprland overrides](/docs/argvus-hyprland/hyprland-overrides/).

```sh
mkdir -p ~/.config/argvus/data/hypr
nano ~/.config/argvus/data/hypr/user.lua
```
### Solution de repli pour machine virtuelle

Certaines machines virtuelles peuvent rencontrer des problèmes de rendu avec Hyprland, Kitty ou Quickshell. ARGVUS détecte les machines virtuelles utilisant le pilote graphique `vmwgfx` et active le rendu logiciel si nécessaire.

Si la détection ne fonctionne pas sur votre machine virtuelle, activez le repli manuellement dans `user.lua` :

```lua
-- Virtual machine compatibility
hl.env("LIBGL_ALWAYS_SOFTWARE", "1")
```
Après avoir créé ou modifié `user.lua`, déconnectez-vous de Hyprland et reconnectez-vous afin que la variable soit appliquée aux applications lancées par la session.

Vérifiez-le avec :

```sh
echo $LIBGL_ALWAYS_SOFTWARE
```
La sortie attendue est :

```text
1
```
> **Remarque :** `LIBGL_ALWAYS_SOFTWARE=1` oblige les applications Mesa/OpenGL à utiliser le rendu logiciel, généralement via `llvmpipe`. Cela peut réduire les performances graphiques et doit être utilisé comme solution de secours pour les machines virtuelles concernées.
