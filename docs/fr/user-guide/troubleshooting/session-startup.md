---
title: Dépannage du démarrage de session
description: Diagnostiquer les échecs du greeter, de Hyprland et de la session utilisateur systemd.
---
Courir :

```sh
argvus sessionctl status
argvus sessionctl logs
systemctl --user status argvus-session.target
```
Vérifiez séparément le greeter, `argvus start`, Hyprland et l’unité utilisateur concernée. Une compilation de package réussie ne prouve pas que la charge utile installée ou le dépassement utilisateur actif est à jour.

Utilisez les vérifications dans cet ordre :

1. Confirmez que le greeter utilise la configuration ARGVUS greetd installée.
2. Lisez le journal de session et recherchez une configuration Lua Hyprland manquante ou invalide.
3. Vérifiez la préparation et l’état cible avec `systemctl --user status argvus-session.target`.
4. Inspectez le composant défaillant plutôt que de redémarrer tous les services en même temps.

Si `argvus-session.target` est inactif alors que la superposition de chargement est active, inspectez l’erreur de projection avant de redémarrer les services :

```sh
argvus config project
journalctl --user -u argvus-session.target -u argvus-session-prepare.service -b --no-pager
```
`argvus-config` est le seul écrivain en dessous de `data/generated/`, donc `project` est la manière autoritaire de reproduire une génération tout en diagnostiquant. Le démarrage de la session l'exécute parce qu'aucun plan engagé n'existe encore ; le plan qu'il écrit dans `${XDG_STATE_HOME:-$HOME/.local/state}/argvus/config-projection.json` est ce qu'un `argvus sessionctl reload` ultérieur applique, sans projeter à nouveau.

La session arrête le superposition de chargement lorsque la préparation critique ou le démarrage de la cible échoue. Une défaillance d'un consommateur visuel devrait soit utiliser son secours documenté, soit être rapportée explicitement ; elle ne doit pas être cachée comme une session réussie.

## Connexion lente

Si `greeter.handoff-takeover-failed` apparaît sans `handoff.master-acquired`, inspectez `handoff.master-unavailable card=<name> errno=<n>` pour l'erreur d'acquisition. Un observateur ouvert pendant qu'un autre compositeur est maître peut renvoyer errno 13 (`EACCES`) même après que ce compositeur a libéré le rôle de maître. L'assistant rouvre le même GPU pour acquérir le rôle de maître implicitement ; `handoff.master-reopened` enregistre cette récupération et un nouveau tampon de trame est préparé sur ce descripteur. Le greeter annule l'assistant échoué (`greeter.handoff-cancelled`), donc il ne peut pas ajouter une autre attente avant que le compositeur utilisateur ne démarre. Une session envoyant `release` pendant que l'acquisition est en attente reçoit `handoff.release-before-master` : l'assistant ferme ses descripteurs DRM avant d'accuser réception. Un crash du compositeur greeter dans Hyprland/Aquamarine est une découverte séparée ; l'annulation de l'assistant n'établit pas que le problème du pilote ou du compositeur a été résolu.

`greeter.vt-switch-returned active=<tty>` enregistre le VT actif lorsque l'appel switch de logind retourne. Un délai d'expiration de dessin enregistre également `active-vt=<tty>` dans son erreur. Ce sont des instantanés, pas la preuve d'un transfert DRM terminé.

Pour une comparaison avant/après, enregistrez le travail et utilisez cinq connexions par version sur le même ordinateur, le même thème et la même configuration de moniteur. Avant chaque déconnexion suivante, copiez `/run/argvus-greeter/timing.log`, le journal de session et le journal de démarrage dans un répertoire d'échantillons séparé sous `/tmp/argvus/` ; la prochaine exécution du greeter remplace son journal de temps. Enregistrez l'identifiant de démarrage avec chaque échantillon et comparez uniquement les horodatages au sein de ce démarrage. Mesurez l'authentification jusqu'au premier affichage, l'entrée du session-wrapper, le lancement final de Hyprland et l'arrêt du spinner de chargement. Comparez la médiane et le pire temps d'authentification jusqu'à l'arrêt du spinner et vérifiez visuellement les écrans noirs, les flashes ou un spinner bloqué. La durée des surfaces du bureau seule n'est pas le temps de connexion de bout en bout. Répétez les tests d'échec et de récupération dans une session jetable, sans terminer une session contenant un travail non enregistré. 

Chaque étape de démarrage enregistre sa durée dans le journal de session. Pour voir les étapes de la dernière connexion, les plus lentes en premier :

```sh
grep timing ~/.local/state/argvus/session.log | tail -30 | sort -k4 -rn
systemd-analyze --user critical-chain argvus-session.target
```
Pour synchroniser le passage du greeter au bureau, lisez les marques du greeter et de transfert ainsi que le journal. Ils utilisent tous `CLOCK_MONOTONIC`, donc ils s'alignent :

```sh
cat /run/argvus-greeter/timing.log
journalctl -b -o short-monotonic --since -2min | grep -E 'greetd|Hyprland|coredump|argvus-greeter-handoff|session-loading'
grep monotonic_ns ~/.local/state/argvus/session.log | tail -5
```

`timing.log` contient uniquement la dernière connexion : le greeter, la surface de transfert DRM (`handoff.master-acquired`, `handoff.output-found`, `handoff.first-frame`, `handoff.release-received`, `handoff.replaced-by-compositor`) et les marques `greeter.*`. Les lignes `handoff.crtc` enregistrent chaque changement de CRTC pendant que le transfert attend le maître DRM ; une ligne avec `fb=0` ou `mode_valid=false`, ou `handoff.output-found ... framebuffer=0`, signifie que le compositeur du greeter a éteint la sortie avant que le transfert ne la prenne, donc le moniteur a dû se synchroniser à nouveau. `greeter.watchdog-stop-requested` signifie que la session du greeter s'est terminée sans attendre son compositeur, et `greeter.compositor-exit-deferred` que le compositeur est demandé de quitter une fois que la surface de transfert est partie ; `greeter.compositor-exit-requested` seul signifie qu'il a été demandé de quitter immédiatement. Si les lignes greetd et logind dans le journal montrent un écart d'environ une demi-seconde entre la fermeture de la session du greeter et celle de la session utilisateur l'ouverture, le démontage du compositeur du greeter s'est exécuté pendant la connexion. C'est attendu tandis que le repère montre `compositor-in-foreground=true` : la coupure du terminal du greeter termine le compositeur avant la sortie différée. Cela se produit lorsque le greeter n'a pas pu quitter son VT avant que son compositeur ne se ferme : cherchez `greeter.vt-switch-requested` suivi de `greeter.handoff-drawing` ; lorsque le second est manquant, `greeter.handoff-takeover-failed error=<reason>` explique pourquoi. `handoff.page-flip` signifie que le transfert a remplacé le dernier cadre du greeter sans un réglage de mode ; `handoff.page-flip-failed errno=<n>` suivi de `handoff.modeset` signifie que le noyau a refusé le changement de page. `handoff.overlay plane=<id>` puis `handoff.retired` signifient que la superposition a caché les premiers cadres de Hyprland jusqu'à ce que le spinner de session soit à l'écran ; `handoff.overlay-unavailable` signifie que le CRTC n'avait pas de plan de superposition libre, et `handoff.retire-timeout` signifie que le spinner de session n'a jamais été envoyé `retire`. Le gestionnaire de session (`argvus-greeter-handoff`) écrit `first layer-shell frame callback` et `stop requested` dans le journal.

Une entrée `project.theme-switch` ou `prepare.startup-theme` non nulle est attendue après un changement d'apparence, des fichiers de thème modifiés, un nouvel accent ou la migration d'un ancien marqueur de démarrage. La réinstallation de fichiers identiques préserve le cache de démarrage basé sur le contenu. S'il apparaît à chaque connexion, les fichiers de thème matérialisés sont manquants ; exécutez `argvus theme-switch <theme>` une fois.

Le journal de session est conservé sous `$XDG_STATE_HOME/argvus/session.log` (normalement `~/.local/state/argvus/session.log`). Après avoir modifié une substitution utilisateur Hyprland, déconnectez-vous et reconnectez-vous afin que le point d'entrée de la session le valide et le sélectionne à nouveau. Voir [Session graphique](/docs/argvus-session/graphical-session/) pour le flux de démarrage normal.
