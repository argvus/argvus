---
title: Superposition de chargement
description: La superposition affichée pendant que la session se recharge directement.
---
La superposition de chargement est une surface GTK 4 layer-shell affichée pendant qu'ARGVUS recharge directement la session (`argvus sessionctl reload`). Un changement de thème ne l'utilise pas : le bureau change en place. C'est un assistant fourni dans le package `argvus` à `/usr/lib/argvus/loading-theme/splash`, et il n'est pas placé dans `/usr/bin`.

La connexion ne l'utilise pas : le splash du greeter et le spinner de démarrage de session sont le mode Wayland de l'assistant de transfert, qui dessine le même spinner sans GTK et affiche sa première image en quelques millisecondes (voir [Greeter, lock and splash](/docs/developer-guide/subsystems/greeter-lock-and-splash/)).
Avec `--spinner-only`, la superposition n'affiche que ce spinner et se ferme sur `SIGTERM`. Avec `--detached`, elle ne se ferme pas lorsque le processus qui l'a démarrée se termine ; un rechargement de session utilise les deux indicateurs et l'arrête via son fichier pid.
Il lit le thème sélectionné dans le répertoire de données ARGVUS et revient à la palette ARGVUS Dark fournie. Il signale sa disponibilité uniquement après la première image de chaque surface qu'il affiche, et non lorsque le processus démarre.
