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

The package installs the theme manifest, `/usr/share/argvus/themes/<theme-id>/theme.toml` (its name, category, wallpaper and palette, with optional `overrides/`), and the theme's wallpaper under `/usr/share/backgrounds/argvus/`. The files of every component are rendered from the palette when the theme is applied. It does not write into your home directory, and it does not change your current theme. Select it from the Control Center afterwards, see [Official themes](/docs/components/argvus-themes/official-themes/).

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
| `argvus-theme-watercolor` | Watercolor |

The built-in **ARGVUS Dark** and **ARGVUS Light** themes do not need a separate package; they come with `argvus-appearance`.

## Check what is installed

List the installed theme packages with:

```bash
pacman -Qs argvus-theme
```

Open the Control Center and go to **Appearance → Themes → Official Themes** to see the themes that are currently available in the selector. A theme that is installed but missing from that list is a packaging problem; see the troubleshooting section of the [Themes guide](/docs/components/argvus-themes/).

## Add a theme of your own

A theme is a manifest, so a theme of your own needs no package. Create
`~/.local/share/argvus/themes/<id>/theme.toml`, where the directory name is the
theme ID (lowercase letters, digits and single hyphens):

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

Only `bg`, `fg` and `accent` are required: every other colour has a default
derived from them, and the manifest defines one only to change it. Check it,
then apply it like any other theme:

```bash
argvus theme check ~/.local/share/argvus/themes/mist
argvus theme apply mist
```

`argvus theme list` shows your themes next to the official ones (an official
theme with the same ID wins). Your themes appear in the theme menu
(`SUPER + Shift + T`); the pre-login greeter cannot read your home directory, so
it keeps the last official theme you applied while one of your themes is active.

The per-theme copies that older ARGVUS versions wrote under
`~/.config/argvus/data/<app>/themes/` are no longer read. The first theme switch
after the update moves them to `~/.config/argvus/data/backups/theme-copies-<time>/`;
to keep a file you edited, put it in a theme of your own under `overrides/` and
name it in the manifest's `[overrides]` table.

## Remove a theme

Remove a single theme package with:

```bash
sudo pacman -R argvus-theme-<THEME_NAME>
```

Removing a theme package does not delete your imported profiles. If the removed theme is the one currently applied, select another theme from the Control Center before removing it.
