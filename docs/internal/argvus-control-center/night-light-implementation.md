# Night Light implementation checkpoint

Date: 2026-10-29.

## Implemented phases

- Phase 0: Hyprland 0.56.2 / Hyprsunset 0.4.0-3 tested in the current session. Official temperature, identity, identity get and profile IPC confirmed; accepted range is 1000–20000 K. The distribution package contains the upstream user service.
- Phase 1: `/displays/night_light`, validation, public CLI, bounded IPC, serialized actions, atomic persistent preferences and runtime overrides implemented. Unit tests cover boundaries, midnight, zones, DST gaps/repetitions, invalid values, persisted versus effective state, IPC failure and idempotency. Session units, dependency and payload staging verified.
- Phase 2: Displays > Night Light, background actions, loading/unavailable states, persistent application errors, prompt validation, search and CLI route implemented. Breadcrumb, back/Esc, selection on resize, route and render tests added. The page was also opened in a real terminal.
- Phase 3: Display card polling/action/refetch and the existing brightness menu integrated with the same CLI. QML lint passed. A Quickshell offscreen component observed external enable/disable at 5000 K without restart; its actual button signal enabled and disabled the backend, and the row hid after the backend stopped. Fourteen isolated shell cases passed, covering the ten brightness actions, both brightness capability states, toggle failure and cancellation.
- Phase 4: English and Portuguese user, command, Control Center, Control Panel, Appearance, Session and shortcut documentation synchronized; catalog key parity and new links checked.

## Decisions and runtime evidence

`argvus-config` is the only persistent writer. All UI surfaces consume `argvus config display night-light`, whose versioned JSON separates saved settings from effective state. Read-back of identity and temperature prevents redundant IPC writes. Runtime action locking prevents parallel quick actions from racing one another.

The scheduler scans real instants in the current local zone, so skipped/repeated civil times remain well-defined. Overrides contain the configuration, compositor instance and next transition; a changed transition invalidates them. Real start/end transitions, manual override, repeated evaluation and override removal were tested using isolated preferences and the current compositor; the system clock and timezone were not changed.

The upstream service starts at 6000 K by default, and the installed 0.4.0 parser rejects `--config <path>`. The package therefore supplies a service drop-in using `--identity` and isolated service-local XDG config discovery with no native profiles. Personal Hyprsunset files are never edited. This drop-in also affects starts of that upstream service outside ARGVUS while the session package is installed; user service overrides may take precedence.

## Validation and remaining session checks

`make validate` in the Rust workspace and session repository, catalog validation, `qmllint`, `sh -n`, ShellCheck, systemd unit verification, `makepkg --printsrcinfo`, payload staging and diff hygiene were run. The final workspace rerun passed, followed by targeted config tests and Clippy after the absolute-runtime-path guard. Full target verification caught a timer ordering cycle; removing the timer dependency on session preparation fixed it, while the oneshot retains that ordering. The complete target verification still emits the existing notification bus-name conflict (`dunst.service: Two services allocated for the same bus name org.freedesktop.Notifications`); the new units verify individually without that warning.

A full logout/login with the newly packaged units has not been performed: the running system was not installed or changed for this task. The upstream service's logout contract was inspected (`PartOf=graphical-session.target`), but actual logout teardown remains an integration check. Midnight, timezone changes and DST were verified with deterministic tests rather than changes to the running system clock. The complete Control Panel was not replaced or restarted; its modified Display component was exercised offscreen, including button actions, external polling and capability loss.

Existing unrelated Hyprland Lua/window-layout documentation edits were preserved. Taskbar source, the shortcut manifest and generated Lua were not changed. No commits or package installation were performed.

## Changed files

### argvus

- `Cargo.lock`
- `crates/argvus-config/Cargo.toml`
- `crates/argvus-config/src/config_validation.rs`
- `crates/argvus-config/src/display.rs`
- `crates/argvus-config/src/lib.rs`
- `crates/argvus-config/src/usr/share/argvus/config/schema.json`
- `crates/argvus-control-center-displays/src/backend.rs`
- `crates/argvus-control-center-displays/src/model.rs`
- `crates/argvus-control-center-displays/src/ui.rs`
- `crates/argvus-control-center-displays/src/ui/input.rs`
- `crates/argvus-control-center-displays/src/ui/jobs.rs`
- `crates/argvus-control-center-displays/src/ui/prompts.rs`
- `crates/argvus-control-center-displays/src/ui/render.rs`
- `crates/argvus-control-center-displays/src/ui/rows.rs`
- `crates/argvus-control-center-displays/src/ui/tests.rs`
- `crates/argvus-control-center-displays/src/ui/tests/profiles.rs`
- `crates/argvus-control-center/src/app.rs`
- `crates/argvus-control-center/src/app/global_search.rs`
- `crates/argvus-control-center/src/app/search_catalog.rs`
- `crates/argvus-control-center/src/cli.rs`
- `crates/argvus-control-center/src/cli/tests.rs`
- `docs/en/components/argvus-control-center/index.md`
- `docs/en/reference/argvus-config.md`
- `docs/en/reference/command-line.md`
- `docs/en/user-guide/index.md`
- `docs/internal/argvus-control-center/ux-audit.md`
- `docs/pt-br/components/argvus-control-center/index.md`
- `docs/pt-br/reference/argvus-config.md`
- `docs/pt-br/reference/command-line.md`
- `docs/pt-br/user-guide/index.md`
- `packaging/archlinux/src/usr/share/argvus/config/schema.json`
- `resources/locales/en-US/appearance.json`
- `resources/locales/en-US/control-center.json`
- `resources/locales/en-US/control-panel.json`
- `resources/locales/en-US/display.json`
- `resources/locales/pt-BR/appearance.json`
- `resources/locales/pt-BR/control-center.json`
- `resources/locales/pt-BR/control-panel.json`
- `resources/locales/pt-BR/display.json`
- `crates/argvus-config/src/night_light.rs`
- `crates/argvus-config/src/night_light/tests.rs`
- `crates/argvus-control-center-displays/src/ui/night_light.rs`
- `docs/en/user-guide/night-light.md`
- `docs/internal/argvus-control-center/night-light-implementation.md`
- `docs/pt-br/user-guide/night-light.md`

### argvus-control-panel

- `docs/en/index.md`
- `docs/pt-br/index.md`
- `src/usr/share/argvus/control-panel/config/quickshell/argvus-control-panel/DisplayCard.qml`
- `src/usr/share/argvus/control-panel/config/quickshell/argvus-control-panel/Strings.qml`

### argvus-appearance

- `docs/en/effects.md`
- `docs/pt-br/effects.md`
- `src/usr/share/argvus/appearance/sh/brightness-switch.sh`

### argvus-session

- `docs/en/graphical-session.md`
- `docs/pt-br/graphical-session.md`
- `packaging/arch/common/functions.sh`
- `packaging/arch/local/PKGBUILD`
- `packaging/arch/remote/PKGBUILD`
- `src/usr/share/argvus/session/config/systemd/user/argvus-session.target`
- `src/usr/lib/systemd/user/hyprsunset.service.d/argvus.conf`
- `src/usr/share/argvus/session/config/night-light/hypr/hyprsunset.conf`
- `src/usr/share/argvus/session/config/systemd/user/argvus-night-light-init.service`
- `src/usr/share/argvus/session/config/systemd/user/argvus-night-light.service`
- `src/usr/share/argvus/session/config/systemd/user/argvus-night-light.timer`

### argvus-hyprland

- `docs/en/keyboard-shortcuts.md`
- `docs/pt-br/keyboard-shortcuts.md`
- `src/usr/share/argvus/hyprland/docs/cheatsheets/en.txt`
- `src/usr/share/argvus/hyprland/docs/cheatsheets/pt.txt`

README files remain unchanged because this addition did not invalidate their existing descriptions. The generated requirements file was not edited; the workspace uses its installed, pinned Rust 1.95.0 toolchain.
