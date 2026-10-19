---
title: Importing and exporting themes
description: Save your current appearance as a theme profile, import a profile, apply it and delete it.
---

A **theme profile** is a `.tar.gz` archive that stores your current appearance so you can restore it later, move it to another ARGVUS installation or share it. Profiles are managed in **Control Center → Appearance → Themes → Export / Import**.

## What a profile contains

A profile records:

- the active theme and its mode (Sticky or Float);
- the highlight color, when you chose a custom one;
- layout, effects, transparency and blur values, and the fonts and Control Panel card selection;
- the wallpaper, when the current wallpaper is a readable file.

Generated runtime files, such as the Hyprland, Waybar and shell configuration, are not part of the archive. ARGVUS recreates them when you apply the profile.

## Export the current appearance

1. Open **Control Center → Appearance → Themes**.
2. Choose **Export / Import**, then **Export theme profile**. You can also press `e` on the Themes page.
3. Type a name for the theme. The name is shown in the list of custom themes.
4. Press `Enter` to export.

The archive is written to your home directory, named after the theme name and the export date and time, for example `my-theme-20260103-101500.tar.gz`. If a file with the same name already exists, a numeric suffix is added.

## Import a profile

1. Copy the `.tar.gz` profile into your home directory (`~`). ARGVUS looks for profiles there.
2. Open **Control Center → Appearance → Themes → Export / Import**, then choose **Import theme profile**. You can also press `i` on the Themes page.
3. Select the archive from the list and press `Enter`.

ARGVUS validates the archive before installing it. Validation checks the manifest, the checksums and the wallpaper payload. If the archive is invalid, the import is cancelled and nothing is installed.

The imported profile is added to **Custom Themes**. Importing and applying are separate steps: importing only adds the profile, and you choose when to apply it.

### Name collisions

If a custom theme with the same identity already exists, ARGVUS asks **Replace the existing imported theme?**. Choose **Replace** to overwrite it, or press `Esc` to keep the existing one.

### Wallpapers

The embedded wallpaper is restored under your home directory when possible:

- ARGVUS uses the original path when it is available.
- Otherwise it uses your pictures directory, or `~/Pictures/ARGVUS`.
- If a file with that name already exists, the restored file receives an `-imported-N` suffix.

When a profile wallpaper cannot be restored at all, the import fails instead of creating a profile that points to a missing file.

## Apply a custom theme

1. Open **Control Center → Appearance → Themes → Custom Themes**.
2. Select the imported theme and press `Enter`.

Applying restores the theme state, the accent, effects, layout and the included wallpaper. Selecting a profile reloads the affected desktop surfaces. If the profile wallpaper cannot be restored as a file, the profile is applied with the official wallpaper and you see a notice.

## Delete an imported theme

1. Open **Control Center → Appearance → Themes → Custom Themes**.
2. Select the theme and press `d`.
3. Confirm with `Enter`, or press `Esc` to cancel.

Deleting an imported theme removes the profile from ARGVUS. Its wallpaper is kept. If the deleted profile was the active one, ARGVUS falls back to **ARGVUS Dark**.

## Share a profile

A profile is a regular archive. Copy it to another machine that runs ARGVUS and import it there. The profile does not include the transparency and blur values saved for inactive themes.

## Troubleshooting

- **The archive is listed but import fails.** The profile is invalid or its wallpaper payload is unreadable. Export the appearance again from the source machine.
- **The imported theme shows the official wallpaper.** The saved wallpaper could not be restored as a file. Choose a wallpaper again under **Control Center → Appearance → Wallpapers**.
- **The profile does not appear in the list.** Check that the archive is directly inside your home directory and ends with `.tar.gz`.
