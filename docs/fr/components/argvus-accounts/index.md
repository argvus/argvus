---
title: Comptes
description: Profils d'utilisateur locaux, noms d'affichage, avatars et appartenance à des groupes, via des comptes argvus.
---
`argvus accounts` répertorie et modifie les comptes utilisateur locaux du bureau ARGVUS. C'est un exécutable privé du paquet `argvus`, installé à `/usr/lib/argvus/commands/accounts`. La commande `argvus` le lance, et seules les opérations qui modifient d'autres comptes ou les données système s'exécutent en tant que root.

## Commandes

```text
argvus accounts list [--all]                   list users (--all includes system accounts)
argvus accounts show USER                      show account details
argvus accounts name USER "DISPLAY NAME"       change another user's display name
argvus accounts avatar USER IMAGE              set another user's avatar
argvus accounts avatar USER --remove           remove another user's avatar
argvus accounts groups USER                    list groups
argvus accounts groups USER --add G --remove G change groups (authorization required)
argvus accounts passwd USER                    change a password (reads three lines from stdin)
argvus accounts self                           show your own account
argvus accounts self name "DISPLAY NAME"       change your own display name
argvus accounts self avatar IMAGE              set your avatar
argvus accounts self avatar --remove           remove your avatar
argvus accounts self groups                    list your groups
argvus accounts --verbose ...                  print diagnostic information to stderr
```
## Noms affichés et avatars

Le nom affiché est stocké dans le champ standard GECOS. Le nom d'utilisateur Unix n'est jamais modifié.

Un avatar est une image au format PNG, JPEG ou WebP, jusqu'à 20 MiB. L'image est redimensionnée à 256×256 pixels et enregistrée au format PNG avec le mode `0644` à `~/.face`, l'emplacement freedesktop lu par le greeter et par la plupart des gestionnaires d'affichage. Votre propre avatar est enregistré dans votre répertoire personnel et ne nécessite pas d'autorisation.

## Autorisation

Seuls `list`, `show`, `self`, `self groups` et votre propre avatar fonctionnent sans autorisation. Tout autre changement demande l'autorisation de l'agent polkit du bureau. La règle ARGVUS (`/usr/share/polkit-1/rules.d/50-argvus-accounts.rules`) décide qui doit s'authentifier :

- un utilisateur dans un groupe d'administrateurs (`wheel`, la valeur par défaut d'Arch, ou `sudo`) confirme avec son propre mot de passe ;
- tout autre utilisateur confirme avec le mot de passe root.

| Opération | Sans autorisation | Avec autorisation |
| --- | --- | --- |
| Lister les utilisateurs, afficher un compte, lister les groupes | ✓ | ✓ |
| Changer son propre avatar | ✓ | ✓ |
| Changer son propre nom d'affichage | — | ✓ |
| Changer son propre mot de passe | — | ✓ (après avoir prouvé le mot de passe actuel) |
| Changer le nom, l'avatar, les groupes ou le mot de passe d'un autre utilisateur | — | ✓ |

Changer son propre mot de passe vérifie d'abord le mot de passe actuel. Un mot de passe incorrect est refusé avant l'apparition de l'invite d'autorisation.

Il n'est pas nécessaire d'exécuter la commande avec `sudo`. La commande demande l'autorisation à polkit lorsqu'elle en a besoin.

## Limites de sécurité

- `passwd` lit le mot de passe actuel, le nouveau mot de passe et la confirmation depuis l'entrée standard, un par ligne, et ne les prend jamais comme arguments. La ligne du mot de passe actuel peut être vide lorsqu'un administrateur réinitialise le mot de passe d'un autre utilisateur. Exemple :

  ```sh
  printf '%s\n%s\n%s\n' "$OLD" "$NEW" "$NEW" | argvus accounts passwd ghost
  ```

  De manière interactive, tapez les trois lignes et appuyez sur Entrée après chacune. L'entrée apparaît sur le
  terminal au fur et à mesure de la saisie.
- La modification d'un compte avec un UID inférieur à 1000 (et `nobody`) est refusée, tout comme le verrouillage, la suppression
  et l'expiration de son mot de passe. Votre propre compte ne peut pas être verrouillé ou supprimé.
- Renommer ou supprimer un groupe avec un GID inférieur à 1000, ou un groupe d'administrateurs (`wheel`, `sudo`), est
  refusé ; ses membres peuvent toujours être modifiés, sauf que vous ne pouvez pas vous retirer d'un
  groupe d'administrateurs.
- L'appartenance à un groupe d'administrateurs correspond aux groupes `wheel` ou `sudo`. Le Centre de Contrôle ajoute de nouveaux
  administrateurs à `wheel` lorsque le système le possède, et à `sudo` sinon. La suppression de votre propre
  appartenance à un groupe d'administrateurs via une modification est refusée.

## Exemples

```sh
# Set your display name and avatar after the first login
argvus accounts self name "William Canin"
argvus accounts self avatar ~/Pictures/profile.png

# Check your groups, then add another user to the wheel group (authorization required)
argvus accounts self groups
argvus accounts groups ghost --add wheel

# Administrator: show a user's details and reset that user's name
argvus accounts show ghost
argvus accounts name ghost "Ghost Account"
```
## Dépannage

- `pkexec was not found` : installez `polkit`. Le paquet `argvus` en dépend.
- L'invite d'autorisation n'apparaît pas : vérifiez qu'un agent d'authentification polkit est en cours d'exécution dans votre session. Le paquet `argvus` dépend de `hyprpolkitagent`, qui en fournit un.
- Erreurs `unix_chkpwd` lors du changement de mot de passe : le paquet `pam` fournit l'aide.
- Exécutez `argvus accounts --verbose ...` pour afficher les étapes de la commande sur stderr.

Pour les notes du développeur sur la limite de privilèges, voir `docs/internal/argvus-accounts/`.

Lorsque un administrateur verrouille ou supprime le compte configuré pour la connexion automatique, le fournisseur de compte désactive d'abord son entrée de connexion automatique greetd. Si ce nettoyage échoue, l'opération sur le compte est refusée. Si le verrouillage ou la suppression échoue ultérieurement, la connexion automatique reste désactivée. La connexion automatique pour un autre compte est préservée ; le déverrouillage d'un compte n'active pas la connexion automatique. Les sessions existantes ne sont pas fermées et greetd n'est pas redémarré.
