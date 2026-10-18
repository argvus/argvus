# Argvus Calendar — Wiki

Documentation of what each module and function does. The app is a native Wayland
popup calendar (GTK4 + Relm4 + `gtk4-layer-shell`) that opens exactly where the
user clicks in Waybar, stores events in SQLite and schedules desktop reminders.

## Entry point — `crates/main/src/main.rs`

Parses the CLI with `clap` and dispatches each command.

- `main()` — bootstraps logging, accepts the immutable `--x`/`--y` position
  reconstructed from Waybar's original button event and layer-surface origin
  (with a live-pointer fallback for direct CLI calls), resolves paths and loads
  `Settings`, purges ended events, then
  dispatches the subcommand. `toggle`/`show` pass that position to a running
  instance over IPC; when none is running they spawn the UI (`ui::run_ui`).

Subcommands: `Toggle`, `Show`, `Hide`, `Config`, `ConfigPath`, `Reload`,
`Import { file }`, `Export { output }`, `Sync`, `Status`, `Service`.

## CLI operations — `src/app.rs`

- `import_file(paths, file)` — opens the database, imports the ICS file into the
  default calendar and prints how many events were added.
- `export_file(paths, output)` — reads all events and writes them as ICS.
- `sync_once(paths, settings)` — reports the configured sync interval and how
  many remote calendars are enabled.
- `status(paths)` — prints the state of the system: database, config/style/theme
  paths, events/reminders state, calendar and event counts.

## Configuration — `src/config/settings.rs`

`AppConfig` (alias `Settings`) is a `serde` struct with `appearance`, `locale`,
`calendar`, `editor` and `terminal` sections. The system file under
`/etc/argvus/taskbar/calendar/config.toml` acts as the default base and the user file is
layered on top, so no elevated privileges are needed to persist changes.

- `Settings::load(paths)` — loads the system config, merges the user config over
  it (TOML deep merge) and validates the result.
- `Settings::save(paths)` — serializes the full config to the user-level file.
- `Settings::validate(config)` — clamps font size, and falls back to valid
  language/week-start values.
- `Settings::week_start()` — maps the week-start string to the `WeekStart` enum.
- `default_*()` helpers — the serde defaults for each key.
- `load_system_config(paths)` — reads the system config file (or the legacy
  `settings.toml` location when the modern file is missing); returns defaults
  when absent.
- `load_user_config(paths)` — reads the user config, if present.
- `merge_toml(base, override_)` — deep-merges the user TOML over the system
  values (recurses into tables).
- `effective_config_file(paths)` — the file `open_config` edits: the user file
  once it exists, otherwise the system file.
- `list_theme_names(paths)` — theme names found in the theme directory.
- `active_theme_name(paths)` — the ARGVUS theme recorded as active.
- `write_active_theme(paths, name)` — persists the active ARGVUS theme.
- `resolve_paths()` — builds `Paths` from XDG dirs (with `XDG_CONFIG_HOME` etc.
  and a user-local fallback).
- `load_events_enabled(paths, config)` — the effective events/reminders state:
  the state file wins over the legacy config key; missing state defaults to off.
- `service_events_enabled(paths)` — reads the persisted events/reminders state and
  defaults to off until the user explicitly enables events.
- `save_events_enabled(paths, enabled)` — persists the state toggle in
  `paths.state_dir` (`~/.local/state/argvus/taskbar/calendar/events-enabled`).
- `read_events_enabled(paths)` — raw state file read.
- `open_config(paths, config)` — opens the config file in the configured editor.
- `editor_terminal_command(config, path)` — builds the terminal command to run
  the editor with sudo (kitty/foot and fallbacks).
- `ensure(path)` / `xdg_home(var, fallback)` — directory helpers.
- `validate_export_path(path)` — rejects exporting over the config/theme files.

## IPC — `src/ipc.rs`

Single-instance control over a UNIX socket in the cache dir.

- `IpcCommand::as_str` / `parse` — serialize/deserialize commands (`toggle`,
  `show`, `hide`, `reload-theme`, `reload-config`).
- `IpcMessage` — a command plus the fixed click position supplied by the Waybar
  launcher, so the running UI does not re-read the cursor at present time.
- `socket_path(paths)` — the socket location.
- `notify(paths, message)` — connects to a running instance, writes the message
  and returns true when it acknowledged.
- `listen(paths)` — binds the socket; handles a stale socket file left by a dead
  instance and returns `None` when another instance is alive.
- `serve(listener, on_command)` — background thread dispatching messages to the
  UI and replying `ok`.

## Calendar domain — `src/calendar/`

### `date.rs`

- `WeekStart::offset_for(date)` — weekday offset for Monday- or Sunday-start.
- `today()` — the current local date.
- `start_of_local_day_utc(date)` — midnight of the local day as UTC.
- `start_of_today_utc()` — midnight of today in UTC.
- `first_of_month(date)` / `next_month(date)` / `previous_month(date)` — month
  navigation.
- `shift_year_clamped(date, years)` — shifts the year, clamping a Feb 29 to
  Feb 28 on non-leap years.
- `month_grid(anchor, week_start)` — the calendar grid of `NaiveDate`s for the
  month, padded with days from adjacent months.

### `event.rs`

- `CalendarSource` — `Local` / `CalDav` / `ImportedIcs`, with string conversion
  for storage.
- `CalendarEvent::new_local(...)` — builds an event in the local calendar.
- `CalendarEvent::validate()` — checks title non-empty and start before end.

### `recurrence.rs`

- `Recurrence::is_empty()` — whether the recurrence is disabled.

## Storage — `src/storage/database.rs`

SQLite persistence (rusqlite). `Database::open(path)` runs migrations on open.

- `open(path)` — opens the database and migrates the schema.
- `migrate()` — applies pending migrations.
- `default_calendar_id()` — the Local calendar id.
- `calendars()` — all calendars.
- `events_for_day(date)` — events for one local day (all-day included).
- `events_between(start, end)` — events in a UTC range.
- `all_events()` — every event.
- `event_by_id(id)` — one event with its reminders.
- `upsert_event(&mut event)` — inserts or updates an event plus its reminders,
  returning the id.
- `mark_deleted(id)` — soft-deletes an event.
- `purge_events_ended_before(cutoff)` — removes ended events and their
  reminders.
- `purge_expired_local_single_events(now)` — removes local non-recurring events
  10 minutes after their end time, preserving CalDAV/ICS and recurring events.
- `due_reminders(now)` — reminders whose trigger time has passed and have not
  fired, joined with their events, including recently missed reminders inside a
  short grace window.
- `ensure_start_reminders_for_unscheduled_due_events(now)` — creates a start-time
  reminder for due events that were saved without explicit reminders.
- `mark_reminder_fired(id, when)` — records that a reminder fired.
- `event_from_row` / `event_from_offset_row` / `reminders_for_event` — row
  mapping helpers.
- `parse_datetime` / `parse_optional_datetime` — SQLite text to `DateTime<Utc>`.

## ICS import/export — `src/ical/`

- `import_ics_file(path, calendar_id)` — reads a file and parses it.
- `parse_ics(contents, calendar_id)` — parses `DTSTART`/`DTEND`, all-day flags,
  reminder `TRIGGER`s and `RRULE` recurrence into `CalendarEvent`s.
- `parse_ics_datetime(value)` — flexible date/datetime parsing.
- `trigger_to_minutes(trigger)` — ISO-8601 `TRIGGER` to minutes.
- `midnight()` — helper for all-day boundary.
- `export_ics_file(path, events)` — writes events as an ICS file.
- `export_ics(events)` — serializes events to an ICS string.

## Reminders — `src/reminders/`

### `notification.rs`

- `notify_event(event, minutes_before)` — shows a desktop notification
  (notify-rust) with a message appropriate to all-day/now/later and requests a
  bell sound, with desktop audio command fallbacks.

### `scheduler.rs`

- `ReminderScheduler::new(paths)` — scheduler polling every 30s.
- `run()` — endless loop calling `tick` and sleeping.
- `tick()` — purges expired local non-recurring events, creates start-time
  reminders for due events without explicit reminders, fetches due reminders,
  skips stale repeat triggers and fires notifications.
- `is_stale_repeat(start, minutes_before, now)` — avoids re-notifying old
  repetitions of an in-progress all-day event.

### `service/mod.rs`

- `run_service(paths)` — the `argvus-calendar service` entry point; exits
  immediately when events/reminders are disabled, otherwise runs the scheduler.

## CalDAV foundation — `src/caldav/`

- `client.rs::CalDavClient::new(base_url, username, password)` — HTTP client with
  Basic auth.
- `client.rs::discover_collections()` — `PROPFIND` to list calendar collections.
- `client.rs::list_objects(collection_url)` — lists remote calendar objects.
- `client.rs::put_object(...)` / `delete_object(...)` — push and remove remote
  objects (with ETag).
- `client.rs::dav(...)` — low-level WebDAV HTTP helper.
- `client.rs::parse_collections(xml)` / `parse_remote_objects(xml)` /
  `local_name(name)` — XML parsing helpers (quick-xml).
- `discovery.rs::discover(client)` — convenience wrapper around collection
  discovery.
- `credentials.rs::store_password(...)` / `load_password(lookup)` — keyring-backed; needs a Secret Service provider on the session bus (see `argvus-session` docs)
  credential helpers.
- `sync.rs::CalDavSyncService::new(client)` and
  `initial_sync(collection_url)` — first-pass one-way pull producing a
  `SyncReport`.
- `types.rs` — the `CalDavAccount`, `CalendarCollection`, `RemoteObject` and
  `SyncReport` types.

## UI — `src/ui/`

### `mod.rs`

- Re-exports the popup command used by `main.rs`.

### `app.rs`

The Relm4 component: the whole popup is one layer-shell surface.

- `run_ui(paths, settings, command, fixed_position)` — entry point. Pins the
  popup to the click position supplied by the Waybar launcher, with an
  in-process fallback read for direct CLI calls, before building the GTK app.
- `AppModel::init` — builds the window, calendar, agenda and editor widgets,
  starts the theme watcher and the IPC listener, and presents when the command
  asks for it.
- `AppModel::update` — handles navigation, day selection, event CRUD, settings
  editing and popup show/hide; reloads events when the state changed.
- `render_model` — pushes the current month/selection/agenda into the widgets.
- `reapply_runtime` — reapplies locale, CSS and layer-shell settings after a
  config save.
- `present_popup(window, popup, state, fixed_position)` — repositions the window
  at the fixed click position, starts a new presentation generation, sets
  exclusive keyboard mode and presents. After 800ms it arms outside-click
  dismissal and downgrades keyboard mode; stale timers cannot affect a newer
  opening.
- `set_monitor_at_pointer(window, fixed_position)` — moves the surface to the
  monitor that contains the click position.
- `apply_layer_shell(window)` — layer-shell setup: overlay layer, anchored
  top-right, exclusive zone 0.
- `position_popup(window, fixed_position)` — sets the top/right margins from the
  computed position and returns the popup bounds for outside-click detection.
- `popup_position(fixed_position)` — math: converts the pointer position into
  margin values 12 pixels below the click (relative to the top-right anchor and
  the reserved Waybar strip), clamped to the monitor.
- `pointer_is_outside_popup(bounds)` — checks the live cursor against the stored
  bounds.
- `current_pointer_position()` — reads the cursor via `hyprctl cursorpos -j`.
- `current_monitor_geometry(x, y)` — the monitor geometry and reserved top strip
  for a point, via `hyprctl monitors -j`.
- `load_css(paths)` — loads the CSS cascade.
- `stylesheet_paths(paths)` — style.css, theme.css, the active ARGVUS theme and
  the user cache theme, in that order.
- `runtime_css(paths)` — font CSS generated from config.
- `css_string(value)` — CSS-escaping helper.
- `load_day_and_month(paths, selected, month)` — events + event-day set.
- `hide_popup`, `cancel_editor`, `update_draft`, `update_settings_draft` — small
  state helpers.
- `control_reminder_service(enabled)` — enables/disables the user systemd unit.
- Widget builders: `editor_row`, `editor_label`, `editor_entry`,
  `editor_number_spin`, `editor_control_has_focus`, `settings_label`,
  `settings_section_label`, `settings_number_spin`, `can_create_event`,
  `file_fingerprint`, `theme_fingerprint` — GTK construction/helpers.

### `calendar.rs`

- `day_cells(month, week_start)` — the grid of (date, in-month) cells.
- `weekday_labels(week_start)` — the header labels.

### `agenda.rs`

- `rebuild_agenda(list, events, i18n, on_event)` — repopulates the day agenda
  list from events.

### `event_editor.rs`

- `EditorDraft::new(date, default_reminder_minutes)` / `from_event(event)` —
  build a draft for a new or existing event.
- `build_editor()` — the editor form widgets.
- `populate_editor(widgets, draft, i18n)` — fills the form and blocks signals so
  typing does not re-enter the UI loop.
- `save_editor(widgets, draft, paths)` — writes the event to the database.
- `build_reminders(start, all_day, enabled, hours, minutes, repeat_*)` — expands
  repeat/all-day reminders into concrete `Reminder`s.
- `local_datetime(date, hour, minute)` — local components to `DateTime<Utc>`.
- `empty_to_none`, `set_entry_text`, `set_number` — widget helpers.

### `settings.rs`

- `SettingsDraft::from_config(...)` / `into_config()` — snapshot the current
  settings for the settings screen and rebuild a `Settings` on save (with
  clamping and fallbacks).
- `build_settings()` — the settings form container.
- `set_dropdown(dropdown, model, value)` / `dropdown_value(dropdown)` — dropdown
  helpers.
- `populate_settings(widgets, draft, i18n)` — fill the form, blocking signal
  handlers while setting values.

## Errors — `src/error.rs`

`ArgvusError` enum and the `Result<T>` alias used everywhere.
