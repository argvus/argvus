---
title: Moniteur système
description: Ouvrez le moniteur de système ARGVUS.
---
Ouvrez le moniteur configuré avec :

```sh
argvus --system-monitor
```
La sous-commande `argvus system-monitor` fournit le point d'entrée natif et l'intégration de btop. La télémétrie affichée dans la barre des tâches est une fonctionnalité séparée fournie par `argvus widget-telemetry`. Cliquer sur ses modules CPU, Mémoire et GPU ouvre btop dans un panneau TUI ARGVUS avec un en-tête de marque et un pied de page `Esc Close`. Le mode GPU affiche jusqu'à six boîtes GPU et pas de boîtes CPU, mémoire, réseau ou processus. Appuyer sur `q` ou `Esc` ferme le panneau. L'onglet Kitty en bas montre l'indice `Press q to close` localisé. Les appels autonomes `argvus system-monitor --cpu`, `--mem` et `--gpu` lancent directement btop avec les boîtes sélectionnées.

Le panneau GPU dépend des capacités de la version de btop installée et des fournisseurs de périphériques du système. La surveillance NVIDIA nécessite le support GPU dans btop, la NVIDIA Management Library du pilote NVIDIA installé, et l'accès au périphérique. La surveillance AMD utilise ROCm SMI, qui est optionnelle et peut ne pas être installée. Les données GPU Intel sont lues depuis sysfs et dépendent de l'accès de l'utilisateur à ces interfaces ; btop 1.4.7 rapporte l'utilisation, la puissance et la fréquence d'Intel, mais ne fournit pas la température Intel. Les pilotes NVIDIA nouveau et autres peuvent exposer une température GPU à la barre des tâches via hwmon lié au périphérique, tandis que le support du fournisseur btop dépend du pilote et des bibliothèques disponibles. ARGVUS ne modifie pas les permissions des périphériques et n'exécute pas btop avec des privilèges élevés. Si aucun GPU n'est exposé, ou si btop ne rend pas la boîte GPU dans les trois secondes parce que le support ou un fournisseur n'est pas disponible, le panneau affiche un message d'indisponibilité localisé.

Le moniteur lit d'abord le thème et l'accent sélectionnés à partir de `$XDG_CONFIG_HOME/argvus/config.json`. Ses fichiers btop générés restent dans un état de cache dérivé ; `.active-theme` et `.accent-color` sont des solutions de repli pour la compatibilité. Comme btop conserve son propre fichier de thème, le moniteur est réconcilié par la transaction de thème (`argvus theme-switch`) en tant qu'adaptateur externe après que le changement canonique est confirmé ; il ne fait pas partie de l'arborescence générée.

## Contrat d'exécution

- Le thème et l'accent proviennent de la configuration canonique (` and ` and `). The files `). The files ~/.config/argvus/data/.active-theme et `.accent-color` sont des valeurs de secours). 
- La configuration générée de btop et les thèmes sont écrits dans `$XDG_CACHE_HOME/argvus/argvus-system-monitor`. Ils sont des états dérivés ; ne les modifiez pas.
- Les valeurs par défaut sont installées sous `/usr/share/argvus-system-monitor/btop`.
- Les raccourcis de module (`--mem`, `--cpu`, `--net`, `--proc`) sont en lecture seule en tant que premier argument. Passer `-c`/`--config` ou `--themes-dir` propres à btop lance btop avec ces chemins et n'écrit aucune configuration.
- `--panel [--cpu|--mem|--gpu]` intègre btop dans le shell TUI d'ARGVUS. Ce mode est utilisé par la barre des tâches ; sans `--panel`, le moniteur continue à se remplacer par btop.
