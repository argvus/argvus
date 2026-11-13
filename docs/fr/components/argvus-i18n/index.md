---
title: Internationalisation ARGVUS
description: Catalogues de langues, sélection et prise en charge de l'Unicode.
---

# Internationalisation ARGVUS

ARGVUS navires 18 langues d'interface: arabe (`ar`), Bengali (`bn-BD`), allemand (`de-DE`),
Français (`en-US`), espagnol (`es-ES`), français (`fr-FR`), Hindi (`hi-IN`), Indonésie
(`id-ID`), italien (`it-IT`), japonais (`ja-JP`), coréen (`ko-KR`), portugais brésilien
(`pt-BR`), portugais européen (`pt-PT`), russe (`ru-RU`), turc (`tr-TR`), Urdu
(`ur-PK`), vietnamien (`vi-VN`) et chinois simplifié (`zh-CN`). L'anglais est le
catalogue canonique et repli. Les traductions ont été produites avec AI l ' assistance et
les corrections techniques; l'examen indépendant par un orateur natif n'a pas été certifié.

## Sélectionner une langue

Open Control Center > Préférences > Langue et région > Langue. La page découvre
les catalogues installés et montre leurs noms natifs. Il sauve `ARGVUS_LANGUAGE` dans
`$XDG_CONFIG_HOME/locale.conf` (par défaut) `~/.config/locale.conf`), indépendamment de
`LANG`; il n'est pas nécessaire de générer une localisation du système. Portugais brésilien (`pt-BR`) et
Portugais européen (`pt-PT`) sont des choix séparés.
La préférence ne modifie pas `/etc/locale.conf` ou les régions `LANG` mise en place.
Les commentaires et les paramètres de catégorie régionale sont conservés. Le panneau de configuration est redémarré
après l'enregistrement; commencer une nouvelle session pour appliquer la langue à tous les composants de la session.
Les utilisateurs non configurés utilisent l'anglais. Contrôles distincts des préférences régionales locales
`LANG`, le formatage régional et les applications tierces. L'accueil de connexion fonctionne
avant que la préférence de l'utilisateur soit disponible et conserve son propre compte de service
langue, anglais par défaut.
Les variantes régionales utilisent le catalogue installé pour leur langue quand une
catalogue est absent; par exemple `es_MX.UTF-8` Utilisations `es-ES`. `zh-Hans` sélectionne
Chinois simplifié. Chinois traditionnel (`zh-Hant`, `zh-TW`, `zh-HK`, `zh-MO`) est
pas tapé silencieusement sur le chinois simplifié et tombe à l'anglais.

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
