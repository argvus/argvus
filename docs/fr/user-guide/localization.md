---
title: Localisation
description: Utilisez ARGVUS en anglais ou en portugais.
---

`argvus-i18n` fournit les catalogues partagés, `argvus i18n validate` commande et le QML module `org.argvus.i18n`. ARGVUS bateaux 18 langues d'interface: arabe, bengali, allemand, anglais, espagnol, français, hindi, indonésien, italien, japonais, coréen, portugais brésilien, portugais européen, russe, turc, ourdou, vietnamien et chinois simplifié.

Native ARGVUS applications et composants shell consomment les catalogues partagés. Une chaîne traduite peut donc être utilisée par plus d'un dépôt. Arabe et Urdu utilisent la disposition de droite à gauche dans le support GTK, Quickshell et interfaces terminal.

Utiliser **Control Center → Préférences → Langue et région → Langue** pour sélectionner le ARGVUS langage d'interface. Ceci écrit `ARGVUS_LANGUAGE` à `$XDG_CONFIG_HOME/locale.conf` (normalement `~/.config/locale.conf`). Par exemple, `ARGVUS_LANGUAGE=pt-PT` sélectionne le portugais européen. Portugais brésilien (`pt-BR`) est une option séparée. Le catalogue de langues est installé avec ARGVUS, donc il ne dépend pas de `locale-gen`.

Le panneau de configuration redémarre automatiquement après les changements de langage d'interface. Commencez une nouvelle session pour l'appliquer aux composantes de session. Le paramètre distinct **Région locale** écrit `LANG` et contrôle les formats régionaux et les applications tierces. Il nécessite un système généré local; `/etc/locale.gen` pistes `locale-gen` avec l'autorisation de l'administrateur. La mise en page du clavier, la variante et la console sont des paramètres séparés.

## Lorsque les variables d'environnement remplacent le changement

ARGVUS lire `ARGVUS_LANGUAGE` de l'utilisateur `locale.conf`. Si elle est absente, ils résolvent `LANG` valeur d'un catalogue installé. Les variables locales héritées ne remplacent pas une variable explicite `ARGVUS_LANGUAGE` préférence. Les applications tierces continuent d'utiliser le système régional local.

Inspecter les valeurs efficaces avec:

```sh
systemctl --user show-environment | rg '^(LANG|LC_|ARGVUS)='
locale
printf '%s\n' "$LANG"
localectl status
```

Vérifiez. `~/.config/locale.conf` contient une `ARGVUS_LANGUAGE=<catalog>` valeur. Pour modifier le formatage régional ou le langage d'application de tiers, utilisez **Regional Locale** et sélectionnez un local généré. Le panneau de configuration est redémarré après chaque changement de préférence; se déconnecter et si d'autres composants de session affichent encore l'ancien langage d'interface. Voir [variables environnementales](/docs/reference/environment-variables/) pour les contrats techniques et la préséance.
