---
title: Dépannage des thèmes et des fonds d'écran
description: Diagnostiquer la propagation des thèmes et les fichiers d'apparence générés.
---
Vérifiez la section canonique `appearance` dans `$XDG_CONFIG_HOME/argvus/config.json`, puis inspectez les fichiers générés sous `$XDG_CONFIG_HOME/argvus/data/generated`. Les fichiers dot hérités sont uniquement des entrées de migration.

`argvus-config` est le seul écrivain sous `data/generated/`. Pour récupérer un thème ou un fond d'écran cassé, reconstruisez l'ensemble de la génération à partir du document canonique et appliquez le résultat au lieu de modifier directement les consommateurs générés :

```sh
argvus config validate
argvus config apply
argvus sessionctl reload
```
`argvus theme-switch` (qui exécute `argvus config theme-apply`) et `argvus config accent` sont des orchestrateurs : ils valident, verrouillent, affichent l'écran de démarrage, s'engagent via `argvus-config`, puis réconcilient les consommateurs externes qui ne peuvent pas lire `config.json`. Ils n'écrivent jamais `data/generated/`, et la transaction du thème n'écrit pas les fichiers GTK, de stockage ou `mode.css` via des scripts séparés. Utilisez `argvus config accent --apply` pour relancer uniquement la réconciliation des accents.

Les modifications de thème officielles appliquent leurs valeurs d'apparence enregistrées et mettent à jour les consommateurs en cours d'exécution affectés. L'application d'un thème personnalisé redémarre la barre des tâches, recharge Hyprland, le papier peint et les services du Panneau de configuration. Le changement vers Sticky ou Spaced réapplique l'espacement, les bordures et les marges de la barre des tâches de ce mode, puis redémarre la barre des tâches, la télémétrie et les services de recharge de Hyprland. Les couleurs de notification de Dunst sont l'exception lorsque le mode Ne pas déranger est actif : leur rechargement est différé jusqu'à ce que le mode NPD soit désactivé. Si une autre surface conserve les anciennes couleurs, vérifiez que la version du thème est active et que c'est bien le thème que vous avez choisi, puis relancez les rechargements :

```sh
cat "${XDG_CONFIG_HOME:-$HOME/.config}/argvus/data/generated/theme/current/build.toml"
argvus config theme-runtime consumers
```
Chaque fichier thématique provient de la compilation : un thème qui n'est pas installé (aucun `theme.toml` sous `/usr/share/argvus/themes/` ou `~/.local/share/argvus/themes/`) ou qui ne s'affiche pas est refusé, et le thème précédent reste. `argvus theme check DIRECTORY` montre le modèle et la ligne de chaque erreur de rendu. Une barre Waybar recharge ses couleurs uniquement avec `reload_style_on_change` dans son JSONC. Si un commutateur semble bloqué, vérifiez s'il fonctionne encore ; il s'enregistre dans `theme-switch.active`, à côté de son verrou :

```sh
cat "${XDG_CACHE_HOME:-$HOME/.cache}/argvus/theme-switch.active"
ps -ef | grep -E 'theme-switch|theme-apply'
```
Un pid dans le marqueur qui est encore actif signifie que l'interrupteur n'a pas terminé. Un interrupteur attend au maximum 60 secondes pour le verrouillage du thème et pour la projection, et 5 secondes pour les commandes de rechargement ; un marqueur dont le processus est terminé est obsolète et le prochain interrupteur l'ignore. Incluez la sortie des commandes ci-dessus lors du signalement du problème.

Si seul le greeter préconnexion est blanc ou utilise le thème de secours, comparez la source privée avec sa projection publique :

```sh
uid=$(id -u)
argvus config get /appearance/theme --raw
sudo cat "/var/lib/argvus/greeter/themes/$uid"
sudo cat "/var/lib/argvus/greeter/themes/$uid.accent" 2>/dev/null || true
```
Réparez la projection sans modifier la configuration greetd :

```sh
sudo argvus greeter-setup --sync-themes
```
Ce fichier est publié pour la pré-authentification par l'orchestrateur d'apparence et se trouve en dehors de `data/generated/`, donc un simple `argvus config apply` ne le rafraîchit pas.

Si le greeter signale `Invalid color name: 'null'` dans un cache Kitty, mettez à jour le paquet `argvus terminal` et régénérez le cache en rouvrant le greeter. Un accent canonique absent est une valeur par défaut, pas une couleur Kitty ; le lanceur de terminal actuel rejette le JSON `null` et les anciennes valeurs invalides avant d'écrire `active_tab_foreground`.

Si le terminal par défaut cesse de s'ouvrir après plusieurs changements de thème, inspectez le cache Kitty dérivé au lieu de le modifier manuellement :

```sh
argvus config get /appearance/theme --raw
ls -l "${XDG_CACHE_HOME:-$HOME/.cache}/argvus/argvus-terminal"
argvus terminal --apply
argvus terminal --print-config
```
La commande ci-dessus lit la préférence canonique. `.active-theme` n'est qu'une projection de compatibilité. Le cache est régénéré par `argvus terminal` et synchronisé avec l'application du thème. Un `/usr/lib/argvus/loading-theme/splash` manquant ou obsolète indique une mise à jour de package incomplète ; reconstruisez et installez les packages coordonnés avant de juger le résultat d'une session en direct.

Si les bordures des fenêtres gardent la couleur du thème précédent, ne modifiez pas manuellement le fichier Lua de Hyprland. Les changements de thème et d'accent sont projetés depuis `config.json`, puis `argvus sessionctl reload` recharge Hyprland et réapplique les couleurs des bordures actives, inactives et des groupes tout au long du cycle de vie de la session :

```text
config.json → projection plan → Hyprland reload → hyprctl reapply of active/inactive/group borders
```
Les *préférences* de bordure sont modifiées via `argvus config borders` (ou son délégué `borders-switch.sh`) dans le Centre de contrôle et le Panneau de configuration ; le rechargement réapplique les couleurs engagées tout au long du cycle de vie de la session et ne les redérive pas.

Inspectez le plan et les valeurs effectives du compositeur :

```sh
jq . "${XDG_STATE_HOME:-$HOME/.local/state}/argvus/config-projection.json"
hyprctl -j getoption general:col.active_border
hyprctl -j getoption general:col.inactive_border
argvus sessionctl reload
```
Si `hyprctl` ne peut pas atteindre le socket Hyprland actuel, la projection de bordure est signalée comme échouée et les composants dépendants ne sont pas considérés comme rechargés avec succès. Vérifiez le journal de la session utilisateur avant de réessayer.
