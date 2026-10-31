---
title: ARGVUS Focus
description: Pomodoro and focus sessions with breaks, notifications and a local session history.
---

ARGVUS Focus is a GTK 4 application for working in focus sessions separated by breaks. It opens as a popup beside the taskbar. Open it from the launcher, by clicking its taskbar indicator, with **SUPER + Shift + F**, or with:

```sh
argvus focus              # open the popup near the pointer
argvus focus --toggle     # hide the popup if it is visible, show it otherwise
argvus focus --toggle --x 1600 --y 30  # open next to a taskbar click position
argvus focus --start      # start or resume the current phase without opening the window
argvus focus --pause      # pause the running phase
argvus focus status --waybar         # print one Waybar JSON status and exit
argvus focus status --waybar --follow  # stream status changes for Waybar
```

The taskbar passes the root click coordinates so the popup uses the clicked monitor and aligns with the bar; a launch without coordinates uses the pointer position. If the pointer stays outside the popup for 5 seconds, it closes automatically; moving the pointer into the popup cancels that countdown. **Esc** on the main page still hides it immediately, and **Esc** on a subpage returns to the previous page. The taskbar indicator shows only its icon while idle and the remaining `MM:SS` during a phase, with the phase colour; a paused phase keeps its remaining time and adds a paused state. The `status` command runs inside `argvus` without starting GTK. `argvus --focus` is an alias. Only one Focus instance runs per session: a second launch hands its options to the running one.

ARGVUS Focus, ARGVUS Clock and the Control Panel share the same screen edge. Opening Focus closes Clock and the Control Panel first; opening either of those surfaces closes Focus.

## Sessions

- **Pomodoro** mode alternates focus and short breaks and ends each cycle with a long break. The default lengths are 25, 5 and 15 minutes, with a long break after 4 focus sessions.
- **Single session** mode runs one focus session of its own length (45 minutes by default) with no breaks.
- The window shows the phase (focus, short break, long break, each in its own colour), the time left inside a progress ring, the position in the cycle and today's total.
- **Start focus / Start break**, **Pause**, **Resume**, **Cancel session** (abandons the phase and records nothing) and **Complete now** (ends the focus session early and records the time actually spent). During a break, **Skip break** goes back to focus.
- When a phase ends, the next one waits for you, unless **Start breaks automatically** or **Start focus automatically after a break** is on.

Keyboard: **Space** starts or pauses, **Ctrl+,** opens the settings, **Ctrl+H** the history, **Esc** goes back, **Ctrl+W** hides the window and **Ctrl+Q** quits.

## Settings

Presets (Classic 25/5/15, Extended 50/10/30, Deep work 90/20/30) fill the lengths, which stay editable. The settings also include the sessions before a long break, the automatic starts, the notification at the end of a phase, and the **session sounds**: a short bell with its own volume and a **Test sound** button. These sounds are separate from the alarm sounds of [ARGVUS Clock](/docs/components/argvus-clock/).

## History

Each completed focus session is recorded with its start, end and the time spent; breaks and cancelled sessions are not. The history page shows today's sessions and minutes, the totals and the recent sessions. Records older than 365 days are dropped. **Clear history** asks for confirmation.

## Window, background and suspend

- Closing the window during a cycle only hides it: the session goes on and notifies you when the phase ends. With nothing under way, closing quits.
- If the system is suspended while a phase runs, the phase is paused at the moment of the suspension, and a message says how long the system slept. Sleeping is not counted as focus time.
- The session is saved on every change and every 30 seconds. If ARGVUS Focus is closed during a phase (logout, crash, **Ctrl+Q**), it comes back paused at that point the next time it opens; it never pretends that the timer kept running while it could not notify.
- Times are measured on the system boot-time clock, so changing the wall clock or the time zone does not shorten or lengthen a phase.

## Paths

| Path | Content |
| --- | --- |
| `$XDG_CONFIG_HOME/argvus/data/focus/settings.json` | Settings |
| `$XDG_DATA_HOME/argvus/focus/history.json` | Completed sessions |
| `$XDG_STATE_HOME/argvus/focus/session.json` | Checkpoint of the current session |
| `/usr/share/argvus/sounds/argvus-bell.oga` | Session bell |

Files are written atomically. A file that cannot be read is renamed to `<name>.corrupt-<time>` and the defaults are used; a message tells you where it went.

The colours follow the active ARGVUS theme and highlight colour (theme consumer `time`). Strings come from the `focus` and `time` translation domains.

The taskbar calendar, ARGVUS Focus, ARGVUS Clock and Control Panel are mutually exclusive: opening one closes the others.
