---
title: Variables d'environnement
description: Contrats d'environnement utilisés par les composants de session ARGVUS.
---
La session établit l'intégration des environnements XDG et Wayland et peut exporter des variables de compatibilité de virtualisation lorsqu'elles sont détectées. `ARGVUS_CONFIG_HOME`, `ARGVUS_SYSTEM_CONFIG` et `ARGVUS_BOOTSTRAP` sont utilisés par les assistants de session et de dispatcher pour résoudre les racines de configuration.

Considérez les variables d'environnement comme des contrats d'exécution : vérifiez le `--help` de l'assistant installé ou sa source avant d'ajouter une substitution persistante.

L'état dérivé se trouve sous la racine d'état plutôt que sous la racine de configuration. `XDG_STATE_HOME` (normalement `~/.local/state`) reçoit `argvus/config-projection.json`, le manifeste de projection `argvus-config` se reconstruit et `argvus sessionctl reload` s'applique, plus `argvus/session.log`. Aucun d'entre eux n'est une source de vérité.

## Environnement linguistique

La seule préférence linguistique générale est `LANG` dans `$XDG_CONFIG_HOME/locale.conf` (par défaut `~/.config/locale.conf`). Sans valeur valide, ARGVUS sélectionne l’anglais (`en-US`), même lorsque `/etc/locale.conf`, PAM ou le gestionnaire de connexion fournit le portugais. Des valeurs telles que `pt_BR.UTF-8` se normalisent en `pt-BR` pour les catalogues.

La session applique cette préférence avant de démarrer les composants du bureau et d’importer les environnements d’activation systemd utilisateur et D-Bus. Elle remplace `LANG`, `LC_MESSAGES`, `LANGUAGE` hérités et efface `LC_ALL` ; les paramètres régionaux `LC_*` dans le même fichier sont conservés. L’anglais utilise `en_US.UTF-8` lorsqu’il est généré, sinon `C.UTF-8`. Aucun fichier de préférence n’est créé automatiquement. Déconnectez-vous et reconnectez-vous après avoir changé la langue.

`session.language`, `argvus/data/language` et `argvus/data/internal/language` sont des données héritées ignorées. Elles ne sont ni importées ni régénérées ; les fichiers existants sont conservés. `argvus config set` et `patch` rejettent les nouvelles écritures vers `session.language` et dirigent les appelants vers `locale.conf`.

Pour le diagnostic, comparez `argvus i18n locale`, `cat "${XDG_CONFIG_HOME:-$HOME/.config}/locale.conf"` et `systemctl --user show-environment`. `localectl status` décrit le système, pas le langage ARGVUS de l'utilisateur. `ARGVUS_I18N_DIR` sélectionne un répertoire de catalogue pour le développement, pas le langage.
