# Internal notes

Developer-only material of the components that were consolidated into the `argvus`
workspace. These files are not published on the user site.

- `argvus-clock/`: architecture of ARGVUS Clock (scheduler, popup, single instance, ring window) and the origin, license and playback of the sounds shared with ARGVUS Focus.
- `argvus-control-center/`: UX audit and the changelogs of the Control Center migration (phases and recovery notes).
- `argvus-focus/`: architecture of ARGVUS Focus (session engine, popup, taskbar status, suspend handling, checkpoints).
- `argvus-i18n/`: the i18n developer guide.
- `argvus-taskbar-calendar/`: UI architecture, the shared popup crate, dependency decisions, Waybar integration notes and the systemd unit reference.
- `argvus-tui/`: the terminal UI framework (crates `argvus-tui`, `argvus-theme`), its row kinds and footer rules.

The user-facing pages of the same components are under `en/components/` and `pt-br/components/`.
