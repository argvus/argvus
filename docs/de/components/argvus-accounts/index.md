---
title: Konten
description: Lokale Benutzerprofile, Anzeigenamen, Avatare und Gruppenmitgliedschaften über argvus-Konten.
---
`argvus accounts` listet und bearbeitet die lokalen Benutzerkonten des ARGVUS-Desktops. Es ist ein privates ausführbares Programm des `argvus`-Pakets, installiert bei `/usr/lib/argvus/commands/accounts`. Der Befehl `argvus` startet es, und nur die Operationen, die andere Konten oder Systemdaten ändern, laufen als Root.

## Befehle

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
## Anzeigenamen und Avatare

Der Anzeigename wird im Standard-GECOS-Feld gespeichert. Der Unix-Benutzername wird niemals geändert.

Ein Avatar ist ein Bild im PNG-, JPEG- oder WebP-Format, bis zu 20 MiB. Das Bild wird auf 256×256 Pixel skaliert und als PNG mit dem Modus `0644` unter `~/.face` gespeichert, dem freedesktop-Standort, den der Begrüßungsbildschirm und die meisten Display-Manager auslesen. Dein eigener Avatar wird in deinem Home-Verzeichnis gespeichert und erfordert keine Autorisierung.

## Autorisierung

Nur `list`, `show`, `self`, `self groups` und dein eigener Avatar funktionieren ohne Autorisierung. Jede andere Änderung verlangt vom polkit-Agenten des Desktops eine Autorisierung. Die ARGVUS-Regel (`/usr/share/polkit-1/rules.d/50-argvus-accounts.rules`) entscheidet, wer sich authentifizieren muss:

- Ein Benutzer in einer Administratorgruppe (`wheel`, der Arch-Standard, oder `sudo`) bestätigt mit seinem eigenen Passwort;
- Jeder andere Benutzer bestätigt mit dem Root-Passwort.

| Vorgang | Ohne Autorisierung | Mit Autorisierung |
| --- | --- | --- |
| Benutzer auflisten, Konto anzeigen, Gruppen auflisten | ✓ | ✓ |
| Eigenes Avatar ändern | ✓ | ✓ |
| Eigenen Anzeigenamen ändern | — | ✓ |
| Eigenes Passwort ändern | — | ✓ (nach Nachweis des aktuellen Passworts) |
| Namen, Avatar, Gruppen oder Passwort eines anderen Benutzers ändern | — | ✓ |

Beim Ändern des eigenen Passworts wird zuerst das aktuelle Passwort überprüft. Ein falsches Passwort wird abgelehnt, bevor die Autorisierungsabfrage erscheint.

Es ist nicht erforderlich, den Befehl mit `sudo` auszuführen. Der Befehl fragt bei Bedarf die Autorisierung über polkit ab.

## Sicherheitsgrenzen

- `passwd` liest das aktuelle, das neue Passwort und die Passwortbestätigung zeilenweise von der Standardeingabe und nimmt sie niemals als Argumente entgegen. Die Zeile für das aktuelle Passwort kann leer sein, wenn ein Administrator das Passwort eines anderen Benutzers zurücksetzt. Beispiel:

  ```sh
  printf '%s\n%s\n%s\n' "$OLD" "$NEW" "$NEW" | argvus accounts passwd ghost
  ```

  Interaktiv geben Sie die drei Zeilen ein und drücken Sie nach jeder eine Eingabetaste. Die Eingabe wird
  auf dem Terminal angezeigt, während sie eingegeben wird.
- Das Bearbeiten eines Kontos unter UID 1000 (und `nobody`) wird verweigert, ebenso wie das Sperren,
  Löschen oder Ablaufsetzen des Passworts. Ihr eigenes Konto kann nicht gesperrt oder gelöscht werden.
- Das Umbenennen oder Löschen einer Gruppe unter GID 1000 oder einer Administratorgruppe (`wheel`, `sudo`)
  wird verweigert; ihre Mitglieder können weiterhin geändert werden, außer dass Sie sich selbst nicht aus einer
  Administratorgruppe entfernen können.
- Die Administratormitgliedschaft besteht in der `wheel`- oder `sudo`-Gruppe. Das Kontrollzentrum fügt neue
  Administratoren der `wheel`-Gruppe hinzu, wenn das System diese hat, ansonsten der `sudo`-Gruppe.
  Das Entfernen Ihrer eigenen Administratormitgliedschaft durch eine Bearbeitung wird verweigert.

## Beispiele

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
## Fehlerbehebung

- `pkexec was not found`: Installieren Sie `polkit`. Das `argvus`-Paket hängt davon ab.
- Die Autorisierungsaufforderung erscheint nicht: Überprüfen Sie, ob ein Polkit-Authentifizierungsagent in Ihrer Sitzung läuft. Das `argvus`-Paket hängt von `hyprpolkitagent` ab, das einen bereitstellt.
- `unix_chkpwd`-Fehler beim Ändern eines Passworts: Das `pam`-Paket stellt den Helfer bereit.
- Führen Sie `argvus accounts --verbose ...` aus, um die Schritte des Befehls an stderr auszugeben.

Weitere Hinweise für Entwickler zur Berechtigungsgrenze finden Sie unter `docs/internal/argvus-accounts/`.

Wenn ein Administrator das für die automatische Anmeldung konfigurierte Konto sperrt oder entfernt, deaktiviert der Kontenanbieter zunächst den greetd-Eintrag für die automatische Anmeldung. Wenn diese Bereinigung fehlschlägt, wird der Konto-Vorgang verweigert. Wenn das Sperren oder Entfernen später fehlschlägt, bleibt die automatische Anmeldung deaktiviert. Die automatische Anmeldung für ein anderes Konto bleibt erhalten; das Entsperren eines Kontos aktiviert die automatische Anmeldung nicht. Bestehende Sitzungen werden nicht beendet und greetd wird nicht neu gestartet.
