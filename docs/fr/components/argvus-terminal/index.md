---
title: Terminal
description: Ouvrez le terminal ARGVUS et les profils de l'application terminal.
---
Ouvrez le terminal configuré avec :

```sh
argvus --terminal
```
`argvus terminal` (un sous-commande de `argvus`) fournit l'intégration du terminal et la configuration de Kitty. `argvus-app-profiles` fournit des profils pour des applications terminales telles que Superfile et Yazi ; il ne fournit pas le répartiteur `argvus`.

`argvus-config` projette le thème dans `data/generated/terminal/`, et `argvus --yazi` et `argvus --spf` lisent leur configuration à partir des arbres `data/generated/yazi/` et `data/generated/superfile/` projetés plutôt qu'à partir de copies matérialisées dans votre répertoire personnel. Un remplacement complet natif dans le répertoire de configuration propre à l'application prévaut toujours, donc vous pouvez remplacer l'arbre projeté par votre propre configuration.

Les modifications de thème régénèrent les fichiers dérivés de Kitty sous `$XDG_CACHE_HOME/argvus/argvus-terminal`. Le générateur sérialise les mises à jour et remplace chaque fichier de manière atomique. Si le terminal ne s'ouvre toujours pas, inspectez le thème actif et régénérez le cache avec :

```sh
argvus config get /appearance/theme --raw
argvus terminal --apply
argvus terminal --print-config
```
Le terminal lit d'abord le thème géré et l'accent depuis `$XDG_CONFIG_HOME/argvus/config.json`. Les fichiers hérités `.active-theme` et `.accent-color` restent des solutions de repli pour les installations plus anciennes.

## Transparence et flou

Sous **Centre de contrôle → Apparence → Terminal**, configurez **Transparence** et **Flou** indépendamment de `0%` à `100%`. Les modifications sont appliquées avec le bouton **Appliquer** et régénèrent le profil Kitty sous `$XDG_CACHE_HOME/argvus/argvus-terminal` ; les fenêtres Kitty existantes reçoivent la configuration mise à jour via un rechargement.

La transparence `0%` garde l'arrière-plan opaque et `100%` le rend totalement transparent. Le flou utilise le rayon de flou d'arrière-plan supporté par Kitty et n'a un effet visible que lorsque la fenêtre est transparente.

## Notification des commandes de longue durée

Les commandes qui s'exécutent pendant 30 secondes ou plus envoient une notification lorsqu'elles se terminent, tant que la fenêtre du terminal n'est pas sélectionnée. La notification utilise `notify_on_cmd_finish` de Kitty avec l'intégration du shell déjà activée, donc `~/.zshrc` et `~/.bashrc` n'ont pas besoin d'être modifiés. Le seuil de 30 secondes est fixé dans la configuration de Kitty.

## Mode application TUI

ARGVUS Control Center et les popups TUI de la barre des tâches utilisent `argvus tui-terminal` avec un profil d'application Kitty dédié. Il hérite du thème et de la police ARGVUS actifs, attribue une classe de fenêtre stable et se ferme avec son enfant au lieu d'ouvrir un shell. Le Control Center masque les onglets et possède des valeurs indépendantes de transparence et de flou ; les popups de surveillance de la barre des tâches et de température utilisent une barre d'onglets inférieure pour leur titre `Press q to close` localisé. Le lanceur accepte `--title TITLE` pour définir un titre fixe de fenêtre et d'onglet Kitty. Le terminal normal conserve ses onglets et ses paramètres.

Lorsque LANG est absent de l'environnement du lanceur, `argvus terminal` fournit la langue personnelle de `$XDG_CONFIG_HOME/locale.conf` (anglais par défaut) avant que Kitty n'initialise la composition du clavier. Les remplacements existants de l'environnement de langue sont conservés. Redémarrez Kitty après la mise à jour : l'exportation de LANG dans un shell existant ne réinitialise pas la table de composition de Kitty.

Les valeurs vides de LC_ALL et LC_CTYPE héritées de l'initialisation de la session sont supprimées de l'environnement enfant de Kitty. Sinon, Kitty considère une langue vide comme explicite et ne peut pas charger sa table de composition, même avec un LANG valide. Les remplacements non vides sont conservés. Cela s'applique également aux nouveaux profils et nécessite de rouvrir Kitty après la mise à jour du lanceur.

La fenêtre dédiée du Centre de Contrôle (`argvus-control-center`) reçoit le remplacement Kitty par lancement `confirm_os_window_close=0`, de sorte que l'action de fermeture du compositeur ne demande pas de confirmation. Le remplacement n'est pas écrit dans le profil de terminal partagé ; les autres TUI et terminaux réguliers conservent leur politique existante.
