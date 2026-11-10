---
title: Internationalisation ARGVUS
description: Catalogues de langues, sélection et prise en charge de l'Unicode.
---

# Internationalisation ARGVUS

ARGVUS propose l'anglais (`en-US`), le portugais brésilien (`pt-BR`), l'espagnol (`es-ES`),
le français (`fr-FR`), l'italien (`it-IT`), l'allemand (`de-DE`), le russe (`ru-RU`), le chinois simplifié (`zh-CN`), le japonais (`ja-JP`) et le coréen (`ko-KR`). L'anglais est le catalogue canonique et de secours. Les huit traductions supplémentaires ont été produites avec l'aide de la traduction automatique et des corrections techniques ; une révision indépendante par un locuteur natif n'a pas été certifiée.

## Sélectionner une langue

Ouvrez le Centre de Contrôle > Préférences > Langue et Région > Langue, ou Paramètres > Langue. La page découvre les catalogues installés et affiche leurs noms natifs. Elle sauvegarde `LANG` dans `$XDG_CONFIG_HOME/locale.conf` (par défaut `~/.config/locale.conf`). Activez d'abord la locale système UTF-8 correspondante dans les Locales Système. Des exemples incluent `es_ES.UTF-8`, `zh_CN.UTF-8`, `ja_JP.UTF-8` et `ko_KR.UTF-8`.

La préférence ne modifie pas `/etc/locale.conf`. Les commentaires et les paramètres de catégorie régionale sont conservés. Le Panneau de configuration est redémarré après l'enregistrement ; déconnectez-vous puis reconnectez-vous pour appliquer la langue dans toute la session. Les utilisateurs non configurés utilisent l'anglais. Les variables d'environnement héritées et les fichiers de langue ARGVUS hérités ne remplacent pas cette préférence. Le greet de connexion s'exécute avant que la préférence de l'utilisateur ne soit disponible et conserve sa propre langue de compte de service, l'anglais par défaut.

Les variantes régionales utilisent le catalogue installé pour leur langue lorsqu'un catalogue exact est absent ; par exemple `es_MX.UTF-8` utilise `es-ES`. `zh-Hans` sélectionne le chinois simplifié. Le chinois traditionnel (`zh-Hant`, `zh-TW`, `zh-HK`, `zh-MO`) n'est pas mappé silencieusement au chinois simplifié et revient à l'anglais.

## Catalogues et contributions

La source est `resources/locales/<locale>/<domain>.json` dans le dépôt `argvus`. Les catalogues installés se trouvent dans `/usr/share/argvus/i18n/`. Ils sont fournis dans le paquet principal `argvus`. `locale.json` fournit l'identifiant ainsi que les noms natifs/affichés. Il n'y a pas de catalogue PO séparé ni de plateforme de traduction en ligne requise.

Utilisez des clés sémantiques et préservez les noms des espaces réservés, les commandes, options, chemins et raccourcis clavier. Traduisez les messages complets au lieu d'assembler des phrases à partir de fragments traduits. Les nouvelles clés canoniques nécessitent des traductions dans les dix langues. Maintenez la cohérence de la terminologie dans chaque domaine et révisez les traductions dans leur contexte, en particulier pour l'authentification, les autorisations et les actions destructrices.

```sh
argvus i18n list
argvus i18n locale
argvus i18n get common welcome name=Alice
argvus i18n validate
```
`ARGVUS_I18N_DIR` sélectionne une racine de catalogue alternative pour le développement et les vérifications des packages en cours de préparation. La validation vérifie les domaines, les clés, les valeurs de chaîne, les espaces réservés, les traductions vides et les formes plurielles entières spécifiques à la langue. La validation prouve l’intégrité structurelle, non la précision linguistique.

## Unicode et polices

`argvus-fonts` nécessite les polices Noto CJK en plus des familles de polices latines, de symboles et d’émojis. Fontconfig fournit une solution de repli pour le chinois, le japonais et le hangul sans remplacer la famille principale choisie par l’utilisateur. Les applications et terminaux ont toujours besoin d’avoir le repli de polices activé.

Les helpers de texte partagés de Ratatui mesurent les cellules du terminal, préservent les clusters de graphèmes étendus lors de la troncature et gèrent l'habillage du texte large. Les éditeurs de texte déplacent et suppriment des graphèmes entiers. Les zones extrêmement étroites ne peuvent pas afficher un glyphe de deux cellules dans une seule cellule ; dans ce cas, le helper d'habillage dur utilise un caractère de remplacement. Les émojis et les caractères à largeur ambiguë peuvent encore différer selon les implémentations du terminal ou de la police. La politique de largeur du terminal devrait être compatible avec la largeur Unicode normale de Ratatui.

La sélection de la langue n'installe pas de méthode de saisie ni ne change la disposition du clavier. Utilisez une méthode de saisie compatible Wayland pour entrer du chinois, du japonais ou du coréen.
