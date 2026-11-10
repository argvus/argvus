---
title: Configuration et état
description: État partagé, remplacements et fichiers d'exécution générés.
---
L'utilisateur normal root est `$XDG_CONFIG_HOME/argvus` (généralement `~/.config/argvus`). Les préférences logiques gérées sont stockées dans `config.json`. Les préférences de thème, d'accent, de fond d'écran, d'espacement et de bordure y sont canoniques ; les fichiers point restent des projections dérivées de compatibilité pour les consommateurs actuels de Lua et du shell Hyprland tandis que ces consommateurs migrent vers `data/`.

`argvus-config` est le seul écrivain en dessous de `generated/`. Chaque fichier consommateur appartenant à ARGVUS est une projection de `config.json`, y compris Hyprland, GTK, Qt6ct, Waybar, Quickshell, Dunst, Hyprlock, la barre des tâches Waybar et les profils de télémetrie des widgets, les arbres de configuration Yazi et Superfile, les arbres de profils du terminal et Qt, les blocs de polices gérés et la feuille de style des périphériques amovibles. Les composants résolvent d'abord les remplacements explicites de l'utilisateur, puis la sortie générée, puis les valeurs par défaut du package. Rofi n'est pas une projection : `argvus-launcher` possède ses fichiers, et l'utilisateur copie en direct sous `data/rofi/`. La projection normale thème/reload ne promeut jamais ni les fichiers générés ni les fichiers packagés en un remplacement utilisateur. La documentation et les outils devraient identifier la source logique au lieu de demander aux utilisateurs de modifier la sortie générée.

`argvus-appearance` possède donc l'orchestration des thèmes et des accents, tandis que `argvus-config` possède l'état canonique et la projection structurée. `argvus theme-switch` (la transaction `argvus config theme-apply`) et `argvus config accent` valident, verrouillent et confirment via `argvus-config` lorsqu'il est disponible, puis réconcilient les consommateurs de compatibilité qui ne peuvent pas lire `config.json` directement. Ces adaptateurs sont le fichier natif Qt6ct, GTK `settings.ini` et `gsettings` hints, les applications terminal et system-monitor, foot, le propre répertoire de configuration de superfile ainsi que le greeter de pré-authentification. Un assistant qui a besoin d'un nouvel adaptateur doit l'ajouter là, jamais à l'intérieur de `generated/`.

La propriété au moment de l'exécution est intentionnellement répartie par responsabilité : Rust possède le magasin canonique, la validation des schémas, les migrations, le modèle de chemin, les verrous, les écritures atomiques, les hachages de projection et les renderers structurés. Shell reste uniquement comme de petites entrées de compatibilité ou des adaptateurs pour les consommateurs externes tels que `hyprctl` et les scripts de thème possédés par des actifs. Python est réservé aux tests, audits et outils de développement ; il ne fait pas partie du runtime critique du bureau.

`argvus config apply` est la frontière de reconstruction (`project` reste l'orthographe de compatibilité). Il régénère les sorties manquantes même lorsque le document canonique n'est pas modifié, rend les projecteurs en staging, et échange atomiquement l'arbre `data/` seulement après que chaque projection réussisse. Le cycle de vie de la session reste la propriété de `argvus sessionctl`, qui utilise ce manifeste pour planifier des rechargements ciblés.

Lorsqu'un appelant a déjà généré une génération — `apply-theme`, un changement d'accent, un `set`/`patch` ou un changement de bloc de télémetrie de widget — `reload` **applique** cette génération au lieu de la projeter à nouveau. Il lit le plan à partir du manifeste, restaure les consommateurs ciblés et ne projette que lorsque le manifeste est absent ou n'a aucun travail en attente. La reprojection à ce moment reconstruirait un arbre identique et oublierait le plan que la mutation vient de produire.

Le manifeste enregistre également la génération opérationnelle, le hachage de la configuration effective, les domaines modifiés et la liste des fichiers générés. `data/internal/generation.json` est un marqueur de projection locale jetable et n'est jamais une préférence utilisateur. `argvus config status` compare ces enregistrements avec la configuration résolue et signale les fichiers générés manquants ou supplémentaires.

Les mutations suivent la même limite de moteur : `UserConfig` est cloné dans un `NextUserConfig`, validé, résolu dans `EffectiveConfig`, projeté, et ce n'est qu'ensuite qu'il est persistant. `.config-transaction.json` est l'état de récupération opérationnelle ; `argvus config recover` reprojette de manière déterministe le document canonique précédent après un crash entre la publication de `data/` et `config.json`. Le rechargement n'est demandé qu'après l'engagement, et un échec de rechargement ne revient pas sur une configuration persistante cohérente.

## Modèle de résolution

```text
config.json logical model
    ↓
ARGVUS native compatibility state and per-app override
    ↓
ARGVUS generated runtime file
    ↓
/usr/share/argvus packaged default
    ↓
upstream default
```
Les candidats exacts varient selon le composant, mais le principe reste le même : une substitution par l’utilisateur prévaut, la sortie générée est un état dérivé, et les fichiers empaquetés restent des entrées en lecture seule. La session exporte `ARGVUS_CONFIG_HOME` et `ARGVUS_SYSTEM_CONFIG` afin que les assistants et les services utilisateurs résolvent les mêmes racines.

La récupération au rechargement suit `recover → ensure → migrate → apply → runtime reload`, et l’étape de projection est ignorée lorsque le manifeste entrant décrit déjà une génération validée. `ensure` matérialise `config.json` à partir des valeurs par défaut du schéma uniquement lorsque le profil est absent, et remplit les champs manquants sans remplacer les `false`, `0` ou valeurs personnalisées explicites. Un document existant malformé reste une erreur plutôt que d’être remplacé silencieusement.

Les mutations persistantes comparent les anciennes et nouvelles configurations résolues via `ChangedDomains`. Après validation, une demande de rechargement est envoyée via `argvus sessionctl` ; l'échec du rechargement est signalé et enregistré sans revenir à la configuration validée. Le démarrage de la session effectue une récupération avant la matérialisation et la projection du bootstrap, de sorte que les consommateurs ne démarrent pas sur une transaction abandonnée.

| État | Objet |
| --- | --- |
| `config.json` | Préférences logiques gérées canoniques ; la source unique pour tout ce qui est sous `generated/`. |
| `.active-theme`, `.accent-color`, `.accent-custom` | Entrées de migration héritées ; pas des sources d'exécution et non régénérées. |
| `.wallpaper-custom` | Entrée de migration héritée pour `appearance.wallpaper` ; pas une source d'exécution. |
| `.spaces`, `.borders` | Entrées de migration de mise en page héritée ; les valeurs canoniques sont stockées sous `layout` dans `config.json`. |
| `fonts.conf` | Projection de compatibilité des sélections de polices canoniques. |
| `generated/effects` | Projection de compatibilité des effets canoniques ; jetable. |
| `$XDG_STATE_HOME/argvus/config-projection.json` | Manifeste de projection : sections modifiées, consommateurs affectés et nécessité éventuelle d’un rechargement. Lu par `reload` ; jamais une source de vérité. |
| `$XDG_STATE_HOME/argvus/widget-telemetry-blocks` | Préférences du bloc de télémétrie héritées. Lu uniquement comme solution de repli pour la migration ; les valeurs canoniques se trouvent à `/control_panel/widget_telemetry_blocks` et le fichier n’est plus écrit. |

Une propriété est canonique lorsqu’elle est lue depuis `config.json` et projetée à partir de là. Une propriété est dérivée lorsqu’elle est projetée mais aussi consommable comme entrée de migration. Tout ce qu’un composant continue d’écrire à l’exécution — un cache, un fichier de configuration natif sous `$XDG_CONFIG_HOME/<app>`, un fichier d’état du greeter — est une cible d’adaptateur et ne doit pas être relu en tant que configuration.

La projection couvre les consommateurs pour Hyprland, GTK/Qt, Waybar, Quickshell, les terminaux, les notifications, l'écran de verrouillage et les applications natives. Hyprland lit les couleurs des bordures actives et inactives à partir du thème sélectionné ; un `appearance.accent_custom=true` explicite est le seul cas qui remplace la bordure active/de groupe avec l'accent de l'utilisateur. Le changement de thème réapplique ces valeurs via le rechargement du compositeur, de sorte qu'il ne peut pas conserver les couleurs du thème précédent. Si la configuration utilisateur est recréée, les effets sont par défaut activés avec les animations. Supprimer `generated/` et exécuter `argvus config apply` le recrée octet par octet à partir de `config.json` ; modifier directement les fichiers générés n'est pas durable. Le diagnostic à l'exécution doit identifier le tuple complet d'exécution : chemin binaire, version/révision, racine des actifs, racine de configuration, racine des données, racine générée, racine interne et manifeste de projection. `status --verbose` expose ce tuple et
compare la génération/hash publiée avec la configuration résolue actuelle.  
Les tests de développement et de paquet installé sont des modes séparés ; aucun ne peut revenir silencieusement à un exécutable non associé ou au répertoire personnel du développeur. `apply` est également la limite de réconciliation pour les identifiants de thème combinés hérités : une valeur telle que `argvus-dark-spaced` devient `theme=argvus-dark` et `variant=spaced` avant résolution. L'identité des ressources empaquetées n'est pas enregistrée dans un manifeste dans cette version, donc aucun fichier de diagnostic ne la décrit.
