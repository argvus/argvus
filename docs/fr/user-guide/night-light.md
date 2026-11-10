---
title: Veilleuse
description: Ajustez la température des couleurs et les horaires locaux fixes sur Hyprland.
---
Ouvrez **Centre de contrôle > Bureau et saisie > Affichages > Lumière nocturne**, recherchez **Lumière nocturne** ou exécutez `argvus control-center displays night-light`.

La Lumière nocturne et son programme sont initialement désactivés. Choisissez une température de **1000 à 20000 Kelvin** (par défaut **4500 K**). Les valeurs plus basses donnent des couleurs plus chaudes. La page affiche l'activation manuelle enregistrée, l'état effectif et la prochaine transition. Entrer modifie une valeur ; Échap l'annule ou revient aux Affichages. Les valeurs invalides sont refusées avant l'enregistrement. Les modifications sont enregistrées et appliquées immédiatement. Si l'application échoue après l'enregistrement, la préférence enregistrée reste visible à côté de l'état effectif réel et de l'erreur.

## Programmation et actions rapides

Le programme fixe utilise le fuseau horaire local actuel. Le début est inclus et la fin est exclue ; **20:00–07:00** traverse minuit. Le début et la fin doivent être différents et utiliser `HH:MM`. Aucune localisation ni calcul solaire n'est utilisé.

Utilisez la carte **Affichage** du Panneau de configuration ou **SUPER + Maj + B** pour des basculements rapides. Le sélecteur de luminosité conserve toutes ses actions de luminosité et ajoute la Lumière nocturne lorsque le backend est disponible, y compris sur un écran sans backend de luminosité. Il n'y a pas de module permanent dans la barre des tâches.

Lorsque la planification est désactivée, l'activation manuelle est persistante. Lorsque la planification est activée, un basculement rapide crée une surchage temporaire jusqu'à la prochaine transition programmée. Changer l'horaire ou la température supprime cette surchage. La déconnexion, une nouvelle instance de compositeur ou une transition suivante modifiée (y compris un changement de fuseau horaire) l’invalide également.

## Commandes et état

Toutes les interfaces utilisent la même commande publique :

```sh
argvus config display night-light status
argvus config display night-light enable
argvus config display night-light disable
argvus config display night-light toggle
argvus config display night-light temperature 4500
argvus config display night-light schedule on 20:00 07:00
argvus config display night-light schedule off 20:00 07:00
argvus config display night-light apply
argvus config display night-light apply-schedule
```
`status` retourne le schéma JSON version 1 : `available`, `unavailable_reason`, `settings` (activé, température, horaire_activé, début, fin), `active`, `effective_temperature`, `mode` (`manual` ou `scheduled`), `manual_override`, `next_transition` (ISO 8601 avec décalage), `minimum_temperature` et `maximum_temperature`. Les champs effectifs sont nuls lorsqu'ils ne sont pas disponibles ; la température effective est nulle lorsque l'identité est active. Les échecs de disponibilité sont signalés dans status ; les échecs d'action retournent une valeur non nulle avec un diagnostic.

La préférence canonique est `/displays/night_light`, stockée de manière atomique par `argvus config` dans `$XDG_CONFIG_HOME/argvus/config/displays.json`. Les remplacements volatiles et le verrou d'action se trouvent sous `$XDG_RUNTIME_DIR/argvus/night-light/`. Les fichiers de surveillance générés sont indépendants. Ne pas éditer les fichiers générés pour configurer cette fonctionnalité.

## Exécution

Nécessite Hyprland et **hyprsunset >= 0.4.0**. La session démarre le `hyprsunset.service` en amont une fois, applique l'état enregistré via `argvus-night-light-init.service`, et évalue les horaires chaque minute via `argvus-night-light.timer` et `argvus-night-light.service`. Les changements d'horloge et de fuseau horaire déclenchent également l'évaluation. Avec la planification désactivée, l'opération périodique laisse l'état manuel intact. Les applications répétées interrogent l'identité/température effective et sautent les écritures inchangées. Aucun rechargement du compositeur n'est nécessaire.

ARGVUS fournit un drop-in systemd pour le service upstream : il commence par `--identity` et isole sa découverte de configuration dans `/usr/share/argvus/session/config/night-light`. Cela évite les profils natifs concurrents et laisse intacts les fichiers personnels `hyprsunset.conf`. Le drop-in s'applique à ce service upstream tant que le paquet de session ARGVUS est installé, y compris pour les démarrages en dehors d'ARGVUS. Les surcharges systemd utilisateur existantes peuvent avoir la priorité ; inspectez `systemctl --user cat hyprsunset.service` lors du diagnostic de cette situation. Hyprsunset 0.4.0 rejette l'argument de chemin séparé de `--config`, de sorte que la découverte de configuration est isolée avec des variables XDG locales au service.

Le backend utilise l'IPC officiel Hyprsunset (`temperature`, `identity`, `identity get`), avec une E/S limitée et une vérification après modifications. Aucun shader, backend de compositeur alternatif, écriture de configuration système ou élévation de privilèges n'est utilisé. Les contrôles indisponibles sont désactivés dans le Centre de Contrôle et masqués dans la carte Affichage.
