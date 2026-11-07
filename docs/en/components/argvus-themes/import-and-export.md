---
title: Importing and exporting themes
description: Export the active theme as a theme of your own, import a theme archive, apply it and delete it.
---

Export and import work with **themes**, in the same format as the official themes. Exporting saves the active theme, with your highlight color and wallpaper, as a new theme of your own. Importing installs a theme archive among your themes. Both are in **Control Center → Appearance → Themes**, and your themes are listed in **Custom Themes**.

## What an exported theme contains

The archive, `argvus-theme-<id>.tar.gz`, holds one theme directory, laid out like an official theme:

```text
<id>/theme.toml        the manifest (schema 2): name, category, palette, wallpaper
<id>/appearance.json  your visual settings, including layout, effects, fonts and panel presentation
<id>/effects.conf     per-surface effect values for the active theme, when present
<id>/overrides/...     the overrides of the theme it was made from, if any
<id>/wallpaper.<ext>   the wallpaper on screen when you exported
```

The manifest is based on the active theme, with your name, a new id, your highlight color as the palette accent and the wallpaper beside it. The appearance profile preserves the visual settings currently applied in ARGVUS, including window spacing, borders, taskbar margins, panel presentation, colors, effects and fonts. Custom wallpaper and accent color are stored with the theme. Applying any theme reapplies its complete profile (or the ARGVUS defaults when no profile exists), replacing independent visual changes made since it was last applied. Switching Sticky or Float always reapplies that mode's window and taskbar geometry; save personal geometry in an exported custom theme to keep it available when that theme is applied. Non-visual settings such as audio, keyboard and network preferences are not part of a theme.

The applied appearance persists across session restarts and reloads. Session preparation rebuilds the necessary files from the current configuration without restoring the original theme's geometry.

Settings changed after applying a theme are temporary across the next theme or mode change. Export the theme from **Control Center → Appearance → Themes → Export theme** to preserve the current visual configuration and restore it later by applying that exported theme.

Your themes are installed in `~/.local/share/argvus/themes/<id>/`. Extracting an archive there installs the theme too.

## Export the active theme

1. Open **Control Center → Appearance → Themes**.
2. Choose **Export theme**, or press `e` on a Themes page.
3. Type a name for the theme. The field shows the file that will be written.
4. Press `Enter`.

The id is made from the name: lowercase letters, digits and hyphens. When the id is already used by an installed theme or by an archive in your home directory, `-2`, `-3` and so on are added. The archive is written to your home directory, for example `~/argvus-theme-my-theme.tar.gz`. ARGVUS checks the theme before writing it: the manifest must load and every component must render in both modes.

## Import a theme

1. Copy the `.tar.gz` archive into your home directory (`~`). ARGVUS lists the archives there that hold a theme.
2. Open **Control Center → Appearance → Themes → Import theme**, or press `i` on a Themes page.
3. Select the archive and press `Enter`.

ARGVUS checks the archive before installing it. It must hold a single directory with a `theme.toml`, with relative paths only (no `..`, no links), at most 128 entries and a size limit per file; the id must match the directory, the wallpaper must be in the archive and every component must render in both modes. An included appearance profile must use supported sections and valid JSON. If any check fails, nothing is installed.

The theme is added to **Custom Themes**. Importing does not apply it: you choose when.

### Name collisions

- If one of your themes has the same id, ARGVUS asks **Replace your theme with the same ID?**. Choose **Replace** to replace it, or press `Esc` to keep yours. The replaced theme is kept as a backup until the new one is installed.
- An archive with the id of an official theme is refused. Export it again under another name.

## Apply a custom theme

1. Open **Control Center → Appearance → Themes → Custom Themes**.
2. Select the theme and press `Enter`.

The theme's complete exported configuration is applied, including its wallpaper, spacing, borders, accent color, effects, fonts and panel presentation. Its visual values replace the current settings. ARGVUS then restarts the taskbar, Hyprland reload service, wallpaper and Control Panel so each surface uses the custom theme.

## Delete a custom theme

1. Open **Control Center → Appearance → Themes → Custom Themes**.
2. Select the theme and press `d`.
3. Confirm with `y`, or press `n` or `Esc` to cancel.

The theme directory is removed from `~/.local/share/argvus/themes`. If it was the active theme, ARGVUS switches to **ARGVUS Dark** first.

## Profiles of earlier versions

Earlier versions exported *theme profiles* (`profile.tar.gz`), a copy of the appearance settings. The first time a Themes page opens, each profile in `~/.local/share/argvus/themes/<name>/` is converted into a theme made from its base theme, with its highlight color and wallpaper. The original profile, and the old `custom-themes.json` list, move to `~/.config/argvus/data/backups/theme-profiles-<seconds>/`, and the status line reports the conversion. A profile whose base theme is not installed is only moved to the backup.

## Share a theme

An exported theme is a regular archive. Copy it to another machine that runs ARGVUS and import it there.

## Troubleshooting

- **The archive is not listed.** It must be directly inside your home directory, end with `.tar.gz` and hold `<id>/theme.toml`.
- **The import is refused.** The status line says why: an official theme with the same id, a missing wallpaper, a manifest that does not load or a component that does not render.
- **The export is refused.** The name needs at least one letter or digit, and the active theme must be installed.
