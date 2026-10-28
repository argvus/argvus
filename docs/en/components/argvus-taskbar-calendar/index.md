---
title: Calendar
description: Use the ARGVUS taskbar calendar.
---

`argvus-taskbar-calendar` is the GTK4 calendar popup opened from the Waybar date module or with:

```sh
argvus --calendar
argvus taskbar-calendar toggle
argvus taskbar-calendar status
```

It supports local events, reminders and ICS import/export. Its user service is `argvus-taskbar-calendar.service`. Use `argvus taskbar-calendar --help` for options supported by the installed version.

The calendar follows the active ARGVUS theme, including Sticky and Float variants. Theme changes are applied through the generated user cache; use `argvus taskbar-calendar reload` if a running popup needs an explicit refresh.

When opened from the taskbar date module, the popup is anchored to the bar edge: it opens below a top taskbar and above a bottom taskbar, aligned to the date module's right edge. Direct launches without taskbar geometry use the pointer or monitor fallback.

## Commands

```bash
argvus taskbar-calendar                 open the calendar
argvus taskbar-calendar toggle          show or hide it (the Waybar date module uses this)
argvus taskbar-calendar show | hide
argvus taskbar-calendar config          open the system configuration for editing (sudo)
argvus taskbar-calendar config-path     print the configuration path
argvus taskbar-calendar reload
argvus taskbar-calendar import file.ics
argvus taskbar-calendar export --output calendar.ics
argvus taskbar-calendar sync
argvus taskbar-calendar status
argvus taskbar-calendar service
```

## Events and reminders

Events are off by default. Turn them on in the popup under **Settings > Events**; the
choice is kept in the state directory and also enables and starts
`argvus-taskbar-calendar.service`, which shows the reminders. Turning them off stops the
service.

- **+ EVENT** (or `n`) opens the editor for the selected day. Events can be created and
  changed from today on; an event of a past day can still be opened and deleted.
- A new event starts at 09:00 and lasts the default length set in Settings. Moving the
  start time keeps the length; an end time earlier than the start time means the event
  ends on the next day.
- Every event shows a notification with a sound at its start, whatever the
  **Reminder** and **Repeat** boxes say. An event saved after it started (an all-day
  event created during the day, for example) notifies within 30 seconds.
- **Reminder** adds an earlier notification, the chosen time before the start (at the
  start when that time has already passed).
- On an all-day event the time field becomes **NOTIFY AT**: the event covers the whole
  day and notifies at that time (00:00 notifies at the start of the day). The time is
  kept when the event is reopened.
- For all-day events, **Repeat** notifies again at the chosen interval after that time.
- Editing an event without moving its start does not repeat reminders already shown.
- Notifications follow the calendar language (Settings > Language).
- A day shows a dot (`•`) when an event takes place on it, including every day of an
  overnight or multi-day event. Days follow the local time zone.
- Local events are removed automatically 10 minutes after they end.

Imported `.ics` files keep their dates, time zones (`TZID`) and alarms (`VALARM`).
Imported events also notify at their start, besides their own alarms. Recurring events (`RRULE`)
are stored and exported, but only their first occurrence is shown and reminded.
After a package update the reminder service exits and systemd starts the new version
within 30 seconds. `sync` only reports the configured remote calendars; CalDAV synchronisation is not
implemented yet.

## Paths

- Configuration: `/etc/argvus/taskbar/calendar/config.toml` (system-wide; a default ships with the package).
- Styles: `/etc/argvus/taskbar/calendar/style.css` and `theme.css`; the colours of the active ARGVUS theme come from the theme build through the cache below.
- Data: `$XDG_DATA_HOME/argvus-taskbar-calendar/` (or `~/.local/share/argvus-taskbar-calendar/`).
- State: `$XDG_STATE_HOME/argvus/taskbar/calendar/` (or `~/.local/state/...`); the events toggle is kept there.
- Cache: `$XDG_CACHE_HOME/argvus-taskbar-calendar/` (or `~/.cache/argvus-taskbar-calendar/`).

The highlight color is read from `~/.config/argvus/data/.accent-color` and applied at runtime
on top of the theme's colours.
