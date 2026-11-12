---
title: Centre de contrôle
description: Configurez les parties persistantes de votre bureau ARGVUS.
---
Le Centre de Contrôle est l'application de paramètres axée sur le clavier d'ARGVUS. Utilisez-le lorsque vous souhaitez configurer le bureau plutôt que d'effectuer une action ponctuelle.

## Centre de Contrôle et Panneau de Configuration

| Utiliser… | Pour… |
| --- | --- |
| **Centre de Contrôle** | Thèmes, fonds d'écran, effets, disposition, comportement de la barre des tâches, polices, applications par défaut, paramètres clavier et saisie, langue, région et configuration du système. |
| **Panneau de Configuration** | Statut actuel et actions fréquentes telles que volume, luminosité, réseau, notifications, alimentation et contrôles de session. |

Le Panneau de Configuration peut exposer des contrôles rapides pour un paramètre configuré de manière permanente dans le Centre de Contrôle. Par exemple, utilisez le panneau pour une action rapide sur l'apparence ou l'affichage, et le Centre de Contrôle lorsque vous souhaitez modifier la configuration du bureau derrière cela. Voir Panneau de Configuration et [Où configurer les choses](/docs/user-guide/where-to-configure/).

## Zones principales

Le centre de contrôle actuel propose ces zones :

Le centre est un catalogue de destinations regroupées en six cartes. Chaque destination ouvre directement sa page, et la recherche la trouve sous n'importe quelle carte.
- **Apparence** — personnaliser le bureau : Style du bureau (thème, mode d'apparence, couleur d'accent, fonds d'écran, icônes et polices), Style de l'application (transparence du terminal et du lanceur), Barre des tâches, Panneau de contrôle et Télémétrie des widgets.
- **Bureau & Saisie** — configurer l'espace de travail : Hyprland (Windows : Règles, Espaces et Mode, animations, flou, bordures, ombre et version installée), Écrans, Clavier & Raccourcis et Souris & Pavé tactile.
- **Préférences** — paramètres personnels et régionaux : Langue & Région (la langue, le fuseau horaire, les formats et le hub de disposition du clavier), Date & Heure, Applications par défaut, Alimentation et Session.
- **Matériel & Connectivité** — appareils et connexions : Vue d'ensemble du matériel, Réseau & Wi-Fi, Bluetooth, Audio et Stockage.
- **Administration du système** — gérer le système d'exploitation : Packages, Services, Démarrage, Nom d'hôte, Utilisateurs et Groupes, Pare-feu et Notifications (Ne pas déranger).
- **Paramètres** — le Centre de contrôle lui-même, listé après les cartes : Langue (la langue de l'utilisateur) et Style (icônes décoratives et transparence). Appuyez sur `s` pour l'ouvrir.
- **Outils et diagnostics** — outils pour développeurs et pour la maintenance : ARGVUS Bienvenue, Projets, Extraits (texte sauvegardé pour `argvus snippets`, tapé avec `SUPER + ALT + N`), Clés SSH (paires de clés dans `~/.ssh` gérées par `argvus keys`), Clés GPG (le porte-clés géré par `argvus gpg` ; gpg-agent utilise pinentry-gtk), Services Dev, Docteur Système et À propos d'ARGVUS.

Lorsque Ne pas déranger est activé, les pop-ups de notification sont fermés et les nouvelles notifications sont conservées pour plus tard. Le Panneau de contrôle cache son historique de notifications jusqu'à ce que Ne pas déranger soit désactivé.

## ARGVUS Bienvenue

Choisissez **Open ARGVUS Welcome** sous **Outils et Diagnostics**, recherchez `welcome`, ou exécutez `argvus welcome`. Welcome est intégré dans le Centre de Contrôle et utilise sa fenêtre unique. Entrée ouvre la page de paramètres sélectionnée ; une fois terminé, revenez au guide avec Échap. F6 Suivant coche l’étape en cours et ouvre la suivante dans la même fenêtre. Échap ne fait que revenir en arrière, et Entrée sur la liste de contrôle ouvre toujours la page sélectionnée. O revoit à nouveau l’étape, S passe sans la cocher, et F termine le guide même si des étapes optionnelles sont en attente. Annuler ou revenir ne complète pas une étape. La progression existante est conservée.

Le guide couvre la langue et la région, les affichages, le matériel et les pilotes, le réseau, l'audio, le Bluetooth, l'apparence, l'alimentation, les applications par défaut et le démarrage de session. Il n’installe jamais de pilotes ni de packages et n’applique pas lui-même de modifications système. La progression est stockée dans `$XDG_STATE_HOME/argvus/welcome/progress.json` (ou `~/.local/state/argvus/welcome/progress.json`).

Une fois le bureau prêt, la session exécute `argvus welcome --if-needed` pour TTY, ARGVUS Greeter et autres gestionnaires de connexion. Il ouvre Bienvenue sauf si les enregistrements de progression par utilisateur `opened`, l’achèvement ou le rejet l'indiquent. Le premier rendu réussi enregistre `opened` dans `${XDG_STATE_HOME:-$HOME/.local/state}/argvus/welcome/progress.json`, empêchant sa réouverture lors des connexions ultérieures. Il ne dépend pas de `/etc/skel`. Les comptes existants sans cet enregistrement reçoivent également une ouverture automatique. Les échecs avant le rendu permettent une nouvelle tentative à la prochaine connexion. `argvus welcome` manuel reste disponible.

Les pages exactes des fournisseurs peuvent dépendre des packages installés et des autorisations. Ne supposez pas qu'un domaine optionnel manquant signifie un Centre de Contrôle cassé.

Lorsque la grille contient plus d'entrées que ce que l'écran peut afficher, une barre de défilement apparaît le long du bord droit pour indiquer qu'il y a plus en dessous.

## Trouver un paramètre

L'écran d'accueil enregistre les paramètres par titre, catégorie, identifiant et mots-clés. La recherche est insensible aux accents et accepte des termes pratiques tels que `theme`, `wallpaper`, `shortcut`, `mouse`, `monitor`, `network`, `user` et `DPI`. Les correspondances exactes du titre sont classées en premier, suivies d'un préfixe de titre, de tout mot du titre ou de la catégorie contenant le texte saisi, et enfin des correspondances de mots-clés — taper une partie d'un mot (pas seulement son début) peut donc toujours permettre de trouver la destination.

Le menu visible est conscient des capacités. Un itinéraire peut être absent lorsque son fournisseur n'est pas installé, lorsque le matériel n'est pas disponible ou lorsque la session en cours ne peut pas fournir la capacité requise. La recherche est donc également un moyen utile de découvrir ce que cette installation peut configurer.

L'arborescence principale des paramètres comprend actuellement :

| Domaine | Destinations actuelles |
| --- | --- |
| Apparence | Style du bureau : thème, mode d'apparence, couleur d'accentuation, fonds d'écran, icônes et polices ; Style de l'application : transparence du terminal et du lanceur ; Barre des tâches ; Panneau de configuration ; Télémétrie des widgets |
| Bureau & Entrée | Hyprland (Windows : Règles, Espaces et Mode, animations, flou, bordures et ombre), Affichages, Clavier & Raccourcis, Souris & Pavé tactile |
| Préférences | Langue & Région (langue, fuseau horaire, formats, locales système et disposition du clavier), Date & Heure, Applications par défaut, Alimentation, Session |
| Matériel et Connectivité | Aperçu du matériel (résumé, CPU, GPU, mémoire, périphériques), Réseau et Wi-Fi, Bluetooth, Audio, Stockage |
| Administration Système | Packages, Services, Démarrage, Nom de l'hôte, Utilisateurs et Groupes, Pare-feu, Notifications |
| Paramètres | Langue (langue de l'utilisateur), Style (icônes et transparence) |
| Outils et Diagnostics | Projets, Extraits, Clés SSH, Clés GPG, Services Dev, Docteur Système, À propos d'ARGVUS |

Certaines de ces sections sont des pages complètes et d'autres sont des sous-pages accessibles via la recherche ou un résumé de domaine. La liste exacte est déterminée par les fournisseurs ARGVUS installés.

Chaque page affiche son chemin dans l'en-tête, par exemple `Control Center > Hardware & Connectivity > Audio`. Le chemin commence par la carte Accueil qui liste la page, ainsi la même page se lit de la même manière depuis l'Accueil, depuis la recherche et depuis le CLI. La racine et les noms des cartes sont traduits : l'interface portugaise affiche `Central de Controle > Hardware e conectividade > Áudio`.

## À quoi sert chaque zone

### Apparence

L'apparence est le principal domaine de personnalisation. Son groupe Bureau contient le thème (avec son mode Collant ou Espacé), le mode d'apparence, la couleur d'accent, les fonds d'écran, le thème d'icônes et la page Polices, et son groupe Style d'application contient la transparence du terminal et du lanceur. La barre des tâches, le Panneau de configuration et la télémétrie des widgets sont listés directement sous Apparence sur la page d'accueil et réutilisent la page de transparence partagée d'Apparence pour leurs propres surfaces. Le flou est un paramètre global, sous Hyprland > Flou, et il s'applique à chaque surface ARGVUS qui a le flou activé. Cette page modifie son interrupteur Activer et les valeurs de flou Hyprland (Taille, Passages, Luminosité, Bruit, Contraste, Vivacité et Obscurité de la vivacité) comme brouillon, et rien ne change tant que **Appliquer** n'est pas sélectionné. Les animations fonctionnent de la même manière sous Hyprland > Animations : son interrupteur Activer, les courbes personnalisées (Bézier, Ressort, Linéaire et Ease-out) et les cinq catégories d'animation forment un brouillon qui est appliqué avec **Appliquer**. Bézier et Spring ont des paramètres modifiables (les points de Bézier et la masse, la rigidité et l'amortissement du ressort) ; Linear et Ease-out sont des courbes fixes, donc leur page ne liste que leurs points de contrôle et ne peut pas être modifiée. Les choix de style suivent ce que Hyprland 0.56.2 dessine : Opening et Closing offrent slide, popin et gnomed, layer entrance et exit offrent slide, popin et fade, et les leaves de l'espace de travail offrent slide, fade, slidefade, slidevert et slidefadevert. Ces contrôles se coordonnent avec les composants d'apparence et de session au lieu de changer uniquement la fenêtre du Centre de Contrôle.

### Barre des tâches

La barre des tâches configure la position du panneau, l'espacement autour de celui-ci et de ses fenêtres, sa hauteur et sa bordure, le groupe d'icônes utilitaires et les blocs date/heure, ainsi que sa propre transparence. Elle est répertoriée sous **Apparence** sur l'accueil. En mode Sticky, **Espaces > Bas** par défaut sur `2` ; Spaced conserve sa ligne de base de `18` pixels.

**Hauteur** choisit 24, 28 (par défaut), 32 ou 36 pixels. **Bordure** dispose d'un interrupteur **Activer** et d'une **Épaisseur** de 1 à 3 pixels, avec 1 comme valeur par défaut. Lorsque la bordure est désactivée, la fenêtre de la barre des tâches, le groupe de droite et le lecteur MPRIS n'ont pas de bordure. Ces modifications sont provisoires jusqu'à ce que **Appliquer** soit sélectionné. La hauteur et la bordure appliquées sont conservées lorsque le thème change ; tant qu'elles ne sont pas appliquées, c'est le thème qui les détermine.

La page **Icônes** ouvre **Modules optionnels**, **Présentation de l'espace de travail**, **Lanceur** et **Utilitaires**. Les modules optionnels peuvent être affichés ou masqués individuellement : lecteur audio, Focus, lanceur, réseau, profil d'alimentation, disposition du clavier, mémoire, CPU et relevés de température, date, horloge, batterie, barre d'état système, notifications, périphériques amovibles, enregistrement, Bluetooth, entrée/sortie audio et maintien de l'activité. Les espaces de travail restent disponibles et peuvent afficher leurs noms (par défaut) ou les icônes des applications. Le titre de la fenêtre focalisée, Paramètres et Actions d'alimentation sont toujours disponibles. La visibilité du lanceur est contrôlée dans Modules optionnels ; son sous-menu modifie l'icône personnalisée. Les modifications prennent effet après avoir cliqué sur **Appliquer**, ce qui redémarre la barre des tâches.

### Panneau de configuration

Le panneau de configuration configure la visibilité et l'ordre de ses cartes d'action rapide, ainsi que ses propres sessions et transparence.

La page s'ouvre et accepte les entrées dès que les paramètres de la carte sont lus. Les cartes qui dépendent du matériel (Bluetooth et Luminosité) sont vérifiées en arrière-plan : jusqu'à la fin de cette vérification, elles conservent la disponibilité vue pour la dernière fois dans cette fenêtre, et la liste se met à jour automatiquement lorsque le résultat arrive.

### Télémétrie des widgets

La télémétrie des widgets configure les sessions et la transparence du widget d'informations système.

### Polices

Les polices sont accessibles depuis **Apparence > Style du bureau > Polices**. Elles permettent de sélectionner une famille et une taille pour la barre des tâches, la télémétrie/informations système, le Panneau de configuration, l'interface système ARGVUS, les applications, le terminal et le navigateur. Elles exposent également les paramètres de rendu des polices tels que l'anticrénelage, l'infobulle des glyphes, le mode sous-pixel et le DPI. La famille principale par défaut est IBM Plex Mono ; les cibles individuelles peuvent être modifiées indépendamment et restaurées depuis la page Polices.

### Applications par défaut

Les applications par défaut sélectionnent les programmes installés pour des rôles tels que le terminal, le gestionnaire de fichiers, l'éditeur de texte, l'éditeur de terminal, le navigateur, le visualiseur d'images, le visualiseur de PDF, le lecteur vidéo, le lecteur audio, l'outil d'archives et le lanceur. La sélection est enregistrée par ARGVUS et peut mettre à jour les associations standard XDG. Les gestionnaires de fichiers TUI utilisent l'éditeur de terminal sélectionné lors de l'ouverture de fichiers texte ; les éditeurs de texte graphiques restent les applications par défaut pour les associations de fichiers sur le bureau.

### Extraits

Extraits est la page pour le texte que `argvus snippets` tape avec `SUPER + ALT + N`. Elle se trouve dans **Outils & Diagnostics → Extraits** (recherchez `snippets` ou exécutez `argvus control-center snippets`).
- **Nouvel extrait** — saisissez un **Nom** et un **Contenu**, puis **Enregistrer l'extrait**. Un nom déjà utilisé remplace le contenu de cet extrait.
- **Extraits enregistrés** — listés avec leur position (`1  email`) et le contenu sur une seule ligne. Appuyez sur Entrée sur le contenu pour le modifier. **Type** indique le type de l'extrait, et **Supprimer** demande confirmation.
- **Ouvrir le sélecteur** — ouvre le même sélecteur que `SUPER + ALT + N`.

La saisie commence 3 secondes après avoir appuyé sur Entrée, car le Centre de Contrôle a lui-même le focus à ce moment-là. Cliquez sur le champ qui doit recevoir le texte pendant ce temps. La page ne modifie que du contenu sur une seule ligne ; un extrait multi-ligne peut être créé dans un terminal avec `argvus snippets add`.

La page lit les entrées depuis `argvus-config` et les écrit via `argvus snippets`, donc le lanceur conserve la validation. Lorsque `argvus snippets` n'est pas installé, la page l'indique.

### Clés SSH

Les clés SSH sont la page pour les paires de clés que `argvus keys` gère. Elle se trouve dans **Outils et Diagnostics → Clés SSH** (recherchez `ssh keys` ou exécutez `argvus control-center ssh-keys`).

La page comporte quatre parties, de haut en bas :

```
Search        filters the key pairs by name or comment
Generate      name, type and the Generate key pair action
Keys          one block per key pair; a scrollbar appears when they do not fit
Actions       Refresh and Unload all keys from agent
```
#### Comment l'utiliser

1. Ouvrez **Centre de contrôle → Outils & Diagnostics → Clés SSH**. Depuis l'accueil, vous pouvez également rechercher `ssh keys`.
2. **Trouver une clé :** appuyez sur `/`, puis tapez pour n'afficher que les paires dont le nom ou le commentaire correspond. `Enter` termine la recherche et exécute la ligne surlignée, comme dans les autres listes, donc vérifiez la ligne avant d'appuyer dessus ; `Esc` efface la recherche.
3. **Créer une paire :** dans **Générer**, sélectionnez **Nom**, appuyez sur `Enter`, tapez le nom et appuyez à nouveau sur `Enter`. Sélectionnez **Type** et appuyez sur `Enter` pour basculer entre `ed25519` et `rsa`. Sélectionnez **Générer une paire de clés** et appuyez sur `Enter`. La nouvelle paire apparaît immédiatement sous **Clés**, sans phrase de passe.
4. **Copier la clé publique :** dans le bloc de la paire, sélectionnez **Copier la clé publique** et appuyez sur `Enter`. Le contenu du fichier `.pub` est copié dans le presse-papiers de Wayland, prêt à être collé dans un `authorized_keys` du serveur ou d'un hébergeur Git. La clé privée n'est jamais copiée. Cela nécessite `wl-clipboard` (`wl-copy`) ; sans cela, la page indique que la clé n'a pas été copiée.
5. **Charger une paire dans l'agent :** dans le bloc de la paire, sélectionnez **Charger dans l'agent** et appuyez sur `Enter`. Une clé sans phrase de passe est chargée immédiatement, sans fenêtre. Une clé avec phrase de passe ouvre un terminal ARGVUS, où OpenSSH la demande ; fermez le terminal avec `Enter` lorsque le message indique que la clé est chargée. La page vérifie l'agent toutes les quelques secondes pendant qu'elle est ouverte, donc la ligne passe automatiquement à **Décharger de l'agent**. Pour mettre à jour immédiatement, utilisez **Actualiser** dans **Actions**.
6. **Supprimer une clé de l'agent :** dans le bloc de la paire, sélectionnez **Décharger de l'agent**. Pour supprimer chaque clé, utilisez **Décharger toutes les clés de l'agent** dans **Actions**. Les fichiers clés restent sur le disque.
7. **Supprimer une paire :** dans le bloc de la paire, sélectionnez **Supprimer** et appuyez sur `Enter`. La confirmation se concentre sur **Annuler**, donc un `Enter` parasite ne fait rien. Appuyez sur `y` pour confirmer ou sur `n`/`Esc` pour annuler. Une clé chargée dans l'agent ne peut pas être supprimée ; déchargez-le d'abord.

Se déplacer : `↑`/`↓` ou `j`/`k` se déplace dans la liste, `Enter` active une ligne, `Esc` revient en arrière et `?` affiche l'aide. `Tab` déplace le focus vers **Actions**, où `←`/`→` sélectionne un bouton et `Enter` l'exécute. `↑`/`↓` ramène le focus sur la liste.

Les phrases secrètes ne sont jamais saisies dans le Control Center et ne sont jamais transmises comme arguments de commande. La page lit la liste avec `argvus keys list` et écrit uniquement via `argvus keys`. Lorsque `argvus-keys` n'est pas installé, la page l'indique.

#### Dépannage

| Symptôme | Cause et solution |
| --- | --- |
| `Could not open a connection to your authentication agent` après **Charger dans l'agent** | Le terminal n'a pas reçu la socket de l'agent. Hyprland conserve l'environnement avec lequel il a démarré, et l'agent peut avoir démarré plus tard. Mettez à jour `argvus-launcher`, qui revient à `$XDG_RUNTIME_DIR/ssh-agent.socket`, et vérifiez `systemctl --user status argvus-ssh-agent.service`. |
| **Agent** affiche `Agent unavailable` | Le lanceur ne peut pas atteindre l'agent. Vérifiez le service comme ci-dessus. |
| La page indique `argvus-keys is not installed` | Mettez à jour le paquet `argvus` : `argvus keys` est une sous-commande de celui-ci. |
| **Supprimer** affiche une erreur | La clé est chargée dans l'agent, et elle est conservée. Déchargez-la d'abord, puis supprimez-la à nouveau. |

### Clés GPG

Les clés GPG sont la page pour le trousseau de clés que `argvus gpg` gère. Elle se trouve dans **Outils et diagnostics → Clés GPG** (recherchez `gpg keys` ou exécutez `argvus control-center gpg-keys`).

La page comporte ces sections, de haut en bas :

```
Pinentry         the pinentry gpg-agent uses, for information only: pinentry-gtk
Generate key ▾   full name, e-mail, comment, algorithm, key strength, expiration date,
                 passphrase, and the Generate key and Cancel actions (expanded at first)
Import ▾         Import public key… and Import private key… (expanded at first)
Keys ›           the keyring, on its own page, with the search at the top
```
`▾` marque une section étendue et `▸` une section repliée. `Enter` ou `Space` l'étend ou la replie.

#### Comment l'utiliser

1. Ouvrez **Centre de contrôle → Outils & Diagnostics → Clés GPG**. Depuis l'accueil, vous pouvez également rechercher `gpg keys`.
2. **Pinentry** est fixé à `pinentry-gtk`. La page ne l'affiche que. La première fois qu'une clé est générée, ARGVUS écrit ce pinentry dans `gpg-agent.conf`, entre ses propres lignes de marqueur, et conserve vos autres lignes. Si vous avez déjà écrit une ligne `pinentry-program` en dehors de ces marqueurs, la génération s'arrête avec une erreur afin que vous puissiez supprimer cette ligne.
3. **Créer une clé :** dans **Générer la clé ▾**, saisissez **Nom complet** et **E-mail**. Les deux sont obligatoires, et **Générer la clé** reste désactivé jusqu'à ce que les deux champs contiennent du texte. **Commentaire** est facultatif. L'identifiant utilisateur devient `Name (comment) <e-mail>`.
- **Algorithme** bascule entre **RSA** et **DSA ElGamal**. Une clé DSA obtient une sous-clé de chiffrement ElGamal ; une clé RSA obtient une sous-clé RSA.
- **Force de la clé** cyclique entre les tailles de l'algorithme : RSA 1024, 2048 et 4096 ; DSA 1024, 2048 et 3072. gpg ne crée pas de clés DSA supérieures à 3072 bits, donc 4096 n'est pas proposé pour DSA.
- **Date d'expiration** ouvre sa propre page. Tapez la date dans la disposition de votre langue : `DD/MM/YYYY` en portugais, espagnol, français, italien, allemand et russe ; `YYYY/MM/DD` en anglais, chinois, japonais et coréen. La date doit être postérieure à aujourd'hui. Activez **N'expire jamais** pour rendre la clé permanente ; le champ de date est alors désactivé et non utilisé.
- **Phrase de passe** ouvre sa propre page. **Protéger avec une phrase de passe** est activé par défaut ; tapez la **Phrase de passe** puis **Confirmer la phrase de passe**. Les deux doivent correspondre. Désactiver la protection efface les deux champs et la page demande une confirmation avant de générer une clé non protégée.
4. Sélectionnez **Générer la clé** et appuyez sur `Enter`. La clé obtient une clé de signature et une sous-clé de chiffrement. La phrase de passe est envoyée au lanceur sur son entrée standard, jamais comme argument de commande. Si la génération échoue, les autres champs restent, mais les phrases de passe sont effacées et doivent être saisies à nouveau.
5. **Annuler** efface tous les champs du formulaire et ramène les dates et les tailles à leurs valeurs par défaut.
6. **Importer :** dans **Importer ▾**, choisissez **Importer une clé publique…** ou **Importer une clé privée…**. Le gestionnaire de fichiers s'ouvre pour que vous puissiez choisir le fichier. Une clé privée demande d'abord une confirmation, et sa phrase de passe est demandée par gpg lorsque la clé est utilisée. Si le gestionnaire de fichiers est graphique, il ne peut pas signaler le fichier à la page ; la page ouvre alors un champ où vous tapez le chemin.
7. **Clés ›** ouvre le trousseau de clés. Chaque clé montre son identité, son empreinte, son type, sa date d'expiration et si sa clé privée est dans le trousseau. Chaque clé dispose de ces actions :
   - **Modifier la clé ›** ouvre la page propre à la clé (décrite ci-dessous).
- **Exporter la clé publique…** demande un chemin de fichier, proposé comme `~/<fingerprint>.asc`. Le fichier est créé avec le mode `600`. Un fichier existant n’est jamais remplacé.
- **Exporter la clé privée…** apparaît uniquement pour les clés ayant une partie privée. Elle demande une confirmation, puis un chemin de fichier, proposé comme `~/<fingerprint>-private.asc`. Le fichier est créé avec le mode `600`.
- **Copier la clé publique** et **Copier l’empreinte** copient dans le presse-papier de Wayland avec `wl-copy`.
- **Supprimer la clé…** demande une confirmation, avec le focus sur Annuler. Elle supprime la clé et sa partie privée.
8. **Trouver une clé :** sur la page **Clés**, la barre de recherche en haut filtre les clés. Tapez pour réduire la liste ; la recherche correspond à l’identité (qui inclut le commentaire) et à l’empreinte. `Esc` efface la recherche.
9. **Modifier la clé ›** affiche les détails de la clé, puis ces actions :
   - **Date d'expiration ›** modifie la date d'expiration de la clé et de chacune de ses sous-clés. Utilisez **N'expire jamais** pour supprimer la date d'expiration. **Appliquer** enregistre la modification.
- **Modifier la phrase de passe** demande l'ancienne et la nouvelle phrase de passe dans la fenêtre pinentry.
- **Signer la clé avec ›** certifie cette clé avec l'une de vos autres clés qui possède une partie privée. La phrase de passe du signataire est demandée. La page demande d'abord une confirmation.
- **Révoquer la clé ›** liste les raisons : aucune raison, clé compromise, clé remplacée, clé plus utilisée. Chacune demande une confirmation. Une clé révoquée ne peut plus être utilisée.
- **Confiance** définit le niveau de confiance envers la clé : non définie, marginale, complète ou ultime.
- **Publier sur le serveur de clés…** envoie la clé publique au serveur de clés défini dans `gpg.conf`, après confirmation. **Récupérer depuis le serveur de clés** télécharge la clé depuis ce serveur.

Se déplacer : `↑`/`↓` ou `j`/`k` se déplacent dans la liste, `Enter` active une ligne, `Esc` revient en arrière et `?` affiche l'aide.

Les phrases de passe ne sont saisies que dans les champs masqués de la page de phrase de passe ou dans la fenêtre pinentry. La page envoie la phrase de passe de génération à `argvus gpg` sur l'entrée standard, et le lanceur la transmet à gpg sur un descripteur de fichier, elle n’apparaît donc jamais dans la liste des processus, dans un argument de commande ou dans les journaux. Les modifications de clés s’exécutent en arrière-plan ; pendant qu’une est en cours, les autres attendent. La page lit le trousseau avec `argvus gpg list` et écrit uniquement via `argvus gpg`. Lorsque `argvus gpg` n’est pas installé, la page l’indique.

Pas encore disponible sur la page : rechercher une clé sur le serveur de clés par nom ou par e-mail. Utilisez **Récupérer depuis le serveur de clés** avec une empreinte connue.

#### Dépannage

| Symptom | Cause et solution |
| --- | --- |
| La page indique `argvus gpg is not available` | Le `argvus-launcher` installé est antérieur à la commande GPG. Exécutez `argvus gpg list` dans un terminal : `unknown command` signifie que le paquet doit être reconstruit et installé. Les clés ont également besoin de `gnupg`. |
| **Générer la clé** reste désactivé | Saisissez à la fois le **Nom complet** et l'**E-mail**. |
| Une ligne d'état indique `argvus-gpg: pinentry-gtk is not installed` | Installez le paquet `pinentry`, qui fournit `pinentry-gtk`. |
| La génération s'arrête avec une erreur `pinentry-program` | `gpg-agent.conf` définit un pinentry en dehors du bloc ARGVUS. Supprimez cette ligne ; ARGVUS écrit alors le sien. |
| **Pinentry** affiche `Not set` | gpg-agent utilise son défaut intégré jusqu'à ce que la première clé soit générée depuis la page. |
| La page indique que le `argvus-launcher` installé est plus ancien que le Centre de Contrôle | Réinstallez `argvus-launcher` : exécutez `make build` puis `make install` dans son dépôt, et rouvrez la page. Les actions clés restent désactivées jusqu'à ce que les versions correspondent. |
| Le gestionnaire de fichiers s'ouvre, mais la page demande un chemin | Le gestionnaire de fichiers est graphique et ne peut pas signaler le fichier choisi. Tapez le chemin dans le champ. |
| **Copier la clé publique** indique qu'elle n'a pas pu être copiée | `wl-copy` n'est pas installé ou la session Wayland n'est pas disponible. Installez `wl-clipboard`. |
| **Modifier la clé ›** indique que la clé n'est plus dans le trousseau | Le trousseau a changé depuis l'ouverture de la page. Revenez à **Clés ›** et actualisez. |
| Une ligne d'état affiche `argvus-gpg: ...` | Le lanceur a refusé l'action, et le texte en donne la raison, comme une date d'expiration invalide, un fichier déjà existant, ou un `--yes` manquant. |
| Une action attend et rien ne se passe | La fenêtre de saisie du code PIN est masquée ou n'a pas été ouverte. Vérifiez si la session expose `WAYLAND_DISPLAY` ou `DISPLAY`. |
| L'importation est refusée | Le fichier ne contient aucune clé du type choisi. Les clés publiques passent par **Importer une clé publique…** ; les clés privées par **Importer une clé privée…**. |

### Langue et région

Cette carte répertorie la langue (la locale régionale), le fuseau horaire, la date et l'heure, les locales du système et la disposition du clavier, et chacune s'ouvre directement depuis l'accueil. Les pages du clavier contiennent également la variante et la table de correspondance des touches de la console. La recherche depuis l'accueil expose également les raccourcis clavier et les paramètres de la souris/du pavé tactile comme des routes de paramètres directes.

La liste des langues découvre les catalogues installés et affiche les noms natifs pour les dix langues. Activez la locale système UTF-8 correspondante avant de la sélectionner.

À la fois **Préférences > Langue et région > Langue** et **Paramètres > Langue** enregistrent la même préférence utilisateur : `LANG` dans `$XDG_CONFIG_HOME/locale.conf` (par défaut `~/.config/locale.conf`), par exemple `LANG=en_US.UTF-8`. Les commentaires et les variables régionales sont préservés. Aucun fichier global ou paramètre linguistique ARGVUS parallèle n'est écrit. Un utilisateur non configuré obtient l'anglais ; les fichiers `session.language` et `data/language` hérités sont ignorés. Déconnectez-vous et reconnectez-vous pour appliquer la langue à la session complète et aux applications.

**Paramètres régionaux du système** modifie `/etc/locale.gen` et exécute `locale-gen`, donc son application nécessite une autorisation administrateur. Un avis persistant **[WARN]** au-dessus du pied de page identifie les pages contenant des paramètres administratifs. Le même avis couvre le fuseau horaire, la date/heure, le nom d'hôte, le mappage clavier de la console, l'édition des comptes et des groupes, le pare-feu, les services système, le régulateur du processeur, les profils et politiques d'alimentation, la configuration de démarrage, les opérations sur les paquets, les instantanés et les modifications réseau protégées. Les préférences personnelles, les services utilisateur et les pages en lecture seule ne reçoivent pas cet avis. L'avis est informatif ; l'autorisation n'est demandée que lorsqu'une action en a besoin.

### Matériel et affichages

Lorsque les capacités correspondantes sont installées, le Centre de Contrôle peut afficher des informations matérielles telles que le CPU, le GPU, la mémoire, l'alimentation et les périphériques. La page d'entrée configure le comportement de la souris et du pavé tactile ; la zone d'affichage gère la résolution du moniteur, le taux de rafraîchissement, l'échelle, la position, l'orientation, l'affichage principal, le VRR et le HDR lorsque la session en cours les prend en charge. Les modifications d'affichage s'appliquent immédiatement. Après un changement pouvant rendre l'écran inutilisable (résolution ou taux de rafraîchissement, position, mise en miroir, désactivation d'un moniteur, profondeur de couleur) ou après **Appliquer**, le Centre de Contrôle demande s'il faut le conserver, avec le focus sur **Revenir** : appuyez sur `y` pour le conserver ; `n`, `Esc` ou `Enter` pour revenir en arrière, et sans réponse dans les 15 secondes, la configuration précédente revient automatiquement. La suppression de la configuration enregistrée d'un moniteur déconnecté et la suppression d'un profil demandent une confirmation ; les profils ouvrent leur propre page avec **Appliquer le profil**, **Renommer** et **Supprimer**. profil**. La disposition est sauvegardée via `argvus config monitor-set`, `monitor-primary`, `monitor-workspaces` et `monitor-remove`, donc la section canonique `/displays` de `config.json` est la seule source de la disposition du moniteur. Seul le catalogue de profils reste dans `display/config.toml`.

### Connectivité et audio

La zone réseau expose le statut et les pages de connexion du fournisseur de réseau installé, y compris le Wi-Fi, Ethernet, VPN, DNS et les routes proxy lorsque disponibles. Le pare-feu est listé sous **Administration du système**. Le Bluetooth est conditionnel au support du Bluetooth. La zone audio est fournie lorsque sa capacité optionnelle est activée et s'intègre avec le service audio installé.

La page d'accueil **Réseau et Wi-Fi** et sa page **État** se remplissent au fur et à mesure que chaque requête de NetworkManager répond, au lieu d'attendre la plus lente : d'abord la liste des périphériques, ensuite la radio, les adresses et DNS des interfaces connectées, les connexions enregistrées et les réseaux Wi-Fi déjà connus de NetworkManager (une nouvelle analyse ne s'exécute que depuis la page Wi-Fi avec `r`). Jusqu'à l'arrivée de la liste des périphériques, les lignes indiquent *Chargement...*, et l'interrupteur Wi-Fi n'est proposé que lorsque l'ordinateur possède une carte Wi-Fi. Le lancement d'une action (l'interrupteur Wi-Fi, la connexion, le DNS) pendant qu'une page se rafraîchit remplace le rafraîchissement, et la page se rafraîchit à nouveau à la fin de l'action.

#### Connexions, inspiré par nmtui

**Réseau & Wi-Fi > Connexions** liste tous les profils de connexion NetworkManager, regroupés par type, avec l'appareil sur lequel ils sont actifs (`★ Active · eno1`), s'il s'agit d'un port d'un pont, d'un bond ou d'une équipe, et s'ils sont en lecture seule. `Enter` ouvre le profil dans l'éditeur, `a` le connecte ou le déconnecte, `/` filtre par nom et `r` actualise. **Activer une connexion** est la même liste où `Enter` met un profil en marche ou l'arrête, comme dans `nmtui`. Les pages de détail d'une interface, d'un réseau Wi-Fi enregistré et d'un VPN ont une ligne **Modifier la connexion** qui ouvre leur profil.

**Ajouter une connexion** crée des profils Ethernet, Wi-Fi, Point d'accès, DSL (PPPoE), Pont, Bond, Team, VLAN, tunnel IP, Loopback et WireGuard, ainsi qu’un profil VPN par plugin VPN NetworkManager installé (OpenVPN a son propre formulaire ; les autres plugins modifient leurs paramètres en tant qu’entrées `key=value`). Il importe également un fichier WireGuard `.conf` ou un fichier de plugin tel qu’un `.ovpn` (`nmcli connection import`). Team nécessite le paquet `libteam`, DSL nécessite `ppp` et un VPN nécessite son plugin (par exemple `networkmanager-openvpn`) ; sans eux, la ligne est désactivée et indique ce qu’il faut installer. WEP, InfiniBand, MACsec et Open vSwitch ne sont pas proposés.

L'éditeur est une seule liste de sections pliables : **Général** (nom, interface, connexion automatique, priorité, disponible pour tous les utilisateurs, mesuré), les paramètres propres au type, **IPv4** et **IPv6** (méthode, adresses, passerelle, serveurs DNS, domaines de recherche, routes, ignorer les DNS ou routes automatiques, ne jamais utiliser comme route par défaut, métrique de route, extensions de confidentialité, génération d'adresse). Les choix et les nombres changent avec `←/→` ; `Enter` saisit une valeur ; les listes (adresses, DNS, routes, IP autorisées) ouvrent leur propre page, où `Enter` modifie une entrée, la supprimer la supprime et **Ajouter une entrée** en ajoute une. Les champs qui ne s'appliquent pas à la méthode choisie sont affichés désactivés. Un pont, un bond ou une équipe choisit ses **Ports** Ethernet sur une sous-page ; WireGuard modifie ses **Pairs** sur une sous-page et peut **Générer une paire de clés** (avec `wg`, depuis `wireguard-tools`, une dépendance du paquet `argvus`) ; la clé publique de la clé privée est affichée en dessous.

Rien n'atteint NetworkManager tant que vous n'avez pas cliqué sur **Enregistrer**, qui reste désactivé tant qu'aucune modification n'a été effectuée ou tant qu'un champ est invalide, et indique alors quel champ et pourquoi. Seules les modifications apportées sont enregistrées. Enregistrer une connexion active demande d'abord confirmation puis la reconnecte, afin que les nouveaux paramètres s'appliquent immédiatement (cela peut interrompre brièvement le réseau). `Esc` avec des modifications non enregistrées demande avant de les ignorer. **Supprimer la connexion**, dans la zone de danger, demande une confirmation avec le focus sur Annuler ; la suppression d'un pont, d'un bond ou d'une équipe supprime également ses ports.

Les mots de passe, clés privées et clés prépartagées ne sont jamais relus : leurs lignes indiquent *Non modifié* jusqu'à ce qu'une nouvelle valeur soit saisie, et la nouvelle valeur est affichée masquée. Ils sont écrits via l'entrée standard de `nmcli connection edit`, jamais comme arguments de commande. Les modifications s'exécutent sous l'utilisateur de la session : les membres locaux de `wheel` sont autorisés par la règle polkit de NetworkManager, et les autres utilisateurs doivent fournir le mot de passe administrateur via l'agent polkit de la session.

### Puissance et session

L’alimentation fournit les pages de contrôle de l’alimentation du système et de la politique disponibles pour l’installation, y compris l’interrupteur Garder éveillé. Lorsqu’il est activé, Garder éveillé empêche la politique d’inactivité ARGVUS de déclencher automatiquement le verrouillage de l’écran et les minuteries d’alimentation de l’affichage. Session expose le statut de la session ARGVUS, ses composants, son démarrage automatique, ses diagnostics et ses journaux. Dans **Alimentation**, la barre **Actions** sous la liste contient **Suspendre**, **Hiberner** et **Actualiser** (`r` actualise également) : `Tab` ou `←/→` s’y déplacent, `Enter` exécute le bouton sélectionné, `↑`/`↓` ramènent le focus sur la liste, et `Esc` revient en arrière. Suspendre et Hiberner demandent d’abord une confirmation. Ces pages peuvent nécessiter des permissions système et doivent être distinguées des actions rapides dans le Panneau de configuration.

### Outils système

La section du système peut exposer des informations de démarrage, des packages, des services, le stockage, des diagnostics, l'administration des utilisateurs/groupes et des informations système. Ce sont des outils administratifs ou de diagnostic ; une page peut être en lecture seule ou nécessiter une autorisation selon l'opération.

**Services** a quatre pages : unités système, unités utilisateur, unités échouées et le journal (Logs). Chaque liste d'unités montre une ligne `Filter:`, que `f` fait défiler, et une boîte `Search:` encadrée, ouverte avec `/`. Chaque unité montre son état Actif et Activé sous forme de points remplis ou vides. Dans Logs, `b` bascule entre le démarrage actuel et le précédent, `u` fait défiler l'unité et `p` fait défiler la priorité ; la ligne `Filter:` montre les valeurs utilisées. `Enter` ouvre une unité ou une entrée du journal.

**Services Dev** rapporte trois sources orientées développement sur une page en lecture seule : les services systemd de l'utilisateur actuel, les ports TCP actuellement à l'écoute (`ss`) et les conteneurs en cours d'exécution de tout runtime de conteneur installé (Podman, Docker). Elle n'apparaît sur la grille d'accueil que si au moins l'une de ces sources est disponible sur le système — une machine sans session utilisateur systemd, sans Podman et sans Docker n'affiche pas l'entrée. `r` actualise les trois sources indépendamment, de sorte qu'une source lente ou défaillante (par exemple un runtime de conteneur qui a cessé de répondre) ne bloque pas les autres pour afficher leurs données.

**Les packages** sont regroupés par source. **Dépôts officiels** et, lorsqu'un assistant AUR (`paru` ou `yay`) est installé, **AUR (Arch User Repository)** ouvrent chacun **Installer**, **Supprimer** et **Mettre à jour** ; **Cache**, **Historique**, **Rétrograder** et **Mirroirs** suivent. Les trois tâches partagent un même écran : la **barre de recherche** fixée en haut prend les lettres que vous tapez, et les packages sont listés en dessous dans le bloc **Résultats** (**Mises à jour** lors de la mise à jour), où une liste vide indique pourquoi (par exemple **Aucun résultat**, **Aucun élément correspondant** ou **Aucune mise à jour en attente**). **Installer** recherche dans les dépôts (`pacman -Ss`) ou dans l'AUR lorsque vous appuyez sur `Enter` dans la barre de recherche en haut ; **Supprimer** liste les packages installés (la liste AUR montre les packages construits en dehors des dépôts, `pacman -Qm`, et tant que le menu AUR est affiché, ils quittent la liste officielle) ; **Mettre à jour** liste les mises à jour en attente (`pacman -Qu`, ou `paru -Qua`/`yay -Qua` pour l'AUR). Sur un package, `Enter` exécute la tâche de l'écran et `→` ouvre la page du package, avec **Install** (ou **Réinstaller**) et, dans sa **Zone de danger**, **Supprimer** ; `←` ou `Esc` revient en arrière. **Dépôts officiels → Supprimer** commence par **Orphelins**, où `Space` marque les packages (`[x]`), `Enter` en ouvre un et **Supprimer marqué** les supprime ensemble. Dans la liste des résultats de la mise à jour, `Tab` se déplace vers la barre **Actions** sous la liste (**Tout mettre à jour** et **Actualiser la base de données**, ou **Tout mettre à niveau (AUR)** sur l'AUR), `←`/`→` se déplacent entre ses boutons, `Enter` exécute celui qui a le focus et `↑`/`↓` (ou en tapant) ramène le focus sur la liste ; **Tout mettre à jour** est grisé et ignoré alors qu'aucune mise à jour n'est répertoriée. Le fil d'Ariane et la recherche globale suivent le même chemin, par exemple **Packages > Official Repositories > Remove > Orphans**. Les nettoyages du cache se font dans la zone Danger de la page Cache et **Miroirs → Configurer les miroirs** définit les options du réflecteur avant **Générer un aperçu**. Chaque opération de paquet affiche son plan, ou pour l'AUR l'avertissement PKGBUILD, et demande une confirmation (`y` confirme, `n` ou `Esc` annule) avant de s'exécuter, et sa sortie est affichée pendant son exécution. Les opérations AUR exécutent l'assistant en tant que votre utilisateur (`-S <package>` pour installer ou mettre à jour un paquet, `-Sua` pour mettre à jour tous les paquets) ; les suppressions AUR utilisent le même `pacman -Rns` que les paquets officiels. Lorsque d'autres paquets installés dépendent du paquet que vous supprimez (officiel ou AUR), la confirmation les nomme et offre deux options : **Supprimer uniquement** le paquet sélectionné (`pacman -Rdd`, ce qui ignore les vérifications de dépendances, donc les paquets qui en ont besoin peuvent cesser de fonctionner) ou **Supprimer avec les dépendants** (`pacman -Rcns`, avec le nombre de paquets supprimés). Le focus commence sur **Annuler**, qui conserve tous les paquets ; `Enter` exécute la ligne sélectionnée et il n'existe pas de raccourci `y` dans ce choix.

La barre **Recherche** supérieure contient la requête du package. `/` le concentre sans effacer son texte ; la saisie déplace également le focus là-bas. Modifiez avec les touches fléchées, `Home`, `End`, `Backspace`, `Delete` ou collez. Dans **Install**, `Enter` soumet une requête de référentiel/AUR même lorsque des résultats précédents existent, puis déplace le focus sur les résultats ; les recherches officielles nécessitent une requête non vide et les recherches AUR nécessitent au moins deux caractères. La modification d'une requête d'installation annule sa demande en attente et efface ses anciens résultats ; une nouvelle demande attend `Enter`. Dans **Remove**, **Update** et **Orphans**, la modification filtre immédiatement la liste chargée sans autre requête du référentiel. `↓` passe de la recherche aux résultats, `↑` sur le premier résultat revient à la recherche et `Esc` dans la recherche revient à la liste tout en préservant la requête. `Enter` sur un résultat exécute l'action de ce package. La barre d'action Mettre à jour reste accessible par `Tab`, et `/` en revient à la recherche.

Toutes les opérations dans **Packages** ouvrent un terminal interactif intégré : packages officiels, AUR, nettoyage du cache, miroirs et modifications du référentiel/clé. Répondez aux invites du programme en tapant et en appuyant sur `Enter` ; pacman fonctionne sans `--noconfirm` dans ce terminal. `Esc` masque la vue et maintient le processus en cours d'exécution. `F8` rouvre la même session à partir de n'importe quelle page Packages ; **Afficher le terminal** est également disponible dans ses menus. Le pied de page indique si la transaction est en cours, terminée ou échouée. Après une commande réussie, `Done!` apparaît à l'intérieur du terminal après sa sortie finale, y compris lorsque le terminal était masqué. `Shift+PageUp` / `Shift+PageDown` fait défiler l'historique, `Shift+End` revient à la sortie en direct et `Ctrl+C` envoie une interruption au processus enfant. Lorsque le terminal est activé, `q` et `?` sont des entrées plutôt que des raccourcis globaux. La sortie, les couleurs ANSI et le curseur sont conservés et le redimensionnement met à jour le terminal enfant. L'histoire conserve jusqu'à 10 000 lignes jusqu'à la prochaine opération ou jusqu'à la fermeture de l'application. Une seule transaction s'exécute à la fois ; masquer son terminal n'autorise pas l'application à se quitter pendant son exécution. Aucun shell libre n'est ouvert.

L'aperçu de la mise à jour lit les bases de données de synchronisation existantes (`pacman -Su --print`) sans les modifier ni nécessiter les privilèges root. Après confirmation, la mise à jour complète exécute `pacman -Syyu` via l'assistant privilégié avec `pkexec` ; la synchronisation peut révéler des mises à jour supplémentaires. L'installation, la suppression de paquets, la mise à jour des bases de données, le nettoyage du cache, la rétrogradation, les changements de dépôt/clé et les écritures sur le miroir utilisent également cet assistant privilégié. Les assistants AUR continuent à se construire en tant qu'utilisateur.

**Dépôts officiels → Dépôts** ajoute et gère les référentiels pacman non officiels. **Ajouter** est un brouillon : choisissez la **Source** de la configuration (**URL** d'un fichier `.conf`, **Texte** tapé ou collé dans un éditeur multiligne, ou **Formulaire** avec **Nom**, **Serveur** et **SigLevel**, écrit dans la mise en page Arch Wiki) et, sous **Clé de signature**, une **Clé** à partir d'une **URL** ou d'un **Texte** (ou **Aucun**). Les choix se déplacent avec `←`/`→` ou `Enter` ; les champs de texte ouvrent un éditeur dans lequel `Enter` enregistre une seule ligne, `Ctrl+S` enregistre plusieurs lignes, le collage fonctionne et `Esc` annule. Les téléchargements acceptent uniquement les adresses `https://`. **Review and add** télécharge et vérifie tout en tant qu'utilisateur, puis demande une confirmation, répertoriant les référentiels, les serveurs et SigLevel ainsi que l'ID utilisateur et l'empreinte digitale de la clé par groupes de quatre, à comparer avec celui publié par le référentiel. La confirmation importe la clé et la signe localement (`pacman-key --add`, `--lsign-key`), écrit `/etc/pacman.d/<name>.conf` (nommé d'après sa première section) et ajoute `Include = /etc/pacman.d/<name>.conf` à `/etc/pacman.conf` (une ligne commentée existante pour le même fichier est activée à la place). Ensuite, le Centre de Contrôle propose `pacman -Syyu`, de sorte que les nouvelles bases de données sont synchronisées sans mise à niveau partielle. Laisser **Ajouter** avec un brouillon demande avant de le jeter. **Lister** montre chaque dépôt non officiel, écrit dans `pacman.conf` lui-même ou inclus depuis `/etc/pacman.d/`, activé ou désactivé ; les sections officielles ne sont jamais listées. Une page de dépôt montre son fichier, SigLevel, serveurs et clé enregistrée, un interrupteur **Activé** et, dans la **Zone de danger**, **Supprimer**. Désactiver commente la ligne `Include`, ou préfixe chaque ligne d'une section en ligne avec `#argvus-off ` ; supprimer supprime la ligne ou la section et le fichier inclus. Les dépôts du même fichier inclus sont activés, désactivés et supprimés ensemble. Lorsque le Centre de Contrôle importé la clé, **Supprimer** offre également de la retirer du trousseau de clés de pacman (`pacman-key --delete`). Chaque modification apportée à `/etc/pacman.conf` conserve une sauvegarde à côté (`pacman.conf.argvus-backup-…`), et les paquets installés ne sont jamais supprimés.

### Utilisateurs et groupes

Lorsque le fournisseur de compte est disponible, ouvrez **Panneau de configuration → Administration du système → Utilisateurs et groupes → Utilisateurs** ou recherchez **utilisateurs**. L'interface peut lister les comptes normaux et systèmes, créer un utilisateur, modifier les métadonnées du compte, gérer les groupes supplémentaires et principaux, changer ou verrouiller un mot de passe, le déverrouiller, exiger un changement de mot de passe lors de la prochaine connexion, définir ou supprimer un avatar, et activer la connexion automatique.

La création d'utilisateur demande un nom d'utilisateur, un nom complet, un shell, des groupes supplémentaires et une confirmation de mot de passe facultative. Un mot de passe vide crée le compte avec le mot de passe verrouillé ; il ne crée pas silencieusement un mot de passe utilisable. Une page utilisateur regroupe les champs du compte (nom complet, shell, groupes principaux et supplémentaires, suivi de **Enregistrer les modifications**), le bouton bascule **Connexion automatique**, les actions **Mot de passe** (changer, verrouiller, déverrouiller, exiger un changement au prochain login) et les actions **Avatar**. Le nom d'utilisateur, l'UID / GID et le répertoire personnel sont affichés à titre d'information uniquement. La **Zone de danger** à la fin contient les actions destructrices séparées pour supprimer le compte tout en conservant son répertoire personnel ou pour supprimer également le répertoire personnel. Lisez attentivement le texte de confirmation avant de choisir cette dernière option. Les opérations sur le compte sont privilégiées et peuvent ouvrir l'invite d'autorisation du système. Les avatars peuvent être affichés par l'écran de connexion via le visage standard du compte. intégration d'image.

**Connexion automatique** démarre la session du compte sélectionné au démarrage suivant sans afficher l'écran de connexion, selon la même convention que GNOME, LightDM et SDDM utilisent pour cette fonctionnalité : aucun mot de passe n'est stocké nulle part. L'activer pour un compte désactive cette option pour tout compte précédemment configuré (un seul compte peut se connecter automatiquement à la fois). Cela s'applique une seule fois, au démarrage suivant ; après une déconnexion manuelle, l'écran de connexion normal revient pour le reste de cette session. L'interrupteur est appliqué immédiatement, avec une invite de confirmation, et n'est disponible que pour les administrateurs.

Les groupes ont leur propre liste et pages d'édition, y compris la gestion des membres et la suppression de groupe. Le Centre de Contrôle ne remplace pas la politique de comptes du système : les comptes système protégés et les opérations nécessitant une autorisation restent soumis aux restrictions du système d'exploitation.

### Paramètres du Centre de Contrôle

**Paramètres** est la dernière entrée de l'accueil. Sa page **Langue** définit la langue de l'utilisateur via locale.conf, et sa page **Style** contrôle les icônes décoratives et la transparence du Centre de Contrôle. Le style ne gère pas le flou : le flou est un paramètre global de Hyprland, sous Hyprland > Flou. La page Style modifie la présentation de l'application de paramètres ; ce n'est pas un bouton qui désactive les services ARGVUS ou supprime les fournisseurs de paramètres. L'enregistrement des icônes peut nécessiter une autorisation, tandis que le changement visuel est appliqué à l'application en cours. Échap sur Style retourne à la page Paramètres, et Échap là-bas retourne à l'accueil.

### À propos d'ARGVUS

**À propos d'ARGVUS** se trouve dans **Outils & Diagnostics → À propos d'ARGVUS** (recherchez `about`, `hardware` ou `firmware`, ou exécutez `argvus control-center about`). Il possède cinq onglets, basculés avec `Tab`/`Shift+Tab`, `←`/`→` ou `h`/`l` : **Système & Matériel**, **À propos**, **Faire un don**, **Crédits** et **Droits d'auteur**. L'en-tête montre le chemin complet, par exemple `Control Center > Tools & Diagnostics > About ARGVUS > System & Hardware`.

**Système & Matériel** est en lecture seule et regroupe les informations de cette machine en trois sections, avec les valeurs alignées dans une colonne :

| Section | Champs |
| --- | --- |
| Démarrage | Micrologiciel (fournisseur, version et date), Type (UEFI ou Hérité (BIOS)), Démarrage sécurisé (UEFI uniquement), Virtualisation, Chargeur de démarrage |
| Système | ARGVUS, Nom d'hôte, Système d'exploitation, Distributeur, Noyau, Architecture, Gestionnaire de fenêtres (Hyprland), Compositeur (Wayland), Pilote GPU, GTK, Thème (avec Sombre ou Clair), Écrans (une ligne par moniteur), Audio (PipeWire), Shell (Quickshell), Connexion (greetd), Init (systemd), Paquets, Temps de fonctionnement, Paramètres régionaux |
| Matériel | Batterie (une ligne par batterie), CPU (avec sa fréquence maximale), Threads, Cœurs, GPU, Mémoire (utilisée / totale), Swap, Stockage, Modèle (uniquement lorsqu'il diffère de la carte mère), Carte mère |

Chaque valeur est lue à partir du système en cours d'exécution, sans privilèges, et la page fonctionne de la même manière sur n'importe quelle machine. Une valeur qui ne peut pas être lue est affichée comme `N/A`, et une ligne qui ne s'applique pas est laissée de côté :

- **Firmware**, **Modèle** et **Carte mère** proviennent du DMI (`/sys/class/dmi/id`) ; les valeurs fictives telles que `To be filled by O.E.M.` sont ignorées. **Type** et **Démarrage sécurisé** proviennent de `/sys/firmware/efi`, et **Virtualisation** de `systemd-detect-virt`.
- **Chargeur d’amorçage** est lu à partir de la variable EFI `LoaderInfo`, que systemd-boot, Limine et d’autres chargeurs d’amorçage suivant l’Interface du chargeur d’amorçage publient. GRUB ne le publie pas, donc il est reconnu par `/boot/grub` (ou `/efi/grub`) et affiché avec la version du paquet `grub`.
- Les versions d’ARGVUS, Wayland, Mesa, PipeWire, Quickshell, greetd, systemd et GRUB proviennent d’une requête `pacman -Q` et sont affichées sans l’époque et la version du paquet (`1:25.2.3-1` devient `25.2.3`). La version de Hyprland provient de `hyprctl version`, ou du paquet lorsque aucune session ne répond.
- **Pilote GPU** indique le pilote associé à chaque carte DRM dans `/sys/class/drm`. Un pilote propriétaire montre sa version de module (par exemple `nvidia 580.82.07`) ; un pilote intégré au noyau montre la version de Mesa (par exemple `amdgpu (Mesa 25.2.3)`).
- **Écrans** liste les moniteurs actifs de `hyprctl monitors -j` : connecteur, résolution, taux de rafraîchissement et échelle (l'échelle est omise si elle est à 1).
- **Paquets** compte tous les paquets installés, ceux installés explicitement et ceux étrangers (AUR ou construits localement), provenant de `pacman -Qq`, `-Qqe` et `-Qqm`.
- **Temps de fonctionnement** provient de `/proc/uptime` et correspond à la valeur au moment de l’ouverture de la page. **Paramètres régionaux** proviennent de `LANG` dans `/etc/locale.conf`.
- **Batterie** liste les batteries de l’ordinateur à partir de `/sys/class/power_supply`, sélectionnées par type, de sorte que les périphériques comme les souris sans fil ne sont pas comptés. Chaque ligne montre la charge, l’état, la santé (capacité maximale comparée à la capacité de conception) et les cycles de charge lorsque le micrologiciel les rapporte. Une machine sans batterie affiche **Non détecté**.
- **CPU** ajoute la fréquence maximale la plus élevée parmi tous les cœurs (cpufreq), ce qui correspond à la fréquence maximale nominale sur les processeurs hybrides; elle est omise lorsque cpufreq n’est pas disponible, comme dans la plupart des machines virtuelles. **Threads** et **Cœurs** proviennent de `/proc/cpuinfo`.
- **Mémoire** et **Swap** proviennent de `/proc/meminfo` (zram inclus); un système sans swap affiche **Désactivé**.
- **Stockage** décrit le disque sous `/` : son modèle, son type (NVMe, SSD, HDD ou eMMC/SD ; les disques virtuels n’affichent aucun type), le système de fichiers et l’espace utilisé et total. Le disque est identifié à travers les partitions, LUKS et LVM jusqu’au périphérique physique.

Les valeurs sont lues en arrière-plan lors du premier affichage de la page, donc ni le démarrage du Centre de Contrôle ni l’ouverture de la page n’attendent ces informations ; l’onglet affiche **Chargement des informations système...** en attendant. Appuyez sur `c` dans cet onglet pour copier le rapport complet en texte brut dans le presse-papiers (avec `wl-copy`), par exemple pour le coller dans un rapport de bug.

## Recherche et fonctionnalités optionnelles

L'écran d'accueil dispose d'un registre de routes consultable. Les termes de recherche incluent des concepts destinés aux utilisateurs tels que l'apparence, les thèmes, les fonds d'écran, les polices, les raccourcis clavier, la souris, le pavé tactile, les écrans, le réseau et la session. Les fournisseurs optionnels sont inclus uniquement lorsque leur fonctionnalité est compilée et que leur capacité d'exécution est détectée, de sorte que le menu visible est intentionnellement conscient des capacités.

Lorsque la recherche sur l'accueil est active, les touches imprimables sont saisies dans la requête, y compris `j` et `k` ; utilisez les touches fléchées `↑` et `↓` pour naviguer parmi les routes correspondantes. Appuyez sur `Enter` pour ouvrir la route sélectionnée, `Backspace` pour modifier la requête, et `Esc` pour effacer la requête ou quitter le mode de recherche.

## Flux de travail de l'apparence

Ouvrez **Apparence** pour voir les contrôles visuels intégrés. Ses groupes sont :

- **Bureau** — **Thème** (choisissez une famille incluse et son mode Sticky ou Spaced, ou exportez, importez, appliquez et supprimez vos propres thèmes, voir [Importation et exportation de thèmes](/docs/components/argvus-themes/import-and-export/)), **Mode d'apparence** (Sticky ou Spaced, indépendant du thème), **Couleur d'accent** (un accent fourni ou une couleur RVB valide à six chiffres), **Fond d'écran** (un fond d'écran fourni ou un fichier personnalisé) **Icônes** (un thème d'icônes installé : recherchez dans la liste avec `/`, choisissez-en un avec Espace ou Entrée, puis sélectionnez **Appliquer** ; rien ne change avant l'Application) et **Polices** (les cibles de police et les paramètres de rendu des polices, ouverts sur la page Polices).
- **Style de l'application** — la transparence du **Terminal** et du **Lanceur**.

La carte Apparence contient les surfaces qui utilisent ces paramètres : **Widget Telemetry** (activer la surface de télémétrie, sélectionner ses blocs disponibles et les réorganiser avec `Shift+Up`/`Shift+Down` sur un bloc sélectionné) et **Panneau de contrôle** (activer, désactiver et réorganiser les cartes du panneau avec `Shift+Up`/`Shift+Down` sur une carte sélectionnée, appliqué immédiatement via le même primitif que celui utilisé par le glisser-déposer du panneau réel).

Ces actions mettent à jour l'état logique ARGVUS et appliquent la configuration d'exécution affectée. Voir [Apparence](/docs/user-guide/appearance/), [Thèmes](/docs/user-guide/appearance/) et [Fenêtres et disposition](/docs/argvus-hyprland/windows-and-layout/).

## Utilisation du clavier

L'application de paramètres prend en charge la navigation au clavier. Chaque page est une liste unique : utilisez les touches fléchées ou `j`/`k` pour vous déplacer, `Enter` pour ouvrir ou exécuter la ligne sélectionnée, `Space` pour basculer, `Esc` pour revenir en arrière, `/` pour rechercher des listes et `?` pour obtenir de l'aide ; le pied de page affiche les clés qui s'appliquent à la ligne sélectionnée. Les actions sont des lignes de la liste, pas des boutons, et `Tab` ne s'y déplace pas, sauf sur les pages avec une barre **Actions** sous la liste (applications par défaut, fuseau horaire, disposition du clavier, paramètres régionaux du système et touches SSH, et créer/modifier un raccourci). Les lignes qui affichent uniquement des informations sont ignorées. Lorsqu'un champ de texte, une recherche ou un filtre de liste est actif, `q` et `?` sont saisis sous forme de texte au lieu de quitter ou d'ouvrir l'aide. Sur **Applications par défaut**, **Restaurer les paramètres par défaut** est le bouton dans la barre **Actions** sous la liste : `Tab` ou `←/→` le concentre et `Enter` demande la réinitialisation. **Réinitialiser les paramètres par défaut** sur les autres pages et **Restaurer tous les raccourcis** sont des lignes dans un **Danger zone** en fin de pages ; `r` demande la même réinitialisation, et dans la liste des raccourcis, `r` restaure le raccourci sélectionné après une confirmation. Dans **Langue et région → Disposition du clavier**, `/` concentre la zone **Recherche** en haut de la page et `Esc` la supprime ; Le `●` marque la disposition utilisée. Il suit la barre des tâches et le raccourci clavier qui font défiler les mises en page, et la page se met à jour en une seconde environ sans rien écrire dans la configuration. `Enter` définit la mise en page par défaut, la première avec laquelle Hyprland commence et celle intitulée `Default` dans ses détails ; `Space` ajoute ou supprime une mise en page (`[x]`). Ces touches ne modifient qu'un brouillon et le bouton **Appliquer** dans la barre **Actions** sous la liste écrit les mises en page et la valeur par défaut sur Hyprland en une seule étape. Dans la liste, `Tab` ou `←/→` déplace le focus sur cette barre pendant que **Appliquer** est activé (le pied de page affiche alors `←/→ Actions`) ; `←/→` parcourt ses boutons et `↑/↓` revient au liste. Quitter la page avec des modifications non appliquées demande d'abord une confirmation. Dans **Langue et région → Fuseau horaire**, la disposition est la même : `/` concentre la zone **Recherche** en haut, `Enter` sélectionne une zone dans un brouillon (marquée par `●`) et le bouton **Appliquer** dans la barre **Actions** sous la liste l'enregistre dans le `environment.d/20-timezone.conf` de l'utilisateur ; `Tab`, `←/→` et `↑/↓` fonctionnent comme sur la disposition du clavier. Partir avec une zone qui n'a pas été appliquée demande d'abord une confirmation. Dans ** Paramètres régionaux du système **, la disposition est la même que celle du fuseau horaire : `/` concentre la zone **Recherche** en haut, `Enter` ou `Space` marque ou décoche un paramètre régional (`[x]`) dans un brouillon, et le bouton **Appliquer** dans la barre **Actions** sous la liste écrit la sélection dans `/etc/locale.gen` et exécute `locale-gen`. Étant donné que cela régénère les paramètres régionaux du système, **Apply** demande d'abord une confirmation ; `Tab`, `←/→` et `↑/↓` fonctionnent comme sur les deux autres. Sur **Bureau et saisie → Raccourcis clavier → Tous Les raccourcis** la barre de recherche se trouve également en haut, avec `/` qui la met au point, mais la page n'a pas de barre **Actions** : sa liste conserve les sections de raccourcis et la **Zone de danger**, et le pied de page ne liste `r Restore default` que dans la liste elle-même, pas sur **Modifier le raccourci**.

Les pages qui collectent les modifications avant de les enregistrer (un utilisateur, un groupe, la création de compte, le changement de mot de passe et la configuration du pare-feu) terminent ce bloc par une ligne **Enregistrer** ou **Appliquer** qui reste grisée tant que rien n'a changé. Si retourner en arrière entraînerait la perte de modifications non enregistrées, le Centre de Contrôle demande d'abord confirmation. Chaque confirmation montre **Confirmer** et **Annuler** comme deux lignes avec le focus sur **Annuler** : `Enter` exécute la ligne ciblée, `y` confirme et `n` ou `Esc` annulent.

Dans **Apparence**, chaque page est une liste unique : le pied de page n'affiche que les touches qui s'appliquent à la ligne sélectionnée, et `Space` ouvre ou sélectionne également la ligne. Les pages qui collectent des modifications avant de les appliquer (Barre des tâches, Widget Télémetrie, Panneau de configuration et les curseurs d'effet) se terminent par une ligne **Appliquer**, qui reste atténuée jusqu'à ce que quelque chose change. Sélectionner **Appliquer** affiche l'écran de chargement uniquement lorsque la modification recrée une surface : une modification de la hauteur de la barre des tâches, du groupe d'utilitaires, de l'icône ou du format de date et d'heure redémarre la barre des tâches ; activer ou désactiver la Widget Télémetrie ou changer ses blocs ou son ordre redémarre le widget ; activer ou désactiver le Panneau de configuration, ou modifier ses cartes alors qu'il est activé, redémarre le panneau. L'écran reste alors jusqu'à ce que le dernier changement de la page soit appliqué, de sorte qu'aucune modification n'apparaît avant cela, et la surface est redémarrée une seule fois. Les bordures, la transparence, le flou, les animations, les interstices, les polices, le papier peint et les autres paramètres s'appliquent sur place, sans que le écran de chargement. Sur les lignes de pourcentage, `←`/`→` (ou `+`/`-`) changent la valeur par étapes de 5 et `Enter` vous permet de la taper ; utilisez `Esc` pour revenir en arrière. Si revenir en arrière devait annuler des modifications non appliquées, le Centre de Contrôle demande d'abord. Supprimer un thème personnalisé (`d`) ou remplacer un thème importé demande une confirmation ; le focus commence sur **Annuler**, `y` confirme et `n` ou `Esc` annulent.

## Persistance et réinitialisation

Les pages de paramètres appliquent les modifications via leur fournisseur propriétaire et sauvegardent l'état utilisateur pris en charge. Les modifications d'apparence sont stockées dans la configuration canonique, et `argvus-config` est le seul composant à écrire les fichiers consommateurs dérivés sous `~/.config/argvus/data/generated/` ; ces fichiers générés ne sont pas l'endroit pour faire une édition permanente. Les pages d'entrée et d'attribution de touches ont leur propre état persistant et actions de réinitialisation. D'autres modifications système peuvent nécessiter des autorisations ou un rechargement du service.

Il n'existe pas de réinitialisation globale pour chaque paramètre ARGVUS. Restaurez une modification depuis la page qui la possède, ou utilisez la procédure de récupération documentée de la fonctionnalité concernée.

Le support de réinitialisation est délibérément par domaine :

- **Apparence** fournit une réinitialisation de la couleur de surlignage à la valeur par défaut du thème actif. Il ne fournit pas de réinitialisation globale unique de l'apparence ; restaurez les autres valeurs d'apparence depuis leurs propres contrôles.
- **Polices** peut restaurer tous les paramètres de police, une cible, ou un paramètre individuel.
- **Applications par défaut** peut restaurer toutes les valeurs par défaut, une catégorie, ou un sélecteur individuel.
- **Raccourcis clavier** peut restaurer une liaison ou toutes les liaisons, puis recharger les liaisons de session générées.
- **Règles de fenêtre** se trouve dans **Bureau & Entrée → Hyprland → Fenêtres → Règles** (recherchez `window rules` ou exécutez `argvus control-center window-rules`). Chaque règle a une ligne **Espace de travail** (les étapes `Enter` ou `←/→` passent de 1 à 10) et une ligne **Classes de fenêtres** qui ouvre un champ de texte avec des expressions régulières séparées par des virgules. **Ajouter une règle** crée `rule-N` avec l’espace de travail 1 et aucune classe. Le **Supprimer la règle** de chaque règle se trouve dans la **Zone de danger** et demande une confirmation. Chaque modification est écrite dans `hyprland.window_rules` dans `argvus config` et recharge la configuration immédiatement ; les motifs de classe invalides sont rejetés et affichés comme une erreur.
- **Projets** se trouvent dans **Outils & Diagnostics → Projets** (recherchez `projects` ou exécutez `argvus control-center projects`). **Ajouter un dossier de projet** et **Ajouter un dossier racine** se trouvent en haut de la page et ouvrent un champ de texte avec le chemin ; le lanceur vérifie qu'il s'agit d'un répertoire et affiche l'erreur sinon, y compris lorsque le dossier ferait dépasser le nombre effectif de projets au-delà de 9 (`SUPER + CTRL + 1..9` ne peut pas en gérer plus). Chaque dossier configuré, marqué comme racine (ses sous-dossiers sont des projets) ou projet unique, obtient son propre bloc sous le formulaire avec une ligne **Supprimer** qui demande confirmation. La page appelle `argvus-projects`, donc les modifications sont écrites par le lanceur et s'appliquent la prochaine fois que `SUPER + O` ou `SUPER + CTRL + 1..9` s'exécute. Si `argvus-projects` n'est pas installé, la page l'indique.
- **Les autres pages du système** exposent des actions de réinitialisation, d'application, de suppression ou de restauration uniquement lorsque le fournisseur sous-jacent les prend en charge.

La suppression d'une préférence utilisateur peut amener un fournisseur à revenir à son état emballé/par défaut, mais supprimer des fichiers à la main n'est pas une procédure générale de récupération. Privilégiez l'action de réinitialisation de la page ou la commande documentée spécifique à la fonctionnalité.

## Points d'entrée CLI

L'assistant est `argvus-control-center`, exécuté via `argvus control-center`. Il peut ouvrir des zones ciblées directement, par exemple :

```sh
argvus control-center appearance themes
argvus control-center appearance wallpapers
argvus control-center input
argvus control-center keybindings
```
Exécutez `argvus control-center --help` pour les itinéraires disponibles dans la version installée. L'itinéraire en ligne de commande est un raccourci vers la même application de paramètres, et non un système de configuration séparé.

Les itinéraires directs utiles incluent `apps`, `window-rules`, `projects`, `snippets`, `ssh-keys`, `gpg-keys`, `firewall`, `fonts`, `locale`, `input`, `keybindings`, `language`, `config`, `system`, `hardware`, `services`, `dev-services`, `network` (avec les pages `status`, `connections`, `activate`, `add`, `interfaces`, `wifi`, `ethernet`, `vpn`, `dns` et `proxy`), `audio`, `bluetooth`, `boot`, `packages` (avec les pages `official`, `search`, `installed`, `updates`, `orphans`, `aur`, `aur-remove`, `aur-updates`, `repositories`, `repositories-add`, `repositories-list`, `cache`, `history`, `downgrade` et `mirrors`), `storage`, `system-doctor` (`diagnostics` et `diagnostic` restent alias), `about` (avec les onglets `system`, `about`, `donate`, `credits` et `copyright`), `power`, `session`, `displays` et `appearance`. L'apparence accepte également les itinéraires ciblés tels que `themes`, `wallpapers`, `icons`, `accents`, `effects`, `spaces`, `taskbar` et `widget-telemetry` ; l'affichage accepte `resolution`, `refresh`, `scale`, `position`, `orientation`, `primary`, `vrr` et `hdr`. Utilisez la sortie `--help` installée lors de la création de scripts car la disponibilité dépend toujours de la version installée.

## Connexe

- [Première configuration](/docs/getting-started/)
- Panneau de configuration
- [Apparence](/docs/user-guide/appearance/)
- [Raccourcis clavier](/docs/argvus-hyprland/keyboard-shortcuts/)
- [Souris et pavé tactile](/docs/argvus-hyprland/input/)

### Docteur du système

Ouvrez **Outils & Diagnostics → Docteur du système**, recherchez-le, ou exécutez `argvus control-center system-doctor`. Les précédentes `diagnostics` et `diagnostic` routes et sélecteurs de catégories restent disponibles. Appuyez sur `r` pour actualiser, Entrée pour inspecter une vérification, `o` dans un détail pour ouvrir sa page associée, et Échap pour revenir. La collecte s'exécute en arrière-plan et est en lecture seule.

Les vérifications utilisent les compteurs d'unités échouées de systemd, les outils graphiques et le snapshot de stockage pour l'utilisation du système de fichiers et la santé SMART. Un problème nécessite des preuves ; les outils indisponibles ou les données SMART inaccessibles sont indiqués comme indisponibles, et une sonde échouée ou expirée est considérée comme une erreur. Les catégories sans preuves collectées n'impliquent pas qu'une vérification a réussi. Les mises à jour des paquets utilisent la base de données locale de pacman et ne synchronisent pas les dépôts ; un fichier de verrouillage peut être obsolète et ne prouve pas qu'une opération est en cours. SMART peut nécessiter des autorisations et aucun accès élevé n'est demandé. Le résumé enregistre la dernière collecte terminée en temps Unix.

Storage et System Doctor partagent un snapshot en mémoire pendant jusqu'à 10 secondes. Appuyer sur `r` collecte explicitement des données récentes ; les requêtes simultanées réutilisent une collecte qui vient de s'achever. La sortie des sous-processus Diagnostic et Storage est limitée à 1 MiB par flux, et une sortie trop volumineuse est traitée comme une collecte échouée.

## Lumière nocturne

Ouvrez **Bureau & Saisie > Affichages > Lumière nocturne** ou exécutez `argvus control-center displays night-light`. Configurez la température, l'activation manuelle et les horaires locaux fixes via `argvus config display night-light`. Consultez [Night Light](/docs/user-guide/night-light/) pour les commandes, l'état et les exigences d'exécution.

## Instantané

Ouvrez **Centre de contrôle > Administration du système > Instantané**, recherchez `Snapper`, ou exécutez `argvus control-center snapshot`. La fonction `snapshot` Cargo est incluse dans `full` (par défaut), et elle est indépendante de `storage`. La page est marquée **Expérimentale** car la fonction est encore en développement et peut être incomplète.

Cette page gère uniquement une configuration Snapper existante dont `SUBVOLUME` est `/` et dont `FSTYPE` est `btrfs`. Elle vérifie le montage contenant `/` avec `findmnt` ; une configuration nommée `root` pointant ailleurs n'est pas acceptée. Plusieurs configurations correspondantes nécessitent une sélection. Le paquet Arch `snapper` est optionnel et doit être installé manuellement. Snapper doit prendre en charge `--jsonout` et les colonnes actives/par défaut explicites. Aucune configuration n'est créée automatiquement.

Sur une racine non-Btrfs, la page affiche **Ce système de fichiers n'est pas Btrfs** et n'offre aucune opération de snapshot. L'échec de détection, l'absence de Snapper, l'absence de configuration de la racine, le refus de permission et les erreurs de backend sont des états distincts. Si les requêtes d'utilisateur normal sont refusées, appuyez sur **a — Autoriser** pour demander l'accès Polkit. Les actions administratives s'exécutent dans l'assistant restreint `system-settings`, et non dans une interface TUI élevée ; une authentification peut être demandée à nouveau lors de requêtes ultérieures. Un agent graphique Polkit existant est requis pour l'authentification.

Touches : **↑/↓** sélectionner, **Entrée** détails/configuration, **c** créer, **d** supprimer, **p** politiques de rétention, **r** actualiser, **Échap** retour, **?** aide. Dans la vue de rétention, **↑/↓** sélectionne une politique, **e** modifier, **c** faire défiler les algorithmes de nettoyage et **Entrée** ouvrir la confirmation. La création ouvre une description optionnelle (jusqu'à 256 caractères imprimables) ; une description vide utilise « ARGVUS snapshot manuel ». La création et la suppression nécessitent toutes deux une confirmation, initialement centrée sur Annuler. La suppression s'applique à un instantané sélectionné et rejette l'instantané 0, l'instantané actuellement actif et l'instantané par défaut. L'assistant vérifie de nouveau ces conditions avant d'exécuter Snapper. Les listes se mettent à jour après les opérations ; une opération interrompue n'est jamais réessayée automatiquement. Inspectez d'abord la liste actualisée.

Pour comparer des instantanés, sélectionnez la base et appuyez sur **b**, puis sélectionnez une destination différente et appuyez sur **v**. La page montre les ID ordonnés, les comptes créés,
chemins supprimés et modifiés, et les six indicateurs de statut de Snapper. Sélectionnez un chemin et appuyez sur **Entrée** pour demander la différence de contenu ; **Échap** revient aux chemins modifiés puis à la liste des instantanés. Les indicateurs codent la création/suppression ou le contenu/le type en position 1, les permissions en 2, le propriétaire en 3, le groupe en 4, les attributs étendus en 5 et l’ACL en 6. Un point signifie aucun changement à cette position. Le statut liste les chemins ; diff montre le contenu textuel. La sortie de diff binaire/non UTF-8 n’est pas affichée. Les sorties de diff supérieures à 256 KiB sont signalées comme tronquées et non affichées. Les lignes de statut ambiguës ou malformées sont rejetées. Le format de ligne de Snapper ne peut pas représenter de manière non ambiguë les chemins contenant des sauts de ligne ; ces noms ne sont pas pris en charge. Les erreurs de permission peuvent être réessayées via **a — Autoriser** ; un instantané supprimé ou une configuration racine modifiée nécessite de rafraîchir la liste. Les tests de backend utilisent un faux `ProcessRunner` et ne nécessitent pas Snapper, Btrfs ou les droits root.

À partir de la comparaison, sélectionnez un chemin et appuyez sur **r** pour examiner la restauration du fichier à partir du snapshot de destination. L'assistant vérifie que les deux snapshots appartiennent toujours à la configuration principale, que le chemin est encore dans la comparaison, et que la source est un fichier régulier dans la vue standard `/.snapshots/<id>/snapshot` de Snapper. La différence limitée de Snapper est affichée lorsqu'il s'agit de texte ; les différences binaires ou trop volumineuses sont identifiées sans charger leur contenu complet dans l'interface utilisateur. **Entrée** ouvre une confirmation séparée nommant la source et la destination ; **Échap** annule sans modifier le fichier. La confirmation crée d'abord un snapshot de sécurité `single`, puis utilise Snapper `undochange` uniquement pour ce chemin et vérifie qu'il correspond au snapshot sélectionné. L'identifiant du snapshot de sécurité est signalé et le snapshot est conservé. La restauration partage le verrou utilisé par les mises à jour du paquet ARGVUS, donc elle ne peut pas chevaucher une transaction pacman officielle du Centre de Contrôle.

La première version n'accepte que les fichiers réguliers appartenant à root sous des répertoires parents appartenant à root et non accessibles en écriture par le groupe ou d'autres utilisateurs. Elle refuse les répertoires, les liens symboliques, les liens physiques, les fichiers spéciaux, les points de montage/sous-volumes, le répertoire interne de Snapper et les emplacements modifiables par l'utilisateur. Les fichiers manquants dans le snapshot sélectionné ne sont jamais supprimés. Le `undochange` de root de Snapper ne garantit pas la cohérence des applications ou des bases de données et n'est pas atomique ; le snapshot de sécurité est un point de récupération, et non une garantie. Snapper ne conserve que les métadonnées prises en charge par sa version. Les sous-volumes séparés et les fichiers en dehors de la racine configurée restent en dehors de l'opération.

Les snapshots manuels sont `single`, en lecture seule par défaut pour Snapper, sans algorithme de nettoyage assigné par cette page. Les minuteries existantes de Snapper et les opérations externes continuent à s'appliquer. Les snapshots ne sont pas des images complètes du système : sous-volumes séparés,
y compris un `/home` séparé, ne sont pas inclus. Cette version **ne fournit pas de restauration complète du système, d'installation ni de création d'une configuration Snapper**. Appuyez sur **p** sur une configuration racine prise en charge pour inspecter les paramètres de rétention connus. Les valeurs manquantes sont étiquetées comme des valeurs de secours ARGVUS, et non comme des valeurs lues à partir du modèle de création Snapper installé ; les clés inconnues sont uniquement à des fins de diagnostic. Appuyez sur **e** pour modifier une valeur prise en charge. La confirmation affiche les anciennes et nouvelles valeurs ; après l'enregistrement, la page lit à nouveau la configuration et la vérifie. Appuyez sur **c** pour faire défiler les algorithmes de nettoyage et sur **Entrée** pour examiner et confirmer un choix. Le nettoyage utilise l'algorithme officiel de Snapper ; il n'y a pas de prévisualisation exacte, et le nombre supprimé rapporté est mesuré en comparant les listes après l'exécution. Arch Snapper 0.13.2 fournit `snapper-timeline.timer` et `snapper-cleanup.timer` ; la création de la chronologie nécessite
le minuteur de la chronologie et les politiques de nettoyage configurées ne fonctionnent pas sans planification du nettoyage. La page ne lit que l'état de l'unité et n'active pas les minuteurs. Les limites d'espace nécessitent des qgroups Btrfs configurés et ne sont pas exposées ici.

### Opérations et instantanés des paquets

L'installation, la mise à jour et la suppression de paquets ne nécessitent pas Snapper, Btrfs ou une configuration d'instantané. **Tout mettre à jour** exécute `pacman -Syyu` via l'assistant privilégié avec `pkexec` ; les mises à jour officielles individuelles exécutent `pacman -S <package>`. ARGVUS ne crée pas d'instantanés automatiques pour ces opérations. La page des instantanés reste indépendante et conserve ses propres exigences Btrfs/Snapper. Les hooks configurés séparément dans pacman restent sous le contrôle du système.

### Validation manuelle

- Sur une racine ext4/xfs, ouvrez la page en anglais et en portugais. En portugais, vérifiez le message exact **Este sistema de arquivos não é Btrfs** et vérifiez que c/d n'ouvre pas une opération.
- Dans une VM Btrfs jetable, testez Snapper absent, Snapper sans configuration racine, et une configuration nommée `root` pointant vers un autre sous-volume.
- Avec une configuration racine existante, listez les snapshots, ouvrez les détails, créez un snapshot avec des espaces/caractères spéciaux, annulez une suppression, puis supprimez seulement le snapshot nouvellement créé après avoir confirmé son numéro et sa description.
- Vérifiez l'authentification refusée/annulée, les snapshots actifs/défaut protégés, actualiser, navigation au clavier, redimensionner et les deux langues. Ne testez pas la suppression sur les snapshots nécessaires à la récupération.

Seul le paquet `argvus` change. Reconstruisez à partir de `de/argvus` avec `make build` pour produire le paquet `.pkg.tar.zst` ; son installation est une action opérateur séparée.

La langue sous Langue et Région enregistre la locale dans le profil utilisateur (`~/.config/locale.conf`) sans modifier `/etc/locale.conf`. La session utilise cette préférence après une nouvelle connexion. Pendant l'accueil, F6 Suivant ou le bouton Suivant dans le pied de page enregistre l'examen et avance ; Maj+Tab met le bouton en focus et Entrée l'active, y compris sur les pages d'information.

Le Bluetooth apparaît dans l'accueil uniquement lorsque le Centre de Contrôle détecte un adaptateur. Les progrès existants pour une étape Bluetooth cachée sont préservés.

La langue et la disposition du clavier de la session sont des préférences personnelles et ne nécessitent pas de privilèges administratifs. Les nouveaux profils sont par défaut en anglais avec la disposition de clavier américaine ; les préférences personnelles existantes sont préservées. La date/heure, NTP, la génération de la locale système et la disposition du clavier de la console (TTY) conservent leur comportement administratif à l'échelle du système.

Pour un clavier de session brésilien, sélectionnez `br`, appliquez la disposition, puis sélectionnez `abnt2` sous Variante. Cela écrit la configuration personnelle de Hyprland sans autorisation administrative ; le mappage des touches de la console est séparé. ABNT2 utilise des touches mortes : `~` suivi de `a` produit `ã`, et `~` suivi de la barre d'espace produit `~`. L'application de la variante active sa disposition par défaut configurée ; les échecs d'activation sont signalés.

Le fuseau horaire enregistre TZ sous `$XDG_CONFIG_HOME/environment.d/20-timezone.conf` sans autorisation. Déconnectez-vous et reconnectez-vous pour que les applications le reprennent. La date et l'heure continuent d'afficher et de modifier l'horloge système dans le fuseau horaire du système, avec autorisation administrative ; NTP reste inchangé.

Les profils d'affichage et les affectations d'espace de travail sont appliqués via la configuration Lua générée et un rechargement de Hyprland. L'alimentation du moniteur utilise le répartiteur DPMS Lua pris en charge par Hyprland 0.56.2.

La puissance de rendu des ombres est limitée de 1 à 4 (par défaut 4) pour la compatibilité avec Hyprland 0.57. Les anciennes valeurs stockées supérieures à 4 sont limitées lors du chargement de la configuration.

## Balancement des fenêtres

Ouvrez **Bureau & Entrée → Hyprland → Animations → Fenêtres** et utilisez les contrôles **Balancement**. Recherchez `wobble`, ou ouvrez directement la page :

```sh
argvus control-center hyprland animations windows
```
Le balancement est désactivé par défaut. Ses valeurs par défaut ARGVUS sont rigidité `100`, amortissement `10`, masse `1`, intensité `1` et résolution du maillage `8`. La rigidité et la masse acceptent `0.0001–1000`, l'amortissement `0–1000`, l'intensité `0–2`, et le maillage un entier `2–32`. Une résolution de maillage plus élevée augmente le coût de rendu. Utilisez les touches fléchées pour ajuster les valeurs ou Entrée pour les saisir. **Restaurer les valeurs par défaut du balancement** modifie le brouillon ; **Appliquer** enregistre toutes les modifications d'animation en attente et recharge Hyprland une fois, sans redémarrer la session. Revenir dans Animations conserve le brouillon partagé.

Le commutateur global Animations supprime le balancement tout en conservant ses paramètres individuels. Les valeurs canoniques se trouvent sous `effects.wobble_*` dans l'état ARGVUS ; `hypr/state.lua` est généré et ne doit pas être modifié manuellement.

Cela nécessite Hyprland **0.57.0 ou plus récent**, appliqué par `argvus-hyprland`.
dépendance du paquet. La version 0.56 n'est pas prise en charge pour cette configuration. Le
l'intensité maximale suit la source [ publiée 0.57.0](https://github.com/hyprwm/Hyprland/blob/v0.57.0/src/config/values/ConfigValues.cpp),
ce qui le limite à `2`, même si une documentation plus récente peut répertorier `3`.

L'accueil s'ouvre avec le logo ARGVUS et BIENVENUE. Appuyez sur F6 Suivant pour afficher la liste de contrôle sans terminer une étape.

Les nouvelles installations utilisent par défaut le fuseau horaire personnel `America/New_York` et seule la disposition du clavier `us` est activée. Les paramètres personnels existants sont conservés ; le fuseau horaire du système reste inchangé.

Les options du module de la barre des tâches pour la batterie et le Bluetooth n'apparaissent que lorsque le matériel correspondant est détecté. Masquer "Garder éveillé" masque son indicateur sans modifier l'inhibiteur de suspension. Les utilitaires prennent en charge un tiroir survol rétracté. La présentation des icônes de l'espace de travail affiche les icônes des fenêtres ouvertes ; la barre système contrôle les icônes des applications en arrière-plan.

La page des raccourcis clavier inclut **sélecteur d'agent IA (package optionnel argvus-ai)** avec SUPER + A par défaut. Il utilise l'ID du catalogue app.ai, donc le remappage et la désactivation suivent la même configuration canonique keyboard_shortcuts que les autres entrées. Son lien exécutable Lua est fourni par argvus-ai ; l'entrée du catalogue seule ne l'active pas sans ce package optionnel.


Centre de contrôle > Apparence > Barre des tâches > Icônes > Modules optionnels inclut désormais **Agents IA**. Il est activé par défaut et est stocké dans /taskbar/icons/ai_agents_enabled. Activez-le ou désactivez-le dans le brouillon de la barre des tâches et appliquez pour le conserver et régénérer le groupe d'utilitaires ; le désactiver ne change pas le raccourci SUPER + A ni les entrées du lanceur d'applications.

Les interrupteurs de démarrage automatique modifient l’entrée `.desktop` de l’utilisateur ou créent une copie utilisateur d’une entrée système. Les noms, commandes et indicateurs d’activation proviennent de `[Desktop Entry]` ; les groupes d’actions supplémentaires du bureau restent inchangés.

Les confirmations de service conservent la portée utilisateur/système sélectionnée ainsi que l’unité et l’action. Une unité portant le même nom dans une autre portée ne peut pas rediriger une action confirmée ; les pages de détails conservent également la portée originale.

Le stockage inclut des points de montage imbriqués tels que `/home`, `/boot` et `/boot/efi` provenant de l'inventaire du système de fichiers. Le diagnostic utilise le même inventaire pour ses vérifications existantes de l'utilisation des systèmes de fichiers root et home.

Les nouveaux instantanés de restauration sécurisés utilisent l'algorithme de nettoyage `number` de Snapper. Leur suppression suit la politique de rétention configurée lorsque le nettoyage s'exécute. Les instantanés existants sans algorithme de nettoyage ne sont pas modifiés, et la création manuelle conserve sa politique existante.

Les comparaisons d'instantanés rapportent séparément les enregistrements de chemins non pris en charge. Les fichiers valides restent disponibles pour examen ; les chemins avec un codage invalide, des caractères de contrôle ou des composants non pris en charge n'offrent pas d'actions de comparaison ou de restauration.

Les paramètres d'alimentation écrivent le logind ARGVUS sous `[Login]`, en créant son répertoire si nécessaire. Les échecs de rechargement sont signalés. Le comportement du capot de la batterie est mappé à `HandleLidSwitch` ; le contexte hérité `default` est un alias, tandis que le comportement sur secteur et en station d'accueil utilise ses propres clés logind.

L'application Mirrorlist n'accepte que les commentaires, les lignes vides et les entrées HTTP/HTTPS `Server`. Les charges utiles sont limitées à 64 KiB et vérifiées avant l'élévation et à nouveau par l'assistant privilégié ; les directives pacman telles que `SigLevel` et `XferCommand` sont rejetées.

Le nettoyage du cache pour les paquets désinstallés ne conserve aucune version mise en cache. Les opérations de dépôt refusent les inclusions partagées par plusieurs sections de dépôt. Les noms d'hôtes entourés d'espaces et une sélection de locale générée vide sont rejetés. Les écritures de fichiers de configuration refusent les destinations liées par des liens symboliques et les fichiers temporaires existants.

Les recherches de paquets et les chargements de page abandonnent les résultats d'une page ou d'une requête précédente. La page actuelle est alors rechargée, de sorte qu'un ancien résultat ne peut pas marquer une vue différente comme récemment actualisée.

La sortie des transactions est assainie et limitée. L'affichage conserve jusqu'à 10 000 lignes ou 4 Mio et signale la sortie omise. Les flux de processus capturés respectent leur limite configurée, par défaut à 8 Mio pour l'exécution en direct ; l'excédent de sortie est drainé et signalé après la fin du processus.

La lecture de l'inventaire de démarrage utilise le noyau Arch `pkgbase` pour localiser les images du noyau, les fichiers initramfs et les préréglages mkinitcpio tout en affichant la version du noyau séparément. Les tableaux littéraux mkinitcpio sont lus sans inclure leurs parenthèses ni exécuter la configuration.

Les modifications d'alimentation pour le couvercle, le bouton d'alimentation et le maintien en veille sont enregistrées dans la configuration canonique uniquement après que l'action d'exécution réussit. L'annulation de l'autorisation laisse la valeur précédemment enregistrée intacte. Lorsqu'aucun module complémentaire logind ARGVUS ne fournit de valeur, la configuration principale de logind est lue comme solution de rechange.

Après qu'un changement de rétention échoue, la page relit les paramètres actuels avant de montrer à nouveau leurs valeurs. La sélection de la configuration commence à la première entrée.

Les opérations du paquet ARGVUS partagent un verrou exclusif avec la restauration sélective. L'installation, la suppression, la réinstallation, la rétrogradation, l'actualisation des métadonnées, le nettoyage du cache ainsi que les modifications du dépôt ou du miroir sont refusées tant que ce verrou est maintenu. Cette coordination couvre les opérations lancées via l'assistant ARGVUS ; les outils externes pacman et AUR n'utilisent pas ce verrou.

Tant qu'une opération en arrière-plan applique des modifications, `q` et `Ctrl+C` maintiennent le Centre de contrôle ouvert et affichent un avis demandant d'attendre. Cela s'applique également après avoir quitté la page ayant déclenché l'action. Attendez son résultat, puis demandez à nouveau la sortie. Les actualisations ordinaires en lecture seule ne bloquent pas la sortie.

Les champs de mot de passe Wi-Fi, de DNS manuel et de recherche de réseau acceptent le collage entre crochets. Les contrôles et les sauts de ligne collés sont omis. Les raccourcis clavier Ctrl seuls ou Alt seuls non pris en charge n'insèrent pas de lettres. Les éditeurs acceptent les événements AltGr imprimables codés comme Ctrl+Alt ; Ctrl+S enregistre uniquement sans Alt. L'éditeur de texte partagé maintient la navigation verticale dans les colonnes visuelles et aligne le défilement horizontal pour les caractères larges.

Les modifications du volume audio et de la mise en sourdine pour les microphones et les sorties non par défaut n'écrasent plus les préférences de sortie par défaut enregistrées. Le backend conserve ces préférences uniquement lorsqu'il peut identifier le nœud sélectionné comme sortie par défaut actuelle. La page affiche des volumes amplifiés au-dessus de 100 % tels que rapportés par PipeWire ; les modifications explicites du volume restent limitées à 0–100 %.

L'aperçu matériel identifie les modèles de GPU séparément de leur adresse PCI et de leur classe de périphérique, en préservant les noms commerciaux entre crochets. Son résumé affiche le moteur OpenGL interrogé pour la session et tous les noms de périphériques renvoyés par la sonde Vulkan, y compris les périphériques logiciels. Ces résultats globaux ne sont pas présentés comme des capacités de chaque GPU physique.

Lorsqu'un administrateur verrouille ou supprime le compte configuré pour la connexion automatique, le fournisseur de compte désactive d'abord son entrée de connexion automatique greetd. Si ce nettoyage échoue, l'opération sur le compte est refusée. Si le verrouillage ou la suppression échoue par la suite, la connexion automatique reste désactivée. La connexion automatique pour un autre compte est préservée ; le déverrouillage d'un compte n'active pas la connexion automatique. Les sessions existantes ne sont pas fermées et greetd n'est pas redémarré.

## Créer et gérer des raccourcis clavier

**Bureau & Entrée → Raccourcis clavier** ouvre un menu avec **Créer des raccourcis** et **Tous les raccourcis**. Les alias CLI existants (`keybindings`, `shortcuts`, `keyboard`) ouvrent ce menu ; ajoutez `create` ou `list` pour ouvrir directement un enfant. La recherche globale trouve également les deux enfants.

**Créer des raccourcis** demande une description, une combinaison de touches et un type d'action. Sélectionnez **Raccourci actuel** et appuyez sur `Enter`, puis appuyez sur la combinaison de touches souhaitée. **Modifier le raccourci** démarre la même capture ; aucun des deux n'ouvre un éditeur de texte. La capture ne change que le brouillon. Si une combinaison est déjà utilisée, le conflit est affiché immédiatement, même avant que la description et la commande ne soient remplies. Confirmer **Remplacer** sélectionne la combinaison et enregistre votre choix ; cela ne crée pas le raccourci ni ne désactive un autre jusqu'à **Créer**/**Enregistrer**. Annuler le conflit préserve la combinaison précédente. Une **Commande** s'exécute lorsque le raccourci est pressé ; sa création ne l'exécute jamais. Un **dispatcheur Hyprland** utilise un nom de dispatcheur Lua pris en charge, tel que `window.float`, avec des arguments optionnels sous forme d'objet JSON, par exemple `{"action":"toggle"}`. Les sous-cartes, les liaisons de souris et les options avancées restent en dehors de ce formulaire.

Utilisez `Tab` pour passer entre le formulaire et **Actions**, `←/→` pour naviguer entre **Créer** et **Annuler**, et `Enter` pour exécuter l'action sélectionnée. Annuler ou `Esc` supprime le brouillon. Après la création, **Tous les raccourcis** s'ouvre avec le nouvel élément sélectionné et visible. Les entrées personnalisées apparaissent dans **Raccourcis personnalisés** ; la recherche correspond à la description, aux touches et à l'action. `Enter` modifie la description, l'action et les touches, `Space` active/désactive, et sélectionner la ligne **Supprimer** et appuyer sur `Enter` ouvre une confirmation centrée sur Annuler. Confirmer supprime l'enregistrement et revient à la liste actualisée ; annuler le conserve.

Les raccourcis activés dans le même contexte ne peuvent pas réutiliser une combinaison silencieusement : les conflits configurables nécessitent une confirmation pour être remplacés, et les raccourcis protégés ne peuvent pas être remplacés. Le remplacement enregistre le nouveau raccourci et désactive les conflits dans une seule transaction de configuration. **Restaurer par défaut** s'applique uniquement aux entrées officielles ; **Restaurer tous les raccourcis** préserve les enregistrements personnalisés. Si la restauration entre en conflit avec un raccourci personnalisé actif, modifiez ou désactivez d'abord ce raccourci personnalisé.

Enregistrer et appliquer s'exécutent en arrière-plan. Si l'enregistrement échoue, le brouillon reste disponible. Si l'application à la session échoue après l'enregistrement, la page indique qu'il a été enregistré et conserve le même ID pour réessayer. Les enregistrements personnalisés se trouvent dans `/hyprland/custom_shortcuts` dans la configuration canonique, projetés vers `data/generated/hypr/custom-shortcuts.lua` ; ne modifiez pas les fichiers générés.

Les erreurs de raccourci restent dans l'IU textuelle : la fenêtre contextuelle s'ajuste au message renvoyé à la ligne et garde **Entrée OK** visible. Les diagnostics techniques de raccourci vont dans le journal local (`journalctl -t argvus-control-center`) au lieu de stdout/stderr. Une suppression qui est enregistrée mais ne peut pas être appliquée indique cette distinction ; rechargez la configuration de la session pour supprimer l'association précédemment active.

**État de la mise à jour** dans **Barre des tâches → Icônes → Modules optionnels** affiche le nombre de mises à jour disponibles et est activé par défaut. **Vérifier AUR** est désactivé par défaut et nécessite `paru` ou `yay` ; appliquez l’ébauche de la barre des tâches pour enregistrer l’un ou l’autre réglage. Les vérifications s’exécutent au démarrage et toutes les heures ; cliquer sur l’indicateur ouvre la page officielle des mises à jour des paquets.

## Mode fenêtre globale

Ouvrez **Centre de contrôle → Bureau & Entrée → Hyprland → Fenêtres → Mode** et sélectionnez **Fenêtres en mosaïque (Par défaut)** ou **Fenêtres flottantes**. Le même sélecteur est disponible depuis l’icône du mode fenêtre dans le groupe d’utilitaires de la barre des tâches.

La préférence s’applique à chaque moniteur et espace de travail et est conservée après la déconnexion. Flottant convertit les fenêtres d’application existantes et les nouvelles fenêtres ouvertes. Mosaïque restaure uniquement les fenêtres que cette politique a converties ; les dialogues et autres fenêtres déjà flottantes restent flottantes. Une fenêtre épinglée explicitement alors que le mode Flottant est actif reste également flottante pour préserver son épinglage. La disposition en mosaïque est recalculée ; son arbre de division exact précédent n’est pas restauré.

`SUPER + SHIFT + Space` conserve son comportement normal par fenêtre en mode Tuile. En mode Flottant, il ne remet pas une fenêtre en mode tuile. Les panneaux, la barre des tâches et les notifications sont des surfaces layer-shell et ne sont pas affectés. Le plein écran et la maximisation continuent d'utiliser le comportement normal de Hyprland.

La préférence canonique est `/hyprland/window_mode` dans `$XDG_CONFIG_HOME/argvus/config/hyprland.json`. Sa projection générée est `data/generated/hypr/window-mode.lua` ; ne la modifiez pas. Le registre de restauration par fenêtre se trouve sous `$XDG_RUNTIME_DIR/argvus`, limité à l'instance du compositeur, et survit aux rechargements de configuration. Il n'est pas réutilisé dans une nouvelle session.

Si l'application est en attente ou échoue, l'interface distingue cela de la préférence enregistrée. Sélectionnez à nouveau l'option pour réessayer. Sans compositeur, la préférence est enregistrée pour la prochaine connexion.

## Boutons de fenêtre et barre d'application

Le mode flottant ajoute une barre de titre ARGVUS avec **Réduire**, **Agrandir / Restaurer** et **Fermer** aux fenêtres d'application normales. Double-cliquer sur la barre de titre bascule la maximisation. La décoration supplémentaire est cachée en mode mosaïque et en plein écran. Les applications qui dessinent leurs propres contrôles peuvent afficher les deux barres ; les contrôles ARGVUS uniformes sont intentionnels.

Une deuxième Waybar apparaît sur chaque moniteur, sur le bord opposé à la barre des tâches principale. Elle ne liste que les fenêtres de l'espace de travail actuel de ce moniteur, y compris ses fenêtres réduites. Les onglets de groupe cachés ne créent pas d'entrées supplémentaires. Les entrées réduites conservent l'association moniteur/espace de travail d'origine. Cliquez sur une fenêtre inactive pour l'activer, sur une fenêtre active pour la réduire, ou sur une fenêtre réduite pour la restaurer ; son menu contextuel offre restaurer, réduire, agrandir/restaurer et fermer. Les icônes d'application inconnues utilisent par défaut un symbole de fenêtre.

`SUPER + Alt + Up` place la barre principale en haut et la barre de fenêtre en bas. `SUPER + Alt + Down` fait l'inverse. Répéter une direction laisse les positions inchangées. Le carrelage n'a que la barre principale. Revenir au carrelage restaure toutes les fenêtres minimisées avant de supprimer la barre de fenêtre, puis préserve la politique de carrelage/flottante originale.

La minimisation utilise l’espace de travail réservé `special:argvus-minimized`. N’utilisez pas cet espace de travail manuellement. Le processus `argvus windows service` enregistre l’identité, l’espace de travail/écran d’origine, la géométrie, l’état plein écran/maximisé et l’état d’épinglage dans un registre d’exécution spécifique à l’instance du compositeur sous `$XDG_RUNTIME_DIR/argvus`. Les enregistrements survivent à un redémarrage du service ou à un rechargement de configuration, mais ne traversent pas les sessions de compositeur. La fermeture d’une fenêtre supprime son enregistrement. Si un écran disparaît, la restauration utilise une sortie disponible. Une barre de récupération manquante ou défaillante empêche de nouvelles minimisations ; la restauration échouée conserve l’enregistrement et la barre de récupération.

La session possède `argvus-windows.service` et `argvus-window-bar.service`. Ce dernier maintient son enfant Waybar inactif dans Tile. Le plugin hyprbars est fourni par `argvus-hyprland`, compilé pour l’ABI Hyprland 0.57.0 sur x86_64 ; il n’est pas compilé dans le répertoire personnel de l’utilisateur. Reconstruisez le plugin lors d’une mise à jour de l’ABI du compositeur.
Commandes : `argvus windows list --json`, `watch --json`, `position top|bottom`, `restore-all` et `minimize|restore|activate|maximize|close ADDRESS`. Les adresses de fenêtres proviennent de la liste. `position top --persist` stocke/projette la préférence sans redémarrer la barre principale. Les détails de configuration et d'erreur d'exécution sont écrits dans le journal ; les échecs d'action visibles affichent également une notification traduite.

## Sous-menu Fenêtres et fermeture de l'application

**Bureau & Entrée → Hyprland → Fenêtres** regroupe **Règles**, **Espaces** et **Mode**.
Règles modifie le placement par application ; Espaces modifie les marges intérieures et extérieures ; Mode contient le sélecteur immédiat Carrelage/Flottant, séparé du formulaire de règles.
Recherche ouvre les mêmes pages, et Retour revient à Fenêtres avant Hyprland.

Les itinéraires directs sont `argvus control-center hyprland windows`, suivis éventuellement de `rules`, `spaces` ou `mode`. Les itinéraires existants `window-rules` et `appearance window-spaces` restent des alias et affichent les nouvelles fil d'Ariane.

`SUPER + Q` ferme immédiatement la fenêtre dédiée du Centre de Contrôle, sans confirmation de la fenêtre de l'OS de Kitty. Les brouillons non appliqués sont supprimés lorsque la fenêtre est fermée de cette manière. L'override est limité à la classe de fenêtre `argvus-control-center` ; les terminaux réguliers et autres TUI conservent leur politique de confirmation de fermeture.

`SUPER + Alt + Space` bascule le mode tuile/flottant global, en utilisant la même préférence que la barre des tâches et le Centre de contrôle. La commande équivalente est `argvus config window-mode toggle`. Les invocations concurrentes sont sérialisées ; le basculement utilise la préférence enregistrée même lorsque l'application est en attente. Le raccourci peut être modifié ou désactivé dans les Raccourcis clavier ; `SUPER + Shift + Space` conserve son action pour une fenêtre individuelle.
