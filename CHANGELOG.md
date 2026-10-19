# Changelog

All notable changes to `argvus` are documented here.

## Unreleased

- A failed theme switch now restarts the services it stopped and reports the error; an invalid theme id is refused before anything changes.
- `argvus config accent` persists the chosen color as the custom accent in the same command; the Control Center no longer does a separate patch step.
- Light/Dark toggle picks the opposite mode from the theme's manifest, so light themes such as Solarized Light now switch too.
- Appearance > Themes lists Nord Light.
- Sticky/Float regenerates the per-component files of the current theme for the new variant (taskbar, launcher, notifications, lock, terminal, telemetry, control panel, TUI, btop).
- The session start reads the theme from the canonical configuration and no longer resets a custom wallpaper at login.
- The terminal uses the accent the user stored, not the packaged default, when a custom accent exists.
- Configuration files are created with private modes (0600, directories 0700); two backup generations are kept.
- The keep-awake marker is written to the name every reader uses (`data/power/.keep-awake`).

- The desktop Rust projects are consolidated into the `argvus` workspace (`crates/`). The former repositories `argvus-config`, `argvus-i18n`, `argvus-keys`, `argvus-projects`, `argvus-terminal`, `argvus-system-monitor`, `argvus-network`, `argvus-widget-telemetry` and `argvus-tui-terminal` are now in-process subcommands (`argvus config`, `argvus i18n`, `argvus keys`, `argvus projects`, `argvus terminal`, `argvus system-monitor`, `argvus network`, `argvus widget-telemetry`, `argvus tui-terminal`).
- `argvus` is the master package. One PKGBUILD produces the split packages `argvus`, `argvus-gtk` (helpers) and `argvus-greeter`. Every component package depends on `argvus`, so no install order is required.
- The greeter source moved to `greeter/`, as its own Cargo workspace outside the root workspace.
- `argvus-game-snake` is an independent, optional project; it is no longer part of `argvus`.
- Removed `argvus-pwvucontrol`, `argvus-snappy-switcher` and the Alt+Tab binding.
