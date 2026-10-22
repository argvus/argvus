---
title: Accounts
description: Local user profiles, display names, avatars and group membership, through argvus accounts.
---

`argvus accounts` lists and edits the local user accounts of the ARGVUS desktop. It is a private
executable of the `argvus` package, installed at `/usr/lib/argvus/commands/accounts`. The
`argvus` command starts it, and only the operations that change other accounts or system data
run as root.

## Commands

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

## Display names and avatars

The display name is stored in the standard GECOS field. The Unix username is never changed.

An avatar is an image in PNG, JPEG or WebP format, up to 20 MiB. The image is resized to 256×256
pixels and saved as PNG with mode `0644` at `~/.face`, the freedesktop location read by the
greeter and by most display managers. Your own avatar is written inside your home directory and
does not require authorization.

## Authorization

Only `list`, `show`, `self`, `self groups` and your own avatar work without authorization. Every
other change asks the desktop's polkit agent for authorization. The ARGVUS rule
(`/usr/share/polkit-1/rules.d/50-argvus-accounts.rules`) decides who must authenticate:

- a user in the `sudo` group confirms with their own password;
- any other user confirms with the root password.

| Operation | Without authorization | With authorization |
| --- | --- | --- |
| List users, show an account, list groups | ✓ | ✓ |
| Change your own avatar | ✓ | ✓ |
| Change your own display name | — | ✓ |
| Change your own password | — | ✓ (after proving the current password) |
| Change another user's name, avatar, groups or password | — | ✓ |

Changing your own password first checks the current password. A wrong password is refused
before the authorization prompt appears.

There is no need to run the command with `sudo`. The command asks polkit for authorization when
it needs it.

## Security limits

- `passwd` reads the current, new and confirmation passwords from standard input, one per line,
  and never takes them as arguments. The current password line may be empty when an administrator
  resets another user's password. Example:

  ```sh
  printf '%s\n%s\n%s\n' "$OLD" "$NEW" "$NEW" | argvus accounts passwd ghost
  ```

  Interactively, type the three lines and press Enter after each one. The input is shown on the
  terminal as it is typed.
- Editing an account below UID 1000 (and `nobody`) is refused, and so are locking and deleting
  it. Your own account cannot be locked or deleted.
- Administrator membership is the `sudo` group. Removing your own administrator membership
  through an edit is refused.

## Examples

```sh
# Set your display name and avatar after the first login
argvus accounts self name "William Canin"
argvus accounts self avatar ~/Pictures/profile.png

# Check your groups, then add another user to the sudo group (authorization required)
argvus accounts self groups
argvus accounts groups ghost --add sudo

# Administrator: show a user's details and reset that user's name
argvus accounts show ghost
argvus accounts name ghost "Ghost Account"
```

## Troubleshooting

- `pkexec was not found`: install `polkit`. The `argvus` package depends on it.
- The authorization prompt does not appear: check that a polkit authentication agent is running
  in your session. The `argvus` package depends on `hyprpolkitagent`, which provides one.
- `unix_chkpwd` errors when changing a password: the `pam` package provides the helper.
- Run `argvus accounts --verbose ...` to print the steps of the command to stderr.

For developer notes about the privilege boundary, see `docs/internal/argvus-accounts/`.
