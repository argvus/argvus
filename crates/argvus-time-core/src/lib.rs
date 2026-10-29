//! Time logic shared by ARGVUS Focus and ARGVUS Clock.
//!
//! Nothing here touches GTK. The applications read the clocks once per tick
//! ([`ticks::Ticks::now`]) and pass the readings in, so every state machine is
//! a pure function of its inputs and can be tested without waiting.
//!
//! - [`duration`]: display formats and hour/minute/second conversion.
//! - [`ticks`]: monotonic and boot-time readings, and suspend detection.
//! - [`elapsed`]: a pausable elapsed-time accumulator, with the
//!   [`elapsed::Countdown`] and [`elapsed::Stopwatch`] built on it.
//! - [`focus_status`]: the status file the taskbar indicator of ARGVUS Focus reads.
//! - [`pomodoro`]: Pomodoro phases, cycles and presets.
//! - [`alarm`]: alarms, recurrence in local time, snooze and late alarms.
//! - [`world`]: IANA time zones for the world clocks.
//! - [`store`]: XDG directories and atomic, versioned JSON files.

pub mod alarm;
pub mod duration;
pub mod elapsed;
pub mod focus_status;
pub mod pomodoro;
pub mod store;
pub mod ticks;
pub mod world;
