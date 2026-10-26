---
title: Customizing the highlight color
description: Change the ARGVUS highlight color independently of the selected theme, and return to the theme default.
---

The **highlight color** is the accent used for focus borders, selection and active states across the desktop. Every theme has a default highlight color, but you can replace it with any color you want without changing the theme.

## Change the highlight color

1. Open **Control Center → Appearance → Highlight color**.
2. Choose **Edit highlight color**. The current value is shown next to the option.
3. Type a six-digit RGB color, with or without the leading `#`. For example `3590BD` and `#3590BD` are both accepted.
4. Press `Enter` to apply.

You can also paste a color into the field. ARGVUS accepts the pasted text only when it is a valid six-digit RGB value. Short forms such as `#123` and values with non-hexadecimal characters are rejected with an error message, and the color is not changed.

The new color is applied to the desktop immediately and is saved in your ARGVUS configuration.

## Return to the theme default

To go back to the color defined by the selected theme, open **Control Center → Appearance → Highlight color** and choose **Reset to theme default**.

## How the highlight color interacts with themes

- A highlight color you choose is an independent override. It survives when you switch to another theme, so the color you picked stays the same.
- A theme that you apply after **Reset to theme default** takes over the highlight color again and uses its own default.
- Exporting the active theme stores the highlight color that is active when you export as the accent of the new theme's palette. See [Importing and exporting themes](/docs/components/argvus-themes/import-and-export/).

## Where the value is stored

The highlight color is part of your ARGVUS configuration, in `$XDG_CONFIG_HOME/argvus/config.json` (usually `~/.config/argvus/config.json`). Do not edit the generated files under `$XDG_CONFIG_HOME/argvus/data/generated`: ARGVUS rewrites them from the configuration.
