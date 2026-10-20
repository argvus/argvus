---
title: Core and dispatcher
description: The argvus workspace, its dispatcher and the shared Rust crates.
---

`argvus` is the Cargo workspace at `de/argvus` and the package that provides `/usr/bin/argvus`. The crate `argvus-cli` is the dispatcher: it reads the first argument and either runs a library command in-process, runs a GTK 4 helper or a private executable, or runs a packaged script. The full subcommand list is in the [command line reference](/docs/reference/command-line/).

The workspace also holds the crates that the commands share:

- `argvus-config`: the canonical configuration store and the only writer of the generated tree (`data/generated/`). It is also a library used in-process by `argvus config`, `theme-switch` and `effects-toggle`.
- `argvus-i18n` and `argvus-i18n-core`: the catalogs in `resources/locales/`, their validation, and the `org.argvus.i18n` QML module.
- `argvus-tui` and `argvus-theme`: shared terminal UI and theme loading.
- `argvus-runtime`: the translated messages and the process helpers shared by the commands.

`argvus-session` stays a separate repository. It provides the session entry points (`session`, `start`, `tty`) and the lifecycle units, and `argvus sessionctl` reaches its controller.
