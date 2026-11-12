---
title: Themes installieren
description: Installiere die offiziellen ARGVUS-Themenpakete mit pacman.
---
Die offiziellen Themen werden als Arch-Linux-Pakete veröffentlicht. Jedes Thema ist ein eigenes Paket mit dem Namen `argvus-theme-<THEME_NAME>`.

## Wo man Themen bekommt

Die Quelle jedes offiziellen Themas befindet sich in der ARGVUS-Projektorganisation:

- Jedes Thema ist ein Repository namens `argvus-theme-<THEME_NAME>` (zum Beispiel `argvus-theme-dracula`), das als Paket mit demselben Namen erstellt wird.

Themenpakete hängen von `argvus-appearance` ab. Installieren Sie zuerst ARGVUS, wie in [Installation](/docs/getting-started/installation/) beschrieben, damit die Erscheinungskomponente vorhanden ist.

## Ein einzelnes Thema installieren

Um alle offiziellen Themen auf einmal zu installieren, installieren Sie das `argvus-themes`-Metapaket:

```bash
sudo pacman -S argvus-themes
```
Es hängt von allen veröffentlichten `argvus-theme-*`-Paketen ab. ARGVUS Dark und ARGVUS Light werden bereits von `argvus-appearance` geliefert.

Installieren Sie ein Thema mit seinem Paketnamen:

```bash
sudo pacman -S argvus-theme-<THEME_NAME>
```
Zum Beispiel:

```bash
sudo pacman -S argvus-theme-dracula
sudo pacman -S argvus-theme-rose-pine
```
Das Paket installiert das Themenmanifest `/usr/share/argvus/themes/<theme-id>/theme.toml` (dessen Name, Kategorie, Hintergrundbild und Farbpalette, mit optionalem `overrides/`) und das Hintergrundbild des Themas unter `/usr/share/backgrounds/argvus/`. Die Dateien jeder Komponente werden aus der Farbpalette gerendert, wenn das Thema angewendet wird. Es schreibt nicht in Ihr Home-Verzeichnis und ändert nicht Ihr aktuelles Thema. Wählen Sie es anschließend im Kontrollzentrum aus, siehe [Offizielle Themen](/docs/components/argvus-themes/official-themes/).

## Mehrere Themen installieren

Es gibt kein Meta-Paket, das jedes Thema installiert. Installieren Sie die gewünschten Themen mit einem Befehl:

```bash
sudo pacman -S argvus-theme-dracula argvus-theme-sunset
```
## Verfügbare Pakete

Die untenstehenden Pakete sind die offiziellen Themenpakete, die heute veröffentlicht wurden.

**Dunkle Themen**

| Paket | Thema |
| --- | --- |
| `argvus-theme-one-dark` | Ein Dunkler |
| `argvus-theme-dracula` | Dracula |
| `argvus-theme-silver-dark` | Silber dunkel |
| `argvus-theme-slate-dark` | Schiefer dunkel |
| `argvus-theme-universe` | Universum |
| `argvus-theme-gruvbox-high-dark` | Gruvbox High Dark |
| `argvus-theme-gruvbox-dark` | Gruvbox Dark |
| `argvus-theme-rose-pine` | Rosé-Kiefer |
| `argvus-theme-tokyo-night` | Tokio-Nacht |
| `argvus-theme-solitude` | Einsamkeit |
| `argvus-theme-sunset` | Sonnenuntergang |
| `argvus-theme-matrix` | Matrix |
| `argvus-theme-monokai-dark` | Monokai Dark |

**Helle Themen**

| Paket | Thema |
| --- | --- |
| `argvus-theme-github-light` | GitHub Licht |
| `argvus-theme-solarized-light` | Solarisiertes Licht |
| `argvus-theme-nord-light` | Nordlicht |
| `argvus-theme-one-light` | Ein Licht |
| `argvus-theme-everforest-light` | Everforest Licht |
| `argvus-theme-catppuccin-latte` | Catppuccin Latte |
| `argvus-theme-gruvbox-light` | Gruvbox Licht |
| `argvus-theme-watercolor` | Aquarell |

Die eingebauten **ARGVUS Dark** und **ARGVUS Light** Themen benötigen kein separates Paket; sie werden mit `argvus-appearance` geliefert.

## Überprüfen, was installiert ist

Liste die installierten Themenpakete mit:

```bash
pacman -Qs argvus-theme
```
Öffnen Sie das Kontrollzentrum und gehen Sie zu **Darstellung → Designs → Offizielle Designs**, um die derzeit im Selektor verfügbaren Designs zu sehen. Ein Design, das installiert ist, aber in dieser Liste fehlt, ist ein Verpackungsproblem; siehe den Abschnitt zur Fehlerbehebung im [Themes-Leitfaden](/docs/components/argvus-themes/).

## Ein eigenes Design hinzufügen

Ein Design ist ein Manifest, daher benötigt ein eigenes Design kein Paket. Erstellen Sie
`~/.local/share/argvus/themes/<id>/theme.toml`, wobei der Verzeichnisname die Design-ID ist (kleine Buchstaben, Ziffern und einfache Bindestriche):

```toml
schema = 2
id = "mist"
name = "Mist"
category = "dark"
wallpaper = "/usr/share/backgrounds/argvus/argvus-dark.jxl"

[palette]
bg = "#101418"
fg = "#D8DEE9"
accent = "#88C0D0"
```
Nur `bg`, `fg` und `accent` sind erforderlich: Jede andere Farbe wird von ihnen abgeleitet, und das Manifest definiert nur eine, um sie zu ändern. Überprüfen Sie sie, und wenden Sie sie dann wie jedes andere Thema an:

```bash
argvus theme check ~/.local/share/argvus/themes/mist
argvus theme apply mist
```
`argvus theme list` zeigt Ihre Designs neben den offiziellen an (ein offizielles Design mit derselben ID hat Vorrang). Ihre Designs erscheinen im Designmenü (`SUPER + Shift + T`); der Pre-Login-Begrüßer kann Ihr Home-Verzeichnis nicht lesen, daher behält er das zuletzt angewendete offizielle Design bei, während eines Ihrer Designs aktiv ist.

Die pro-Design-Kopien, die ältere ARGVUS-Versionen unter `~/.config/argvus/data/<app>/themes/` geschrieben haben, werden nicht mehr gelesen. Der erste Designwechsel nach dem Update verschiebt sie nach `~/.config/argvus/data/backups/theme-copies-<time>/`; um eine von Ihnen bearbeitete Datei zu behalten, legen Sie sie in ein eigenes Design unter `overrides/` und geben Sie ihr einen Namen in der `[overrides]`-Tabelle des Manifests.

## Ein Design entfernen

Entfernen Sie ein einzelnes Designpaket mit:

```bash
sudo pacman -R argvus-theme-<THEME_NAME>
```
Das Entfernen eines Themenpakets löscht Ihre importierten Profile nicht. Wenn das entfernte Thema aktuell angewendet wird, wählen Sie ein anderes Thema aus dem Kontrollzentrum, bevor Sie es entfernen.
