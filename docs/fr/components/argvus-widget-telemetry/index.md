---
title: Télémétrie du widget
description: Les blocs de télémétrie de Waybar et leur bascule, via le widget-telemetry d'argvus.
---
Le widget de télémétrie est l'information système affichée dans la barre des tâches (en-tête, clés, système, CPU et GPU, mémoire, stockage, processus, réseau et le tableau de bord du développeur). Il est en cours d'exécution dans le package `argvus` : il n'y a pas de collecteurs de shell.

## Commandes

```text
argvus widget-telemetry block BLOCK ...   prints the text of one block (used by the bar)
argvus widget-telemetry toggle ...        switches the blocks on and off, and their order
```
`block` est le chemin critique de la barre : il est appelé à chaque actualisation. `toggle` change les blocs que la barre affiche et déplace la fenêtre de défilement.

Les blocs sont `header`, `keys`, `system`, `cpu_gpu`, `memory`, `storage`, `processes`, `network` et `dev_dashboard`. La barre des tâches utilise également les sondes `cpu-temp` et `gpu`. La sonde GPU accepte `--present` (un GPU pris en charge est visible dans DRM/sysfs), `--available` (un GPU et la commande btop sont disponibles pour le panneau), `--available-temp` (une température attribuable peut être lue), et `--temp` (la température valide la plus élevée, ou `N/A`).

Les températures du GPU sont associées au périphérique PCI du GPU. Les relevés propriétaires NVIDIA utilisent les identifiants de bus `nvidia-smi`/NVML ; les relevés NVIDIA nouveau, AMD et Intel utilisent hwmon seulement lorsque le lien du périphérique sysfs correspond de manière unique à ce GPU. La température dans la barre des tâches est la valeur maximale valide parmi les GPU détectés. Les relevés manquants, invalides ou ambigus restent indisponibles. La température propriétaire NVIDIA nécessite le fournisseur du pilote ; la température AMD et Intel dépend du noyau exposant un capteur hwmon lié. La température Intel n'est pas fournie par le panneau GPU de btop 1.4.7.

## Démarrage avec la session

L'unité utilisateur `argvus-widget-telemetry.service` démarre avec la session uniquement lorsque le widget est activé. L'interrupteur principal est `effects.widget_telemetry_enabled` dans la configuration, qui est activé par défaut, donc un nouveau compte démarre le widget à la connexion.

Une fois que le commutateur a écrit `$XDG_CACHE_HOME/argvus/waybar/widget-telemetry-state`, ce fichier décide à la place. Seule une première ligne de `enabled` démarre le widget. Les anciens fichiers `desktop-telemetry-state` et `sysinfo-state` sont toujours lus lorsque le fichier actuel est manquant.

Les étiquettes de télémétrie sont compactes dans les dix langues prises en charge. Les identifiants techniques tels que CPU, GPU, IP, DE, WM et Kernel gardent leur signification technique. Le temps de fonctionnement est lu depuis `/proc/uptime` et affiché avec des unités abrégées (par exemple `1d 12h 49m`) ; il ne se développe plus en mots complets selon la langue du système. Les noms de modèles matériels et autres valeurs de fournisseur peuvent toujours affecter la largeur du widget.
