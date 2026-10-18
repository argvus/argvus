# Changelog

All notable changes to `argvus` are documented here.

## Unreleased

- The desktop Rust projects are consolidated into the `argvus` workspace (`crates/`). The former repositories `argvus-config`, `argvus-i18n`, `argvus-keys`, `argvus-projects`, `argvus-terminal`, `argvus-system-monitor`, `argvus-network`, `argvus-widget-telemetry` and `argvus-tui-terminal` are now in-process subcommands (`argvus config`, `argvus i18n`, `argvus keys`, `argvus projects`, `argvus terminal`, `argvus system-monitor`, `argvus network`, `argvus widget-telemetry`, `argvus tui-terminal`).
- `argvus` is the master package. One PKGBUILD produces the split packages `argvus`, `argvus-gtk` (helpers) and `argvus-greeter`. Every component package depends on `argvus`, so no install order is required.
- The greeter source moved to `greeter/`, as its own Cargo workspace outside the root workspace.
- `argvus-game-snake` is an independent, optional project; it is no longer part of `argvus`.
- Removed `argvus-pwvucontrol`, `argvus-snappy-switcher` and the Alt+Tab binding.
