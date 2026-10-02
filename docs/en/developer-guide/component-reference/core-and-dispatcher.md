---
title: Core and dispatcher
description: Core ARGVUS infrastructure projects.
---

`argvus` provides the `/usr/bin/argvus` dispatcher and package entry point. `argvus-config` provides the canonical configuration store and is the only writer of the generated tree. `argvus-session` provides session entrypoints, lifecycle units and `argvus-sessionctl`. `argvus-tui` provides shared Rust TUI libraries. `argvus-i18n` provides catalogs, validation and the `org.argvus.i18n` QML module.
