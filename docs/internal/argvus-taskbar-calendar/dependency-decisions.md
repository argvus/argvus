# Dependency Decisions

Validated on 2026-08-11 with `cargo search` / `cargo info` in the local environment.

- `relm4 0.11.0`: current maintained Relm4 release, Rust 1.93+, compatible with this environment's Rust 1.94.
- `gtk4 0.11.4`: current gtk-rs GTK4 binding generation and compatible with `gtk4-layer-shell`.
- `gtk4-layer-shell 0.8.1`: current safe binding for layer-shell behavior under wlroots compositors.
- `rusqlite 0.40.2`: lightweight SQLite wrapper; chosen over a heavy ORM.
- `icalendar 0.17.13`: maintained builder/parser with recurrence support; used for RFC 5545 import/export.
- `reqwest 0.13.4` + `quick-xml 0.41.0`: used for a small internal CalDAV/WebDAV client instead of adopting an immature CalDAV crate.
- `secret-service 5.1.0`: Secret Service API support with pure Rust crypto and Tokio; selected over `oo7` because current `oo7` requires Rust 1.95.
- Secret Service provider: `gnome-keyring` is declared as an `optdepends` (not `depends`) because the calendar only needs a provider when a CalDAV password is stored. ARGVUS sessions start it through `argvus-secret-service.service`; without a provider, saving a CalDAV password fails with a `secret service unavailable` error.
- `notify-rust 4.18.0`: standard Linux desktop notifications.

CalDAV 0.1 uses a conservative internal client for `PROPFIND`, `REPORT`, `PUT`, and `DELETE`. Ambiguous ETag conflicts are preserved instead of blindly overwriting remote changes.
