---
title: Installing themes
description: Install the official ARGVUS theme packages with pacman.
---

The official themes are published as Arch Linux packages. Each theme is its own package named `argvus-theme-<THEME_NAME>`.

## Where to get themes

The source of every official theme lives in the ARGVUS project organization:

- Each theme is a repository named `argvus-theme-<THEME_NAME>` (for example `argvus-theme-dracula`), built as a package with the same name.

Theme packages depend on `argvus-appearance`. Install ARGVUS first, as described in [Installation](/docs/getting-started/installation/), so that the appearance component is present.

## Install a single theme

Install one theme with its package name:

```bash
sudo pacman -S argvus-theme-<THEME_NAME>
```

For example:

```bash
sudo pacman -S argvus-theme-dracula
sudo pacman -S argvus-theme-rose-pine
```

The package installs the theme payload under `/usr/share/argvus/appearance/themes.d/<theme-id>/` and the theme's wallpaper under `/usr/share/backgrounds/argvus/`. It does not write into your home directory, and it does not change your current theme. Select it from the Control Center afterwards, see [Official themes](/docs/components/argvus-themes/official-themes/).

## Install several themes

There is no meta-package that installs every theme. Install the themes you want in one command:

```bash
sudo pacman -S argvus-theme-dracula argvus-theme-sunset
```
## Available packages

The packages below are the official theme packages published today.

**Dark themes**

| Package | Theme |
| --- | --- |
| `argvus-theme-one-dark` | One Dark |
| `argvus-theme-dracula` | Dracula |
| `argvus-theme-silver-dark` | Silver Dark |
| `argvus-theme-slate-dark` | Slate Dark |
| `argvus-theme-universe` | Universe |
| `argvus-theme-gruvbox-high-dark` | Gruvbox High Dark |
| `argvus-theme-gruvbox-dark` | Gruvbox Dark |
| `argvus-theme-rose-pine` | Rosé Pine |
| `argvus-theme-tokyo-night` | Tokyo Night |
| `argvus-theme-solitude` | Solitude |
| `argvus-theme-sunset` | Sunset |
| `argvus-theme-matrix` | Matrix |
| `argvus-theme-monokai-dark` | Monokai Dark |

**Light themes**

| Package | Theme |
| --- | --- |
| `argvus-theme-github-light` | GitHub Light |
| `argvus-theme-solarized-light` | Solarized Light |
| `argvus-theme-nord-light` | Nord Light |
| `argvus-theme-one-light` | One Light |
| `argvus-theme-everforest-light` | Everforest Light |
| `argvus-theme-catppuccin-latte` | Catppuccin Latte |
| `argvus-theme-gruvbox-light` | Gruvbox Light |

The built-in **ARGVUS Dark** and **ARGVUS Light** themes do not need a separate package; they come with `argvus-appearance`.

## Check what is installed

List the installed theme packages with:

```bash
pacman -Qs argvus-theme
```

Open the Control Center and go to **Appearance → Themes → Official Themes** to see the themes that are currently available in the selector. A theme that is installed but missing from that list is a packaging problem; see the troubleshooting section of the [Themes guide](/docs/components/argvus-themes/).

## Remove a theme

Remove a single theme package with:

```bash
sudo pacman -R argvus-theme-<THEME_NAME>
```

Removing a theme package does not delete your imported profiles. If the removed theme is the one currently applied, select another theme from the Control Center before removing it.
