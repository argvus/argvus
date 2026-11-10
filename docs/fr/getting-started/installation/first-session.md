---
title: Première séance
description: Démarrez ARGVUS après l'installation.
---
Pour une connexion graphique, configurez l'intégration greetd empaquetée avec `argvus greeter-setup --help` et utilisez le flux du gestionnaire d'affichage décrit dans [Sessions](/docs/user-guide/sessions/)].

Pour une session TTY, connectez-vous sur un terminal virtuel et exécutez :

```sh
argvus --start-desktop
```
Après la connexion, utilisez `argvus sessionctl status` pour inspecter la cible utilisateur ARGVUS. Si le compositeur ou le shell ne démarre pas, suivez [dépannage de session](/docs/user-guide/troubleshooting/session-startup/).
