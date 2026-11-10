---
title: Installiere ARGVUS
description: Installieren, aktualisieren, entfernen und konfigurieren Sie ARGVUS.
---
ARGVUS wird als Arch-Linux-Paket über das offizielle ARGVUS-Paket-Repository vertrieben.

## Vor der Installation

Aktivieren Sie das offizielle Arch-Linux-Repository `multilib`, bevor Sie ARGVUS installieren. Bearbeiten Sie `/etc/pacman.conf` und heben Sie die Auskommentierung sowohl des `[multilib]`-Abschnitts als auch seiner `Include`-Zeile auf:

```ini
[multilib]
Include = /etc/pacman.d/mirrorlist
```
Synchronisieren Sie dann die Paketdatenbanken und führen Sie ein vollständiges System-Upgrade durch:

```sh
sudo pacman -Syu
```
Der Schnellinstallateur konfiguriert das ARGVUS-Repository, aktiviert jedoch `multilib` nicht für Sie.

## Schnellinstallation

Auf einer frischen Arch-Linux-Installation können Sie ARGVUS direkt von einem TTY aus installieren, ohne zuerst eine andere Desktop-Umgebung zu installieren.

Führen Sie aus:

```sh
curl -fsSL https://argvus.github.io/install.sh | sh
```
Der Installer:

1. Importiert und signiert lokal den ARGVUS-Repository-Signierschlüssel.
2. Konfiguriert das ARGVUS-Paket-Repository.
3. Aktualisiert die Paketdatenbank.
4. Installiert das Paket `argvus` und seine Abhängigkeiten.

Nach der Installation, melden Sie sich ab und wählen Sie **ARGVUS** in Ihrem Display-Manager.

> **Hinweis:** Der Schnellinstaller benötigt `curl`, `sudo` und die Standard-Paketverwaltungswerkzeuge von Arch Linux.

## Nur das Repository konfigurieren

Um das ARGVUS-Paket-Repository zu konfigurieren, ohne den Desktop zu installieren, führen Sie aus:

```sh
curl -fsSL https://argvus.github.io/repo-install.sh | sh
```
Installieren Sie ARGVUS anschließend manuell:

```sh
sudo pacman -Syu argvus
```
## Manuelle Installation

Wenn Sie das Repository Schritt für Schritt konfigurieren möchten, verwenden Sie die folgenden Befehle.

### 1. Laden Sie den Signaturschlüssel des Repositorys herunter

```sh
curl -fsSLo /tmp/argvus.gpg https://argvus.github.io/packages/arch/argvus.gpg
```
### 2. Den Signierschlüssel importieren

```sh
sudo pacman-key --add /tmp/argvus.gpg
```
### 3. Signiere den Schlüssel lokal

```sh
ARGVUS_KEY="$(gpg --show-keys --with-colons /tmp/argvus.gpg | grep '^pub:' | head -n1 | cut -d: -f5)"
sudo pacman-key --lsign-key "$ARGVUS_KEY"
```
### 4. Konfigurieren Sie das ARGVUS-Repository

```sh
curl -fsSL https://argvus.github.io/packages/arch/argvus.conf \
  | sudo tee /etc/pacman.d/argvus.conf
```
Fügen Sie die Repository-Konfiguration zu `/etc/pacman.conf` hinzu:

```sh
echo "Include = /etc/pacman.d/argvus.conf" \
  | sudo tee -a /etc/pacman.conf
```
Die Repository-Konfiguration aktiviert die ARGVUS-Repositories, einschließlich `argvus` und `argvus-extras`, zusammen mit `SigLevel = Required TrustedOnly`. Pacman akzeptiert Pakete nur, wenn deren Signaturen gültig sind und vom lokalen Schlüsselbund vertraut werden.

### 5. Installieren Sie ARGVUS

```sh
sudo pacman -Syu argvus
```
Nach der Installation melden Sie sich ab und wählen **ARGVUS** in Ihrem Display-Manager aus.

## Aktualisierung

Aktualisieren Sie ARGVUS zusammen mit dem Rest von Arch Linux:

```sh
sudo pacman -Syu
```
Um nur das ARGVUS-Paket zu aktualisieren:

```sh
sudo pacman -S argvus
```

> **Empfehlung:** Arch Linux empfiehlt im Allgemeinen ein vollständiges Systemupgrade mit `sudo pacman -Syu` anstelle des Aktualisierens einzelner Pakete.

## ARGVUS entfernen

Entfernen Sie ARGVUS, während Abhängigkeiten beibehalten werden, die noch von anderen installierten Paketen benötigt werden:

```sh
sudo pacman -R argvus
```
Entfernen Sie ARGVUS und Abhängigkeiten, die von keinem installierten Paket mehr benötigt werden:

```sh
sudo pacman -Rns argvus
```
### Entfernen Sie das ARGVUS-Repository

Wenn Sie auch das Paket-Repository entfernen möchten, entfernen Sie dessen Konfiguration:

```sh
sudo rm /etc/pacman.d/argvus.conf
```
Dann entfernen Sie die entsprechende Zeile aus `/etc/pacman.conf`:

```text
Include = /etc/pacman.d/argvus.conf
```
Sie können die Datei bearbeiten mit:

```sh
sudo nano /etc/pacman.conf
```
### Entfernen des Signaturschlüssels

Um den Signaturschlüssel des ARGVUS-Repositorys aus dem Pacman-Schlüsselbund zu entfernen, listen Sie zunächst die Schlüssel auf:

```sh
sudo pacman-key --list
```
Entfernen Sie dann den ARGVUS-Schlüssel mit seiner Schlüssel-ID:

```sh
sudo pacman-key --delete KEY_ID
```
Ersetzen Sie `KEY_ID` durch die ARGVUS-Signaturschlüssel-ID.

> **Warnung:** Entfernen Sie diesen Schlüssel nur, wenn Sie auch das ARGVUS-Repository entfernen. Entfernen Sie keine Schlüssel, die von anderen Repositories noch benötigt werden.

## Benutzerkonfiguration

Die Paketinstallation schreibt keine Konfiguration in `$HOME`. ARGVUS liest voreingestellte Pakete aus `/usr/share/argvus` und funktioniert, ohne sie in Ihr Home-Verzeichnis zu kopieren.

Die Laufzeitkonfiguration folgt dieser allgemeinen Priorität:

```text
~/.config/argvus/data/generated/<app>  ->  ~/.config/argvus/data/<component>/<app>  ->  $XDG_CONFIG_HOME/<app>  ->  /usr/share/argvus/<app>  ->  upstream defaults
```
`argvus-config` ist die einzige Komponente, die `~/.config/argvus/data/generated/` schreibt, und diese Projektion ist die aktive Schicht: Eine verwaltete oder native Kopie tritt nur in Kraft, solange die entsprechende generierte Datei fehlt.

Die von ARGVUS verwaltete Konfiguration befindet sich unter `~/.config/argvus/`. Eine optionale `$XDG_CONFIG_HOME/<app>`-Überschreibung bleibt für Anwendungen verfügbar, die eine native Benutzerkonfiguration unterstützen.

Der `argvus --setup`-Dispatcher ist optional und dient für explizite manuelle Überschreibungen, nicht für die Wiederherstellung. Wenn `~/.config/argvus` gelöscht wird, erstellt `SUPER + SHIFT + R` das kanonische Profil und den generierten Laufzeitzustand über die normale Reload-Pipeline neu.

```sh
argvus --setup --copy foot-tui
argvus --setup --copy kitty-tui
argvus --setup --copy-all
```
`--copy-all` ist eine explizite Positivliste der unterstützten TUI-Profil-Dateien; es klont nicht `/usr/share/argvus` und kopiert keine Skripte, Dienste, Assets, generierten Dateien oder `config.json`. Führen Sie `argvus --setup --help` aus, um die aktuellen Optionen zu sehen. Sobald die Dateien kopiert wurden, werden sie zu benutzerdefinierten Überschreibungen und Paket-Upgrades ersetzen sie nicht.

Um eine vorhandene kopierte Konfiguration zu ersetzen, verwenden Sie `--force`. ARGVUS erstellt eine mit Zeitstempel versehene Sicherung, bevor es sie ersetzt:

```sh
argvus --setup --copy <app> --force
```
## Hyprland-Benutzerüberschreibungen

Für die vollständige Hyprland-Überschreibungsanleitung, einschließlich des Unterschieds zwischen einer vollständigen nativen Konfiguration und inkrementellen ARGVUS-Fragmente, siehe [Hyprland overrides](/docs/argvus-hyprland/hyprland-overrides/).

```sh
mkdir -p ~/.config/argvus/data/hypr
nano ~/.config/argvus/data/hypr/user.lua
```
### Virtuelle Maschinen-Ausweichlösung

Einige virtuelle Maschinen können Darstellungsprobleme mit Hyprland, Kitty oder Quickshell haben. ARGVUS erkennt virtuelle Maschinen, die den Grafiktreiber `vmwgfx` verwenden, und aktiviert bei Bedarf das Software-Rendering.

Wenn die Erkennung auf Ihrer virtuellen Maschine nicht funktioniert, aktivieren Sie die Ausweichlösung manuell in `user.lua`:

```lua
-- Virtual machine compatibility
hl.env("LIBGL_ALWAYS_SOFTWARE", "1")
```
Nachdem Sie `user.lua` erstellt oder geändert haben, melden Sie sich von Hyprland ab und wieder an, damit die Variable auf Anwendungen angewendet wird, die von der Sitzung gestartet werden.

Überprüfen Sie es mit:

```sh
echo $LIBGL_ALWAYS_SOFTWARE
```
Die erwartete Ausgabe ist:

```text
1
```

> **Hinweis:** `LIBGL_ALWAYS_SOFTWARE=1` zwingt Mesa/OpenGL-Anwendungen dazu, die Softwaredarstellung zu verwenden, normalerweise über `llvmpipe`. Dies kann die grafische Leistung verringern und sollte als Notlösung für betroffene virtuelle Maschinen verwendet werden.
