---
title: Dispositifs amovibles
description: Monter et gérer le stockage amovible depuis ARGVUS.
---
`argvus-removable-devices` surveille UDisks2 et fournit le module de stockage ARGVUS Taskbar et son menu de périphérique. Le menu est également disponible sous forme d'interface séparée de style Rofi, de sorte que les mêmes actions sur les périphériques peuvent être utilisées depuis la barre des tâches ou depuis un flux de travail de lanceur.

Les actions disponibles sont `open`, `mount`, `unmount`, `eject`, `poweroff`, `lock`, `unlock` et `copy`. Les commandes `list`, `watch` et `menu` exposent les interfaces correspondantes de statut, de flux de la barre des tâches et de menu pour les intégrations.

## Menu de la barre des tâches

Avec `"mode": "gui"` (le réglage par défaut du système), un clic sur l'icône de stockage de la barre des tâches ouvre un menu GTK attaché à l'ARGVUS Taskbar, comme le popup du calendrier : en dessous de la barre lorsque la barre des tâches est en haut de l'écran, et au-dessus lorsqu'elle est en bas. Chaque ligne de périphérique possède un sous-menu d'action avec uniquement les actions permises par son état.

- **Pointeur :** passer le curseur ou cliquer sur une ligne de périphérique ouvre son sous-menu. Le sous-menu reste ouvert tant que le pointeur traverse les bords du menu ou quitte les menus, et il suit une autre ligne seulement après que le pointeur s'y soit arrêté brièvement, donc un chemin diagonal vers le sous-menu ne le ferme pas. Un clic en dehors des deux menus ferme la fenêtre contextuelle.
- **Clavier :** `Up`/`Down` sélectionne une ligne, `Right`, `Enter` ou `Space` ouvre le sous-menu (ou exécute **Rafraîchir**/**Quitter**), `Enter` ou `Space` exécute l'action sélectionnée, `Left` ou `Escape` revient à la liste des périphériques, et `Escape` ferme la fenêtre contextuelle.

**Copier le chemin** met le point de montage dans le presse-papiers via `copy_command` (par défaut `wl-copy`, depuis `wl-clipboard`), de sorte que le chemin soit toujours disponible après la fermeture du menu ; une notification le confirme lorsque `show_notification` est activé. **Verrouiller** est proposé pour chaque volume chiffré déverrouillé et le démonte d'abord si nécessaire.

L'action par défaut du gestionnaire de fichiers suit le gestionnaire de fichiers sélectionné dans le Centre de contrôle ARGVUS. Cela permet de maintenir la cohérence des actions sur les périphériques amovibles avec les paramètres par défaut de l'application de l'utilisateur.

La configuration système est `/etc/argvus/removable-devices/config.json`. La projection par utilisateur gérée par ARGVUS est `~/.config/argvus/data/removable-devices/config.json`, avec un `theme.css` correspondant à côté. Cette feuille de style est également projetée vers `~/.config/argvus/data/generated/removable-devices/theme.css` à partir du thème canonique, des accents et du mode, avec une alternative famille/variante pour les thèmes qui ne fournissent aucun élément dédié aux périphériques amovibles, de sorte qu'un changement de thème maintient la cohérence du module sans aucune étape manuelle. Les chemins des composants hérités sont des entrées de migration ; la configuration ultérieure de l'utilisateur remplace les paramètres par défaut du système.

Les paramètres utiles incluent si le module est caché lorsqu'aucun appareil n'est présent (`hide_when_empty`), si les noms et les capacités sont affichés, l'ordre de tri des appareils, le comportement des notifications, le backend du menu et les icônes de statut. Sa feuille de style est rendue à partir du thème actif ARGVUS (`removable-devices/theme.css` de la construction du thème) et copiée dans le `theme.css` du panneau à chaque changement de thème ou d'accent.

L'interface utilise la langue de l'utilisateur à partir de `LANG` dans `$XDG_CONFIG_HOME/locale.conf`, l'anglais étant la valeur par défaut pour un utilisateur non configuré. Les variables locales héritées ne sélectionnent pas la langue de l'interface.

## Catalogues de langue partagés

L'interface utilise les catalogues partagés `argvus-i18n` pour les dix langues. Les textes sont stockés dans `resources/locales/<locale>/removable-devices.json` dans l'espace de travail principal et installés par `argvus`. L'anglais est la langue de secours ; les nouvelles langues ne nécessitent plus de dictionnaire bilingue local.
