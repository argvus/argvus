---
title: Accueil
description: Le greeter de connexion qui a accueilli démarre, expédié dans le paquet argvus.
---
`argvus-greeter` est l'écran d'accueil de connexion pour greetd. C'est une interface terminale axée sur le clavier qui découvre les sessions Wayland installées, affiche les comptes avec leurs avatars et applique le thème ARGVUS actif pour le compte sélectionné.

Il fait partie du paquet `argvus`, le paquet unique du bureau. Son code source se trouve dans `crates/argvus-greeter` du dépôt `argvus`, en tant que membre de l'espace de travail racine Cargo, et il est construit avec le reste de `argvus`.

L'écran d'accueil ne gère pas lui-même l'authentification des utilisateurs. greetd gère l'authentification via PAM. Le lanceur ouvre l'écran d'accueil avec `argvus tui-terminal`, donc l'écran d'accueil n'a pas besoin d'un programme de terminal séparé.

Les fichiers de configuration de l'écran d'accueil sont installés sous `/etc/argvus`. Utilisez `argvus greeter-setup --help` pour voir les options de configuration que le paquet fournit. Pour le flux de connexion, voir [Sessions](/docs/user-guide/sessions/).

La socket de transfert de connexion autorise chaque client avec les identifiants du pair noyau. Seul root, le propriétaire de l'assistant et le compte authentifié par greetd peuvent envoyer des commandes de contrôle. Les autres comptes locaux sont refusés même si la socket permet les connexions entre utilisateurs. Une trame DRM préparée reste liée à son descripteur d'origine ; l'assistant attend ce descripteur avant de prendre en charge l'affichage.

Lorsque le champ d'authentification est actif, les caractères imprimables (y compris `j` et `k`) sont saisis dans la réponse. Tab et Shift-Tab déplacent le focus ; Backspace supprime le dernier caractère. Cela s'applique aux invites secrètes et non secrètes.

La liste des comptes défile pour garder le compte sélectionné visible lorsque vous naviguez ou redimensionnez le terminal.
