# ARGVUS Focus: architecture

Crate `crates/argvus-focus` (binary `argvus-focus`, installed as `/usr/lib/argvus/commands/focus`,
run as `argvus focus`). Application ID `sh.argvus.Focus`; layer-shell namespace `argvus-focus`.

- `engine.rs`: the session state machine (`Engine`) over `argvus_time_core::pomodoro::Pomodoro`
  and a `Countdown`. It reads no clock and does no I/O; every method takes the boot-time instant
  and the wall time, and returns `Event`s (`PhaseFinished`, `PausedBySuspend`). Unit tested.
- `history.rs`: completed focus sessions, day summary in local time, 365-day retention.
- `settings.rs`: lengths, mode, notifications, session sounds and volume.
- `app.rs`: GTK lifecycle, actions and accelerators, notifications (`gio::Notification` with an
  `app.start` button), the bell (`SoundPlayer`), checkpoints and the display timer.
- `ui/`: the timer page (progress ring drawn in the widget's CSS colour), settings and history.

## Popup and taskbar status

The undecorated 398-pixel GTK window uses `gtk4-layer-shell` with namespace `argvus-focus` and
`argvus_taskbar_popup::PopupController`. A taskbar click supplies root coordinates; the shared
crate resolves the monitor and horizontal bar, then positions the popup with a 4-pixel gap.
Without coordinates, placement falls back to the pointer. Esc hides the main page and returns
from a subpage. If the pointer remains outside for 5 seconds, the popup is dismissed; entering
the popup cancels that countdown, and leaving it starts another. The shared controller also
guards against reopening from the same click that dismissed it.

`argvus_time_core::focus_status::FocusStatus` is atomically written to
`$XDG_RUNTIME_DIR/argvus/focus/status.json` at state changes and removed on shutdown. It records
the process ID, mode, phase and state; a running phase carries an absolute UTC `ends_at`, while a
paused phase carries `remaining_ms`. The Waybar reader in `argvus-cli` handles
`argvus focus status --waybar [--follow]` in-process, without starting GTK. It ignores a file
whose process no longer exists, emits one JSON line or streams only changed output, and renders
the phase, remaining time and state for the `custom/focus` module.

## Timing

The display is refreshed by a one-shot timeout armed for the moment the shown second changes
(`Engine::until_next_second`), only while a phase runs. Each tick compares the boot-time and
monotonic deltas since the previous tick; a gap of 3 seconds or more means the system slept,
and the phase is paused at the previous tick (`Countdown::pause_at`).

## Persistence

`session.json` is written on every change and every 30 seconds while a phase runs. A started
phase restores as `Paused(Restored)`; the countdown keeps the elapsed time of the checkpoint.

## Window and quitting

Closing the window hides it while the engine is active (running, paused or waiting for the
next phase); the process quits when the window is hidden and the engine is idle. `--start` and
`--pause` act without showing the window.
