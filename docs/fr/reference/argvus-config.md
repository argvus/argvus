---
title: Configuration ARGVUS
description: Le contrat de configuration JSON logique utilisé par ARGVUS.
---
ARGVUS conserve les préférences utilisateur gérées dans des fichiers JSON modulaires sous `$XDG_CONFIG_HOME/argvus/config/` (normalement `~/.config/argvus/config/`). Ils forment le modèle logique canonique pour les préférences gérées par ARGVUS, et `argvus-config` est le seul composant qui écrit des fichiers en dessous de `data/generated/`. Les fichiers natifs et les fichiers générés restent des projections de compatibilité ou des substituts spécifiques à l'application.

## Guide de l'utilisateur : Démarrage rapide

Pour la plupart des utilisateurs, la configuration se fait via le Centre de contrôle ARGVUS ou cet outil en ligne de commande :

```sh
# View your configuration location and validate it
argvus config path
argvus config validate

# Get a specific value
argvus config get /appearance/theme
argvus config get /appearance/theme --raw    # Without JSON formatting

# Set a value (must be valid JSON)
argvus config set /appearance/theme '"gruvbox-dark"'

# Apply changes to reload the session
argvus config apply
```
Pour les administrateurs et la configuration avancée, continuez à lire ci-dessous.

L'utilisateur root contient uniquement le document canonique et les données gérées :

```text
~/.config/argvus/
├── config.json
└── data/
```
Le fichier est écrit de manière atomique, reste privé pour l'utilisateur et est protégé par `.config.lock`, en dehors de l'arborescence `data/` qui peut être échangée lors de la projection. Une version précédente est stockée à `data/backups/config.json.bak`.

## Ligne de commande

La sous-commande `argvus config` est fournie en processus par le package `argvus` :

```sh
argvus config path
argvus config validate
argvus config ensure
argvus config migrate
argvus config status
argvus config explain /layout/gaps_out
argvus config get /appearance/theme --effective --raw
argvus config set /appearance/accent '"#61AFEF"'
argvus config patch appearance-patch.json
argvus config audio volume +5
argvus config audio mute toggle
argvus config apply-theme universe --variant sticky --accent '#EEEEEE' --gtk-mode dark
argvus config apply-theme universe-spaced --variant spaced --accent '#EEEEEE' --gtk-mode dark
argvus config theme-manifest universe-spaced
argvus config unset /appearance/accent
argvus config apply
argvus config rebuild
argvus config project
argvus config project --force
```
`ensure` matérialise un nouveau `config.json` avec les valeurs par défaut du schéma, ou remplit les champs manquants sans remplacer les valeurs explicites. Il échoue avec un JSON mal formé.
`migrate` importe l'état d'apparence hérité existant sans remplacer les valeurs déjà présentes dans `config.json`. `argvus sessionctl` exécute `recover`, `ensure`, `migrate` et `project` avant d'appliquer un rechargement après les modifications de configuration :

```sh
argvus sessionctl apply-config
```
`apply` et `rebuild` sont des alias pour reconstruire toutes les projections à partir de `config.json` ; `project` est l'orthographe historique et accepte `--force`. `apply` délègue également le rechargement d'exécution post-commit à `argvus sessionctl`. Configurez `ARGVUS_NO_RUNTIME=1` pour la préparation ou les tests isolés. La génération se fait en staging et l'arbre `data/` n'est échangé qu'après la réussite de tous les projecteurs. Par conséquent, supprimer `data/generated/` et exécuter `argvus config apply` constitue une reconstruction reproductible sans publier une séquence partielle des sorties Hyprland ou Taskbar. La commande enregistre les hachages de section dérivés et la décision à l'exécution dans `$XDG_STATE_HOME/argvus/config-projection.json` (normalement `~/.local/state/argvus/config-projection.json`). Ni le manifeste ni les fichiers générés ne sont sources de vérité. `apply` effectue le rechargement post-commit via `argvus sessionctl` ; `project` reste uniquement pour les projections pour des raisons de compatibilité.

Chaque mutation persistante utilise la même règle : si le document canonique a changé, le CLI invoque un `argvus sessionctl reload` centralisé après le commit. Le résultat du rechargement est enregistré dans `data/internal/last-reload.json`.

`argvus sessionctl reload` est donc une étape *d'application*, et non une étape de projection. Lorsque le manifeste entrant signale un travail en attente, il restaure les consommateurs ciblés décrits par le plan et n'appelle pas `argvus config project` à nouveau ; une reprojection reconstruirait un arbre identique et jetterait le plan que la mutation vient de produire. Il revient à la migration et à la projection uniquement lorsqu'aucun plan commis n'existe, par exemple lors du démarrage de session.

### Cibles de rechargement

`argvus config session-plan` répond au rechargement avec les cibles des sections qui ont changé, et `argvus sessionctl` exécute uniquement celles-ci. L'écran de démarrage couvre un rechargement uniquement lorsqu'une cible recrée une surface : une barre Waybar ou le panneau de contrôle revenant à l'écran déplace toutes les fenêtres. Tout le reste se recharge sur place.

| Changement | Cibles | Splash |
| --- | --- | --- |
| Thème, couleur de surbrillance ou mode GTK | `hyprland`, `consumers`, `wallpaper`, `polkit` | non |
| Fond d'écran uniquement | `wallpaper` (IPC hyprpaper ; le service redémarre uniquement pour swaybg) | non |
| `effects` (flou, animations, transparence, ombre) | `hyprland`, `consumers` | non |
| `layout` des fenêtres (espacements, bordures) | `hyprland` | non |
| `layout.taskbar` (position, marges) | `hyprland`, `taskbar`, `widget_telemetry` | oui |
| `fonts` | `consumers` | non |
| `control_panel` (interrupteur, cartes) | `control_panel` | oui |
| `default_apps`, `keyboard_shortcuts`, `hyprland`, `displays` | `hyprland` | non |
| `power` | `hypridle` | non |

`consumers` exécute `argvus config theme-runtime consumers` (les feuilles de style de la barre, le Panneau de Contrôle via IPC, les couleurs de Hyprland, le rechargement du thème de Dunst, btop et kitty). Dunst différé le rechargement de son thème lorsque le mode Ne Pas Déranger est actif. `wallpaper` exécute `argvus config theme-runtime wallpaper`, qui affiche le `/appearance/wallpaper` effectif. Le hachage `layout.taskbar` est conservé séparément de la disposition des fenêtres dans le manifeste de projection, donc un changement d'écart ne redémarre pas les barres. Le plan de secours `full`, utilisé lorsqu'aucun plan engagé n'existe, redémarre chaque surface sous l'écran de démarrage.

Chaque configuration effective possède une génération opérationnelle. Elle est stockée dans `data/internal/generation.json` et dans le manifeste de projection sous `$XDG_STATE_HOME/argvus/config-projection.json` ; elle ne fait pas partie de la configuration des fichiers pointés (dotfiles). La réapplication d'une configuration effective inchangée préserve la génération. `status` indique si le hachage du manifeste, les métadonnées de génération et tous les fichiers générés sont cohérents ; `apply` répare une projection obsolète ou incomplète.

`explain /path` lit directement le résolveur et affiche la valeur effective, la source, le thème et la variante. Il ne parse pas les fichiers générés ni ne crée une seconde règle de priorité.

La projection des thèmes et des effets est mise en œuvre dans `argvus-config` lui-même.
La transaction de thème (`argvus theme-switch`, qui exécute `theme-apply`), `argvus config accent` et `argvus effects-toggle` ne projettent pas :
ils valident, verrouillent, confirment via ce CLI puis
réconcilient les consommateurs externes qui ne peuvent pas lire `config.json`. `argvus effects-toggle`
est un délégué léger sans sortie propre. Les assistants d'apparence ne sont jamais
appelés par le projecteur, et ils ne doivent jamais écrire en dessous de `data/generated/`.

### Surfaces de projection

`project_effective` rend chaque consommateur appartenant à ARGVUS dans un arbre de préparation
avant publication :

| Surface | Sortie |
| --- | --- |
| Hyprland | `generated/hypr/`, y compris les profils de thème, `spaces-effective.conf`, `borders-effective.conf`, `hypridle.conf` et `monitors.lua` |
| Layout/effets | `generated/effects/`, `generated/spaces.conf`, `generated/borders.conf` |
| Waybar | `generated/waybar/`, y compris `argvus-taskbar.css` et `argvus-widget-telemetry.{jsonc,css}` |
| Thèmes de télémétrie des widgets | `generated/widget-telemetry/` et `generated/waybar/widget-telemetry-themes/` |
| QuickShell | `generated/quickshell/` |
| Notifications | `generated/notifications/` et `dunstrc` |
| Écran de verrouillage | `generated/hypr/hyprlock.conf` et les profils de thèmes Hyprtoolkit |
| GTK | `generated/gtk/` |
| Yazi | `generated/yazi/` |
| Superfichier | `generated/superfile/` |
| Terminal et profils Qt | `generated/terminal/`, `generated/qt6ct/` |
| Polices | `generated/fonts.conf` ainsi que les blocs de polices gérés dans les feuilles de style Waybar |
| Périphériques amovibles | `generated/removable-devices/theme.css` |
| Fond d'écran | `generated/hypr/hyprpaper.conf` et `data/hypr/hyprpaper.conf` |
| Panneau de contrôle et valeurs par défaut | `data/control-panel/`, `data/control-center/defaults.json` |
| Calendrier, saisie, clavier, alimentation | calendrier TOML, fragment de saisie, état du clavier, marqueur de veille active |

La couche de thème est également dupliquée en dehors de la racine générée :
`data/waybar/argvus-taskbar.{jsonc,css}`,
`data/waybar/argvus-widget-telemetry.{jsonc,css}`, `data/foot/`,
`data/qt6ct/`, `data/hypr/hyprtoolkit.conf` et
`data/hypr/application-style.conf`. Ces copies sont remplacées par les valeurs par défaut emballées lorsque la section apparence change, et le projecteur réécrit ensuite uniquement ses blocs gérés délimités — le bloc de police dans les feuilles de style Waybar et les blocs `ARGVUS_TELEMETRY_*` dans le profil de télémétrie — donc le texte en dehors des marqueurs ne survit pas à un changement de thème. `fonts.targets` et
`control_panel.widget_telemetry_blocks` restent canoniques : le Centre de Contrôle
les conserve à travers `argvus-config` et ne réécrit jamais ces fichiers lui-même.

La sélection des blocs de télémétrie des widgets est canonique à `/control_panel/widget_telemetry_blocks`, un tableau des sept noms de blocs (`system`, `cpu_gpu`, `memory`, `storage`, `processes`, `network`, `keys`). Le fichier héritage `$XDG_STATE_HOME/argvus/widget-telemetry-blocks` est en lecture seule uniquement comme solution de secours de migration et n'est jamais écrit. ` and ` and blocks all modifie le tableau canonique en une seule écriture atomique, et `apply-state` suppose que la sélection a déjà été persistée par son appelant.

Les préférences de spacing et de bordure de Hyprland suivent la même limite. Les commandes argvus config spaces et argvus config borders (et leurs wrappers de compatibilité historique) mettent à jour /layout ; Rust génère data/generated/spaces-effective.conf et data/generated/borders-effective.conf et est responsable de la validation et de l'application en temps réel via hyprctl. .spaces et .borders sont des états uniquement pour la migration et ne sont jamais la source de vérité.

L'accent et l'état du mode GTK utilisent la même limite Rust. Utilisez ` or ` or argvus config accent --theme-default pour la propriété de l'accent, et `argvus config mode toggle|set dark|light` pour le mode GTK. Les scripts historiques d'accent et de mode sont des wrappers de compatibilité ; `.accent-color`, `.accent-custom` et `.gtk-mode` sont uniquement des fichiers de migration/projection.

Les chemins logiques ARGVUS sont également résolus par `argvus config paths`. Le fichier héritage `paths.sh` n'expose que des fonctions de compatibilité ; les remplacements natifs, les données utilisateur, les données générées et les secours du système sont résolus par le modèle de chemin Rust.

`audio` change la sortie par défaut et enregistre le résultat dans `/audio` en une seule étape : `argvus config audio volume <N|+N|-N> [--limit PERCENT]` et `argvus config audio mute <on|off|toggle>`. Il écrit le magasin sans recharger la configuration, car PipeWire possède déjà la valeur. Le module de volume Waybar, les touches média Hyprland et le panneau de contrôle l'utilisent, donc `argvus config project`, qui projette `/audio` sur la sortie, ne restaure jamais un volume plus ancien. Un volume supérieur à 100 % (les touches Hyprland permettent jusqu'à 150 %) est stocké comme 100.

`patch` applique un objet JSON dont les clés sont des pointeurs JSON dans une seule opération verrouillée de lecture-modification-écriture. `apply-theme` effectue l'équivalent d'une transaction atomique de thème et remplace les sections d'apparence gérées du profil d'apparence optionnel du thème ou des valeurs par défaut emballées. Les commandes de mutation construisent un `NextUserConfig`, résolvent un `EffectiveConfig`, préparent tous les projecteurs, et seulement ensuite publient le JSON canonique. Un marqueur opérationnel `.config-transaction.json` permet à `argvus config recover` de reprojeter la configuration précédente après un crash entre les données et la publication JSON. Les valeurs par défaut du thème sont déclarées par le manifeste de chaque thème (`themes/<theme>/theme.toml`) : l'accent par défaut (`palette.accent`), le schéma de couleurs GTK et le fond d'écran. Les fichiers CSS, Lua, Rasi et Qt sont rendus à partir de celui-ci dans la construction du thème.

Changer le thème, y compris Clair/Sombre, réinitialise la couleur de surlignage à la valeur par défaut du thème, même lorsque l'utilisateur a choisi une couleur personnalisée : la couleur personnalisée est conservée uniquement jusqu'au prochain changement de thème. Utilisez `argvus config accent` après le changement de thème pour la définir à nouveau. Appliquer un thème avec `argvus config theme-apply` écrit `/appearance/accent` et `/appearance/gtk_mode` à partir du manifeste, et `argvus config accent` écrit la couleur choisie avec `accent_custom` défini, dans une seule commande.

`apply-theme` prend la variante explicitement comme `--variant sticky|spaced` et résout le nom demandé via le manifeste, donc une demande ne peut jamais se résoudre aux actifs de l'autre variante. `theme-manifest THEME` est la même résolution exposée en lecture seule : elle imprime l'entrée du manifeste ainsi que les `asset_id` et `layout_variant` auxquels elle s'est résolue, ce dont un appelant a besoin pour accéder aux actifs empaquetés. Sticky est l'identifiant sans suffixe et seul Spaced porte un suffixe `-spaced`, donc `dracula-sticky` et `argvus-dark-dracula-sticky` sont acceptés et normalisés en `dracula`.

Avec `--raw`, les valeurs de chaîne sont imprimées sans guillemets JSON, tandis que les objets et les tableaux restent des valeurs JSON. Les fichiers dot de thème et de mise en page ne sont plus écrits par l'assistant de projection ; `config.json` reste la seule source persistante pour ces préférences. En cas d'échec de la projection, le rechargement de la session s'arrête avant de redémarrer les surfaces du bureau, de sorte qu'un ancien thème partiel n'est pas présenté comme actuel.

## Exportation et importation

Le même modèle logique est utilisé pour l'échange de profils :

```sh
argvus config export --scope appearance --output appearance.json
argvus config import --scope appearance appearance.json
```
Utilisez `--scope desktop` pour le modèle complet de bureau géré. Les champs manquants utilisent les valeurs par défaut ARGVUS ; la sortie générée ne doit pas être modifiée directement.

L'exportation et l'importation de thème du Centre de contrôle échangent un thème et son profil visuel : l'archive contient un répertoire de thème de schéma 2 (`<id>/theme.toml`, optionnel `appearance.json` et `effects.conf`, `overrides/`, le fond d'écran) qui s'installe sous `$XDG_DATA_HOME/argvus/themes/`. Le profil contient l'apparence, la géométrie de la disposition, les effets visuels, les polices et la présentation des panneaux. Les préférences audio, clavier, réseau et autres préférences non visuelles restent hors des thèmes. Utilisez le scope `appearance` ci-dessus pour déplacer la configuration d'apparence complète, et le scope `desktop` pour une sauvegarde complète de configuration.

Le schéma actuel regroupe les préférences sous `appearance`, `layout`, `effects`, `fonts`, `control_panel`, `default_apps`, `keyboard_shortcuts`, `hyprland`, `displays`, `power`, `session`, `calendar` et `removable_devices`. `default_apps` est la seule source persistante pour les applications préférées, et `keyboard_shortcuts` stocke des clés stables en anglais, indépendantes de la locale, avec des valeurs de chaîne ou `null` pour les associations désactivées.

Le Centre de contrôle lit les valeurs canoniques d'apparence lorsqu'il ouvre une page d'apparence et écrit les modifications via `argvus-config`. Les fichiers d'état natifs existants restent des projections pour les scripts d'apparence et les installations plus anciennes.

Le Panneau de Contrôle utilise `control_panel.cards` pour l'ordre et la visibilité des cartes, et `control_panel.enabled` pour l'interrupteur principal du panneau. Ses fichiers `control-panel/cards.json` et `state/control-panel` sont conservés comme projections de compatibilité. Les modifications apportées dans le Panneau de Contrôle font donc partie du périmètre d'exportation `desktop`.

Les préférences de police utilisent `fonts.targets` pour la famille, le style et la taille de la barre des tâches, de la télémétrie, du Panneau de Contrôle, du système, des applications, du terminal et des cibles du navigateur. `fonts.rendering` stocke les préférences d'anticrénelage, de hinting, d'ordre des sous-pixels et de DPI. `fonts.conf`, la règle fontconfig, les paramètres du bureau et les blocs de polices gérés à l'intérieur des feuilles de style Waybar sont des projections : le Centre de Contrôle conserve `/fonts/targets/*` à travers `argvus-config`, et le projecteur ne réécrit que son bloc délimité.

Les préférences de Hyprland utilisent `keyboard_shortcuts` pour les raccourcis, `hyprland.input` pour les paramètres de la souris et du pavé tactile, et `hyprland.keyboard` pour la disposition XKB, la variante et les options. Un ID de liaison de manifeste correspond à un `config_key` stable (par exemple `window.close` → `close_window` et `window.drag_mouse` → `drag_window__floating_window_only`). Les préférences d'affichage utilisent `displays.monitors`, `displays.workspaces` et `displays.primary_monitor`. Les fichiers TOML natifs et les fichiers Lua générés sous `data/generated/` restent des projections utilisées par Hyprland.

Chaque interrupteur `effects` est activé par défaut (animations, flou, télémétrie des widgets et chaque surface de transparence), même sur les machines virtuelles et les sessions à faible consommation ; l’ombre des fenêtres (`shadow_enabled`) et le tremblement (`wobble_enabled`) sont désactivés par défaut. Les préférences d’effet utilisent `effects` pour l’interrupteur principal des animations (`animations`, appliqué à `hyprland.lua` à partir du `hypr/state.lua` projeté, indépendant de l’interrupteur des effets, qui est `blur_global_enabled` plus le `transparency_<surface>_enabled` de `taskbar`, `control-panel`, `widget-telemetry`, `terminal` et `launchers`, et est activé lorsque l’un d’eux l’est ; `argvus effects-toggle animations|effects` les bascule), l’interrupteur global de flou (`blur_global_enabled`), les paramètres de flou d’Hyprland (`blur_size`, `blur_passes`, `blur_brightness`, `blur_noise`, `blur_contrast`, `blur_vibrancy`, `blur_vibrancy_darkness`), l’ombre des fenêtres (`shadow_enabled`, `shadow_sharp`, `shadow_color`, `shadow_range`, `shadow_render_power`, `shadow_offset_x`, `shadow_offset_y`, `shadow_scale`), les valeurs des animations (`animation_<leaf>_speed`, `_curve`, `_style`, `_popin` et `_enabled` pour chaque animation, plus `animation_bezier_x1` à `animation_bezier_y2` et `animation_spring_mass`, `animation_spring_stiffness` et `animation_spring_dampening` pour les courbes personnalisées) et les valeurs de transparence par surface. Les effets sont la propriété du thème, pas de la variante : Sticky et Spaced utilisent la même projection d'effets `<theme>.conf` tandis que leurs projections de disposition diffèrent. Les surfaces incluent actuellement `taskbar`, `control-panel` et `widget-telemetry` ; les valeurs de transparence sont des pourcentages de `0` à `100` (le Centre de Contrôle et les surfaces du lanceur par défaut à `15`, les autres à `50`), les paramètres de flou sont des nombres dans les plages déclarées dans `schema.json`, et les états activés sont des booléens. Les valeurs des animations sont également validées par `argvus-config` : les vitesses sont nombres de `1` à `10` (décimales autorisées), les pourcentages des pop-in sont des nombres entiers de `1` à `100`, les courbes doivent être des noms de la page Animations, les styles doivent être des noms de la page Animations et acceptés par la propre feuille de l'animation (par exemple `popin` est refusé pour `animation_workspace_in_style`, `gnomed` est accepté uniquement pour l'ouverture et la fermeture de fenêtre, et les variantes de glissement sont refusées sur la fenêtre et les vantaux de calque), les points Y de Bézier restent entre `-1` et `2`, la plage analysée par Hyprland 0.56.2 et les autres paramètres de courbe restent dans leurs plages documentées. Les feuilles `layers` et `switching` n'existent plus, donc les configurations enregistrées avant leur suppression perdent les clés `animation_layers_enabled`, `animation_layers_speed`, `animation_layers_curve` et `animation_switching_*` lors de leur chargement. Les styles en dehors de l'ensemble actuel sont supprimés et les points Y de Bézier en dehors de `-1` à `2` sont bloqués, de sorte que les valeurs par défaut du package s'appliquent à nouveau à tout ce qui a été supprimé.

Les préférences d’alimentation utilisent `power.screen_off_minutes`, `power.lock_minutes`, `power.keep_awake`, et les politiques du couvercle/bouton d’alimentation. Les formulaires projetés sont `data/generated/hypr/hypridle.conf` et le marqueur `data/power/keep-awake` ; `.keep-awake` est uniquement une entrée de migration. `session.language` est retiré : les valeurs existantes sont ignorées, les nouvelles écritures sont rejetées, et aucun fichier de compatibilité linguistique n’est projeté. Définissez `LANG` dans l’utilisateur `locale.conf` à la place. Les préférences du calendrier utilisent `calendar` et sont projetées vers le TOML utilisateur de la barre des tâches du calendrier. Les préférences des périphériques amovibles utilisent `removable_devices` ; l’application des périphériques amovibles considère cette section comme une substitution utilisateur aux valeurs par défaut incluses.

`migrate` importe une seule fois les anciens fichiers racine et sections, en préservant une valeur canonique existante. Les anciens `defaults.json`, `/defaults`, `/hyprland/keybindings`, `keybindings.toml`, fichiers point, `generated/` et répertoires de composants sont déplacés sous `data/` ; les rechargements répétés ne recréent pas l'ancien agencement racine. Sélectionner à nouveau le thème actuel est idempotent : si l'état canonique et les projections sont déjà en accord, le plan de projection indique aucune section modifiée et la session évite le démarrage de l’écran d’accueil et des services.

La puissance suit le même contrat de projection : le Centre de Contrôle écrit `/power` dans `config.json`, `argvus config project` rend `data/hypr/hypridle.conf` de manière atomique, et ce n’est qu’alors que `argvus sessionctl` peut redémarrer `argvus-hypridle.service`. La suppression de ce fichier est récupérable avec `argvus config project` ; une valeur de `0` omet le délai correspondant et signifie Jamais.

L'apparence de l'écran de verrouillage est projetée par Rust à partir du thème canonique, de l'accent, des effets, du fond d'écran et de l'état de la police dans `data/hypr/hyprlock.conf`. L'entrée historique `hyprlock-theme.sh` est uniquement pour la compatibilité et délègue à `argvus config lock apply` ; `--invalidate` invalide seulement le cache de fond d'écran flouté dérivé.

Les modifications du délai d'extinction utilisent `/power/lock_minutes` et `/power/screen_off_minutes` comme source de vérité. `argvus config project` réutilise le moteur Hypridle existant, et l'entrée `idle-timeout.sh` n'est qu'un adaptateur de compatibilité ; le redémarrage du service reste délégué à `argvus sessionctl`.

L'apparence des notifications est dérivée du thème canonique, de l'accent et des effets dans `data/notifications/dunstrc`. Le fichier historique des notifications `theme.sh` n'expose que des fonctions de compatibilité qui appellent `argvus config notifications` ; il n'analyse ni ne modifie la configuration de Dunst.

Les adaptateurs orientés matériel tels que la découverte d'affichage et le contrôle Bluetooth sont désormais importés par le domaine d'affichage Rust : `nwg-displays` est un importateur externe, tandis que `/displays` et `data/generated/hypr/monitors.lua` restent respectivement l'état canonique et dérivé. Les outils d'affichage modifient la disposition avec `argvus config monitor-set`, `monitor-primary`, `monitor-workspaces` et `monitor-remove` ; les commandes d'affichage sont `argvus config display status|list-modes|set|apply|apply-nwg|migrate-nwg|nwg-import|nwg-apply`. Le fichier Hyprland est projeté depuis `/displays` ; aucun script shell ne l'écrit plus. La sélection du fond d'écran, le mode groupe d'utilitaires de la barre des tâches et la recherche d'application par défaut utilisent également les API `argvus-config` ; leurs points d'entrée historiques Shell sont des interfaces UI ou de compatibilité. Keep Awake est une préférence persistante `/power/keep_awake` projetée vers le marqueur `data/power/keep-awake`, tandis que lock-DPMS est un état d'exécution de session sous
`$XDG_CACHE_HOME/argvus/lock-dpms`. La disposition du clavier est maintenant gérée par l'aide Rust `argvus-keyboard-layout` : l'ordre configuré provient de la projection d'entrée générée par Hyprland ou du repli de la locale, tandis que la disposition active reste l'état d'exécution de la session découvert par Hyprland. Le point d'entrée du cycle et le démon de disposition du clavier ne sont que des wrappers de compatibilité ; le service utilisateur exécute directement le démon Rust et `argvus sessionctl keyboard-layout cycle` est la commande stable pour les appelants.

L'état d'exécution Bluetooth suit la limite de l'adaptateur externe mais ne fait pas partie de `argvus-config` : `argvus bluetooth` gère l'analyse typée et la validation autour de `bluetoothctl`, les tentatives limitées, les vérifications des services système et l'état RFKill. `argvus bluetoothctl` est le point d'entrée. L'alimentation de l'adaptateur, les appareils appariés et les appareils connectés restent des états d'exécution BlueZ plutôt que des états JSON canoniques ; la validation Bluetooth en direct sur le matériel dépend de l'environnement.
## Identité et réconciliation en runtime

Utilisez `argvus config status --verbose` pour inspecter le binaire exact, l'identité de la build, la racine de configuration, la racine générée, la racine des assets, la source du manifeste et la génération active. Il rapporte les états de projection `synchronized`, `stale`, `incomplete`, `legacy` et `pending-transaction`.

`argvus config --version` signale la version du paquet, la révision de la compilation lorsque fournie par l'environnement de compilation, et le profil. Les tests de développement doivent injecter explicitement leur binaire et leurs ressources ; ils ne doivent pas utiliser silencieusement `argvus config` ou `~/.config/argvus`. `apply` normalise en toute sécurité les identifiants combinés hérités tels que `argvus-dark-spaced` en valeurs séparées de thème et de variante avant de reconstruire les projections. L'identité des ressources emballées n'est pas enregistrée dans un manifeste dans cette version.

## Lumière nocturne

Ouvrez **Bureau & Entrée > Affichages > Lumière nocturne** ou lancez `argvus control-center displays night-light`. Configurez la température, l'activation manuelle et les horaires locaux fixes via `argvus config display night-light`. Voir [Lumière nocturne](/docs/user-guide/night-light/) pour les commandes, l'état et les exigences d'exécution.

### Paramètres de wobble (Hyprland 0.57+)

`effects.wobble_enabled`, `wobble_stiffness`, `wobble_damping`, `wobble_mass`, `wobble_intensity` et `wobble_mesh` se projettent directement dans `hypr/state.lua`. Utilisez `argvus effects-toggle wobble-settings get` pour les lire, ou `argvus effects-toggle wobble-settings set enabled=enabled stiffness=100 damping=10 mass=1 intensity=1 mesh=8` pour sauvegarder et demander un rechargement. `enabled=disabled` désactive l'effet.

Le Centre de Contrôle partage le brouillon des Animations : `animations-settings get` retourne également les entrées `wobble_*`, et `animations-settings set` accepte ces noms complets en plus de ses arguments d'animation existants. Cela persiste tout le brouillon dans un patch validé et demande un rechargement. Par exemple :

```sh
argvus effects-toggle animations-settings set enabled=enabled wobble_enabled=true wobble_intensity=0 wobble_mesh=8
```
Le `enabled` global contrôle les animations ; `wobble_enabled` est sa préférence indépendante. Désactiver les animations globales supprime le balancement dans Lua sans effacer cette préférence. Les valeurs invalides rejettent l'ensemble du patch. Les clés manquantes reçoivent les valeurs par défaut ARGVUS sans remplacer les choix existants. Voir [Appearance](/docs/user-guide/appearance/#window-wobble) pour les limites et valeurs par défaut.

### Raccourcis clavier créés par l'utilisateur

`/keyboard_shortcuts` conserve ses valeurs existantes de type chaîne ou null pour les actions officielles du manifeste. Les raccourcis clavier globaux personnalisés utilisent un objet `/hyprland/custom_shortcuts` séparé, stocké dans `config/hyprland.json`. Ses identifiants stables `custom_` sont indépendants des descriptions et des locales. Chaque enregistrement possède `description`, `keys`, `enabled` et un `action` typé :

```json
{
  "custom_editor": {
    "description": "Open my editor",
    "keys": "SUPER + CTRL + E",
    "enabled": true,
    "action": { "type": "command", "command": "editor --new-window" }
  },
  "custom_float": {
    "description": "Toggle floating",
    "keys": "SUPER + CTRL + F",
    "enabled": false,
    "action": {
      "type": "dispatcher",
      "dispatcher": "window.float",
      "arguments": { "action": "toggle" }
    }
  }
}
```
L'exemple est la valeur de `/hyprland/custom_shortcuts`, pas un document de configuration complet. Les noms de dispatchers proviennent des usines de dispatchers Lua prises en charge ; les arguments sont un objet JSON optionnel ou `null`, jamais du code évalué. Les raccourcis personnalisés n'ont pas de sous-carte, de liaison de souris ni de drapeaux avancés. La projection écrit `data/generated/hypr/custom-shortcuts.lua` comme une table de données échappée, y compris une table vide lorsque le dernier enregistrement est supprimé. La projection complète et `project --only keyboard_shortcuts` la génèrent toutes deux. Hyprland enregistre les entrées activées avec `hl.bind` ; les échecs d'enregistrement facultatifs de fichiers et d'individus sont contenus et enregistrés.

Le Centre de Contrôle crée/modifie des enregistrements et désactive les conflits confirmés en utilisant une seule transaction `argvus config patch`. Il enregistre avant d'appliquer via le service de configuration existant. Une application échouée n'annule pas un enregistrement réussi. La restauration des paramètres par défaut officiels préserve les enregistrements personnalisés et est refusée si cela entrait en collision avec un raccourci personnalisé actif. Les descriptions des utilisateurs restent du texte littéral dans toutes les langues et dans la fiche générée effective.

## Mode fenêtre globale

Ouvrez **Centre de Contrôle → Bureau & Entrée → Hyprland → Fenêtres → Mode** et sélectionnez **Fenêtres en mosaïque (Par défaut)** ou **Fenêtres flottantes**. Le même sélecteur est disponible depuis l'icône du mode fenêtre dans le groupe d'utilitaires de la barre des tâches.

La préférence s'applique à chaque moniteur et espace de travail et est mémorisée après la déconnexion. Le mode flottant convertit les fenêtres d'application existantes et les nouvelles fenêtres ouvertes. Le mode en mosaïque restaure uniquement les fenêtres converties par cette politique ; les boîtes de dialogue et autres fenêtres déjà flottantes restent flottantes. Une fenêtre épinglée explicitement pendant que le mode flottant est actif reste également flottante pour préserver son épingle. La disposition en mosaïque est recalculée ; son arbre de division exact précédent n'est pas restauré.

`SUPER + SHIFT + Space` conserve son comportement normal par fenêtre en mode en mosaïque. En mode flottant, il ne renvoie pas une fenêtre vers la mosaïque. Les panneaux, la barre des tâches et les notifications sont des surfaces de type layer-shell et ne sont pas affectés. Le mode plein écran et la maximisation continuent d'utiliser le comportement normal de Hyprland.

La préférence canonique est `/hyprland/window_mode` dans `$XDG_CONFIG_HOME/argvus/config/hyprland.json`. Sa projection générée est `data/generated/hypr/window-mode.lua` ; ne la modifiez pas. Le registre de restauration par fenêtre se trouve sous `$XDG_RUNTIME_DIR/argvus`, limité à l'instance du compositeur, et survit aux rechargements de configuration. Il n’est pas réutilisé dans une nouvelle session.

Si l'application est en attente ou échoue, l'interface distingue cela de la préférence enregistrée. Sélectionnez de nouveau l'option pour réessayer. Sans compositeur, la préférence est enregistrée pour la prochaine connexion.

```sh
argvus config window-mode get
argvus config window-mode status --json
argvus config window-mode set floating
argvus config window-mode set tile
argvus config window-mode toggle
```
## Boutons de fenêtre et barre d'application

Le mode flottant ajoute une barre de titre ARGVUS avec **Réduire**, **Agrandir / Restaurer**, et **Fermer** aux fenêtres d'application normales. Double-cliquer sur la barre de titre bascule la maximisation. La décoration supplémentaire est masquée en mode mosaïque et plein écran. Les applications qui dessinent leurs propres contrôles peuvent afficher les deux barres ; les contrôles ARGVUS uniformes sont intentionnels.

Une deuxième Waybar apparaît sur chaque moniteur, sur le bord opposé à la barre des tâches principale. Elle ne liste que les fenêtres appartenant à l’espace de travail actuel de ce moniteur, y compris les fenêtres minimisées de cet espace de travail. Un espace de travail spécial ouvert a la priorité. Les fenêtres épinglées restent visibles ; les onglets de groupe masqués ne créent pas d’entrées supplémentaires, et un groupe minimisé a une seule entrée de récupération. Les onglets à l’intérieur d’une application ne sont pas énumérés séparément. Les entrées minimisées conservent leur association d’origine avec le moniteur/l’espace de travail. Cliquez sur une fenêtre inactive pour l’activer, sur une fenêtre active pour la minimiser, ou sur une fenêtre minimisée pour la restaurer ; son menu contextuel offre les options restaurer, minimiser, maximiser/restaurer et fermer. Les icônes d’application inconnues se remplacent par un symbole de fenêtre. L’activation ou la restauration d’une entrée préserve la position du pointeur. La barre secondaire utilise le CSS primaire effectif (couleurs, polices, bordure et arrondi), avec sa hauteur et des marges miroir sur le bord opposé.

`SUPER + Alt + Up` place la barre principale en haut et la barre de fenêtre en bas. `SUPER + Alt + Down` fait l'inverse. Répéter une direction laisse les positions inchangées. Le carrelage n'a que la barre principale. Revenir au carrelage restaure toutes les fenêtres minimisées avant de supprimer la barre de fenêtre, puis préserve la politique de carrelage/flottante originale.

La minimisation utilise l’espace de travail réservé `special:argvus-minimized`. N’utilisez pas cet espace de travail manuellement. Le processus `argvus windows service` enregistre l’identité, l’espace de travail/écran d’origine, la géométrie, l’état plein écran/maximisé et l’état d’épinglage dans un registre d’exécution spécifique à l’instance du compositeur sous `$XDG_RUNTIME_DIR/argvus`. Les enregistrements survivent à un redémarrage du service ou à un rechargement de configuration, mais ne traversent pas les sessions de compositeur. La fermeture d’une fenêtre supprime son enregistrement. Si un écran disparaît, la restauration utilise une sortie disponible. Une barre de récupération manquante ou défaillante empêche de nouvelles minimisations ; la restauration échouée conserve l’enregistrement et la barre de récupération.

La session possède `argvus-windows.service` et `argvus-window-bar.service`. Ce dernier maintient son enfant Waybar inactif dans Tile. Le plugin hyprbars est intégré par `argvus-hyprland`, compilé pour l’ABI Hyprland 0.57.0 sur x86_64 ; il n’est pas compilé dans le répertoire personnel de l’utilisateur. Reconstruisez le plugin lors d’une mise à jour de l’ABI du compositeur.Commandes : `argvus windows list --json`, `watch --json`, `position top|bottom`, `restore-all` et `minimize|restore|activate|maximize|close ADDRESS`. Les adresses des fenêtres proviennent de la liste. `position top --persist` stocke/projette la préférence sans redémarrer la barre principale. Les détails de configuration et d'erreurs d'exécution sont écrits dans le journal ; les échecs d'action visibles affichent également une notification traduite.

En **Espacé + Flottant**, les fenêtres respectent les marges extérieures et les zones réservées par les deux barres. Les limites incluent la bordure et le titre, qui se situe à l'intérieur de la même bordure de fenêtre. Le fait de maximiser via ARGVUS contrôle remplit cette zone avec des marges ; la restauration récupère la géométrie précédente, y compris après le redémarrage d'un service. Le plein écran utilise toujours toute la sortie. Les fenêtres flottantes peuvent se chevaucher ; les interstices internes du carrelage ne les séparent pas.

Les marges intérieures réservées par les barres ne sont pas comptées deux fois dans les espacements extérieurs de la fenêtre. Avec un espacement symétrique, la distance visible entre la fenêtre et la barre est égale en haut et en bas, y compris après permutation des barres.

`SUPER + Alt + Space` bascule le mode global Carrelage/Flottant, en utilisant la même préférence que la barre des tâches et le Centre de contrôle. La commande équivalente est `argvus config window-mode toggle`. Les invocations simultanées sont sérialisées ; le basculement utilise la préférence enregistrée même lorsque l’application est en attente. Le raccourci peut être modifié ou désactivé dans les Raccourcis clavier ; `SUPER + Shift + Space` conserve son action pour la fenêtre individuelle.
