---
title: ARGVUS Clock
description: Clock, world clocks, stopwatch, countdown timer and alarms with sound.
---

ARGVUS Clock is a GTK 4 application with the current time and the time tools: world clocks, a stopwatch, a countdown timer and alarms. Open it from the launcher, by clicking the time in the taskbar, with **SUPER + Shift + C**, or with:

```sh
argvus clock                     # open the popup near the pointer
argvus clock --toggle            # hide the popup if it is visible, show it otherwise
argvus clock --toggle --x 1600 --y 30  # open next to a taskbar click position
argvus clock --page alarms       # open on a page: clock, world, stopwatch, timer, alarms
argvus clock --background        # run without a window, to keep alarms active
```

ARGVUS Clock opens as a popup aligned with the taskbar, with a 4-pixel gap. The taskbar passes the root click coordinates so the popup uses the clicked monitor; launching it without coordinates uses the pointer position. If the pointer stays outside the popup for 5 seconds, it closes automatically; moving the pointer into the popup cancels the countdown. **Esc** on a main tab still hides it immediately, and **Esc** on a full page returns to the previous tab. `argvus --clock` is an alias. Clicking the date in the taskbar still opens the calendar; ARGVUS Clock does not show events.

## Pages

- **Clock**: the time in large digits (24-hour by default, 12-hour in the settings, seconds optional), the localized date, the local time zone and its UTC offset, and the next alarm.
- **World**: cities added by searching a city or IANA time zone (accents and case are ignored). Each shows its time, the difference to local time and whether it is yesterday, today or tomorrow there. Daylight saving comes from the IANA time zone database; nothing is computed with fixed offsets. Cities can be reordered and removed.
- **Stopwatch**: start, pause, resume, reset and laps (lap time and total).
- **Timer**: hours, minutes and seconds, or the presets 5, 10, 15, 25, 30 and 60 minutes. Pause, resume, restart and cancel. When it ends it rings like an alarm. This timer is independent of ARGVUS Focus sessions.
- **Alarms**: create, edit, switch on and off and delete alarms. Each has a time, an optional name, the weekdays it repeats on (none means it rings once), a sound and a snooze length.

Keyboard: **Ctrl+1** to **Ctrl+5** switch pages, **Ctrl+N** creates an alarm, **Ctrl+,** opens the settings, **Esc** goes back, **Ctrl+W** closes the window and **Ctrl+Q** quits.

## Alarm sound

Alarms play a real sound until you answer them:

- Sounds: **Chime** (default) and **Beacon**, shipped in `/usr/share/argvus/sounds/`, an audio file of your own, or **No sound**, which must be chosen explicitly. If your file disappears, Chime plays and a message says so.
- **Alarm volume** (10 to 100, default 80) and **Listen to the alarm sound** are in the settings; the editor has a test button for the chosen sound.
- A ringing alarm shows a card at the top of the screen, above other windows, and a notification, both with **Snooze** and **Dismiss**. With the card focused, **Enter** dismisses and **S** snoozes.
- Snooze rings again after the alarm's snooze length (1 to 60 minutes, default 10).
- An alarm that is not answered stops after **Stop ringing after** (5, 10, 15 or 30 minutes, default 10) and leaves a notification.
- Two alarms ringing together are both listed; the second never silences the first.
- If the sound cannot be played, the card and the window say so with the error, and the error is logged. ARGVUS Clock never reports a sound that did not play.

Playback uses GTK's media support (GStreamer), so it works with PipeWire, PulseAudio or ALSA. The package depends on `gst-plugins-base` (Ogg Vorbis) and `gst-plugins-good` (audio output).

## When alarms ring

Alarms ring only while ARGVUS Clock runs in your session:

- Closing the window keeps ARGVUS Clock running while alarms, a running timer or a running stopwatch need it (**Keep running when the window is closed**, on by default). With nothing pending, it quits.
- **Start with the session** enables the user unit `argvus-clock.service`, which runs `argvus clock --background` after login. It is off by default.
- Alarms cannot ring while you are logged out, the computer is off or it is suspended.
- An alarm noticed up to 5 minutes late still rings. Later than that (the clock was not running, or the system was asleep or off) it does not ring: it is reported as **missed** in a notification and on the Alarms page. A repeating alarm goes on to its next day; a one-time alarm switches off.
- Each occurrence is saved as handled before it rings, so reopening ARGVUS Clock never rings the same alarm twice, and only one instance runs per session.
- Repeating alarms are computed in local time on the chosen weekdays, so they keep their time across daylight-saving changes. A time skipped by the spring change rings at the first minute after the gap; a time that happens twice rings the first time.
- A timer keeps counting during a suspension. If it ended while the system slept, it rings on wake-up and says how late it is. After ARGVUS Clock was closed, a running timer or stopwatch comes back paused.

## Paths

| Path | Content |
| --- | --- |
| `$XDG_CONFIG_HOME/argvus/data/clock/settings.json` | Settings and world clocks |
| `$XDG_DATA_HOME/argvus/clock/alarms.json` | Alarms |
| `$XDG_STATE_HOME/argvus/clock/timers.json` | Timer and stopwatch checkpoint |
| `/usr/lib/systemd/user/argvus-clock.service` | Start with the session |
| `/usr/share/argvus/sounds/` | Packaged sounds |

Files are written atomically; a file that cannot be read is renamed to `<name>.corrupt-<time>` and defaults are used. The colours follow the active ARGVUS theme and highlight colour (theme consumer `time`). Strings come from the `clock` and `time` translation domains.
