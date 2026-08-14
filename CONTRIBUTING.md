# Contributing

Thank you for contributing to Argvus Environment.

## Scope

Good contributions improve the desktop experience as a whole: session startup, Hyprland behavior, Waybar modules, Quickshell controls, themes, scripts, packaging compatibility or documentation.

## Guidelines

- Keep defaults predictable and suitable for a ready-to-use Hyprland desktop.
- Preserve user configuration. Do not introduce package-time writes to `$HOME`.
- Keep shared theme behavior consistent across Hyprland, Waybar, Rofi, Dunst, Kitty, Wlogout and terminal tools.
- Prefer small, focused changes with clear motivation.
- Test scripts with `bash -n` before opening a pull request.
- Test UI-facing changes in a real session when possible.

## Pull requests

Include:

- what changed;
- why it changed;
- how it was tested;
- screenshots or short recordings for visible UI changes.

## Documentation

Technical user documentation belongs in `site-src`, not in this README. When a change affects installation, configuration, official apps, themes or sessions, update the documentation site in the same work.
