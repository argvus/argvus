# ARGVUS Clock: architecture

Crate `crates/argvus-clock` (binary `argvus-clock`, installed as `/usr/lib/argvus/commands/clock`,
run as `argvus clock`). Application ID `sh.argvus.Clock`; popup namespace `argvus-clock`.

## Crates

- `argvus-time-core`: timers (`elapsed`), alarms and recurrence (`alarm`), time zones (`world`),
  XDG paths and atomic versioned JSON (`store`), boot-time clock readings (`ticks`). No GTK.
- `argvus-time-ui`: `SoundPlayer`, the `time` theme consumer loader (`ThemeStyle`) and the
  `Catalog`/`Calendar` wrappers over `argvus-i18n` (`time` domain for weekday and month names).
- `argvus-clock`: `app.rs` (lifecycle, heartbeat, ringing, persistence, notifications),
  `ringing.rs` (queue of ringing entries), `timers.rs`, `settings.rs`, `alarms.rs`, `autostart.rs`,
  `cli.rs` and `ui/` (one module per page, plus `ring_window.rs`).

## Scheduling

One heartbeat source (`glib::timeout_add_local_once`, re-armed on every tick):

- While the window is visible, a timer runs or something rings: at the next wall-clock second,
  or earlier when the timer's shown second changes.
- Otherwise, with only alarms pending: at the next alarm trigger, and at most every 30 seconds,
  so a wake-up or a clock change is noticed quickly. No source runs when nothing is pending.

Each tick: `AlarmBook::take_due(now, Local)` marks due occurrences handled, the book is saved,
then on-time occurrences ring and late ones (over `MISSED_GRACE_MINUTES` = 5) are reported as
missed; `Timer::take_finished` rings once; ring entries older than the ring limit stop.

Alarm occurrences use `chrono::Local`, so a change of `/etc/localtime` is picked up. Durations
(timer, stopwatch) use `CLOCK_BOOTTIME` through `Ticks`, which keeps counting during a
suspension and ignores wall-clock changes.

## Popup window

The undecorated 398-pixel GTK window is a layer-shell overlay with namespace `argvus-clock`.
`argvus_taskbar_popup::PopupController` positions it beside the taskbar using the root click
coordinates, monitor and horizontal bar geometry, with a 4-pixel gap. Without coordinates it
uses the pointer position. Esc hides the main tabs and returns from a full page; losing focus
does not itself immediately hide the popup. If the pointer remains outside for 5 seconds, the
popup is dismissed; entering it cancels that countdown, and leaving it starts another. The
controller also handles keyboard focus and the same-click reopen guard shared with Focus and the
calendar. The calendar retains its existing 800 ms focus-dismiss behavior.

The alarm/timer ringing card remains a separate layer-shell surface (`argvus-clock-alarm`). The
alarm editor hides the popup while opening GTK's `FileDialog`, then presents it again at its
previous anchor when the dialog finishes. Scheduler, alarms and background policy do not depend
on the popup being visible.

## Single instance

`GApplication` with `HANDLES_COMMAND_LINE`: the first process owns `sh.argvus.Clock` on the
session bus and runs the scheduler; later launches forward their arguments and exit. `main`
parses the arguments first, so `--help` and errors are printed by the launching process.
Notification buttons use the application actions `app.snooze-alarm(t)`, `app.dismiss-alarm(t)`
and `app.dismiss-timer`.

## Background and quitting

`quit_if_idle` quits when the window is hidden, nothing rings, and either no work is pending
(enabled or snoozed alarms, a running timer or stopwatch) or background running is not allowed
(setting off and not started with `--background`). `argvus-clock.service` is enabled or disabled
with `systemctl --user enable|disable` through `GSubprocess`, without `--now`.

## Ring window

A `gtk::Window` created when the queue becomes non-empty and destroyed when it empties. With
`gtk4-layer-shell` it is an overlay surface (namespace `argvus-clock-alarm`, anchored to the top,
keyboard on demand so typing elsewhere cannot dismiss it). GTK can log
`gdk_gl_context_make_current() failed` when such a surface is created or destroyed on some
drivers; the card still renders.

## Files

`settings.json` (config), `alarms.json` (data), `timers.json` (state): envelopes
`{"version": 1, "data": …}` written by `argvus_time_core::store::save_json`.
