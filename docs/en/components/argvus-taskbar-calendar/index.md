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

## Paths

- Configuration: `/etc/argvus/taskbar/calendar/config.toml` (system-wide; a default ships with the package).
- Styles: `/etc/argvus/taskbar/calendar/style.css` and `theme.css`; the colours of the active ARGVUS theme come from the theme build through the cache below.
- Data: `$XDG_DATA_HOME/argvus-taskbar-calendar/` (or `~/.local/share/argvus-taskbar-calendar/`).
- State: `$XDG_STATE_HOME/argvus/taskbar/calendar/` (or `~/.local/state/...`); the events toggle is kept there.
- Cache: `$XDG_CACHE_HOME/argvus-taskbar-calendar/` (or `~/.cache/argvus-taskbar-calendar/`).

The highlight color is read from `~/.config/argvus/data/.accent-color` and applied at runtime
on top of the theme's colours.
